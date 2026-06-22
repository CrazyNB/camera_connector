use std::collections::hash_map::DefaultHasher;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use image::{DynamicImage, ImageDecoder, ImageReader};
use serde::{Deserialize, Serialize};
use tauri::State;

use super::{desktop_error, DesktopError, DesktopState};

#[path = "thumbnail_raw.rs"]
mod thumbnail_raw;
pub(super) use thumbnail_raw::raw_sensor_thumbnail_image;

#[path = "thumbnail_embedded.rs"]
mod thumbnail_embedded;
pub(super) use thumbnail_embedded::{
    embedded_jpeg_preview_image, is_browser_original_extension, is_raw_extension,
    raw_thumbnail_image_from_file,
};

fn thumbnail_error(message: impl Into<String>) -> DesktopError {
    DesktopError {
        code: "thumbnail".to_string(),
        message: message.into(),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThumbnailRequest {
    pub source_path: String,
    pub max_edge: Option<u32>,
    pub quality: Option<ThumbnailQuality>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OriginalPreviewRequest {
    pub source_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThumbnailBatchRequest {
    pub source_paths: Vec<String>,
    pub max_edge: Option<u32>,
    pub quality: Option<ThumbnailQuality>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThumbnailQuality {
    #[default]
    Fast,
    Full,
}

impl ThumbnailQuality {
    fn cache_dir_name(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Full => "full",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ThumbnailResponse {
    pub path: String,
    pub cached: bool,
    pub quality: ThumbnailQuality,
}

#[derive(Debug, Clone, Serialize)]
pub struct OriginalPreviewResponse {
    pub path: String,
    pub cached: bool,
    pub direct_source: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ThumbnailBatchItem {
    pub source_path: String,
    pub path: Option<String>,
    pub cached: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ThumbnailBatchResponse {
    pub thumbnails: Vec<ThumbnailBatchItem>,
}

#[tauri::command]
pub async fn get_asset_thumbnail(
    state: State<'_, DesktopState>,
    request: ThumbnailRequest,
) -> Result<ThumbnailResponse, DesktopError> {
    let state_dir = state.service.storage_state_dir().map_err(desktop_error)?;
    tauri::async_runtime::spawn_blocking(move || get_asset_thumbnail_blocking(state_dir, request))
        .await
        .map_err(|error| thumbnail_error(format!("thumbnail task failed: {error}")))?
}

#[tauri::command]
pub async fn get_asset_original_preview(
    state: State<'_, DesktopState>,
    request: OriginalPreviewRequest,
) -> Result<OriginalPreviewResponse, DesktopError> {
    let state_dir = state.service.storage_state_dir().map_err(desktop_error)?;
    tauri::async_runtime::spawn_blocking(move || {
        get_asset_original_preview_blocking(state_dir, request)
    })
    .await
    .map_err(|error| thumbnail_error(format!("original preview task failed: {error}")))?
}

#[tauri::command]
pub async fn get_asset_thumbnails(
    state: State<'_, DesktopState>,
    request: ThumbnailBatchRequest,
) -> Result<ThumbnailBatchResponse, DesktopError> {
    let state_dir = state.service.storage_state_dir().map_err(desktop_error)?;
    let source_paths = request.source_paths;
    let max_edge = request.max_edge;
    tauri::async_runtime::spawn_blocking(move || {
        let thumbnails = source_paths
            .into_iter()
            .map(|source_path| {
                let thumbnail_request = ThumbnailRequest {
                    source_path: source_path.clone(),
                    max_edge,
                    quality: request.quality,
                };
                match get_asset_thumbnail_blocking(state_dir.clone(), thumbnail_request) {
                    Ok(response) => ThumbnailBatchItem {
                        source_path,
                        path: Some(response.path),
                        cached: response.cached,
                        error: None,
                    },
                    Err(error) => ThumbnailBatchItem {
                        source_path,
                        path: None,
                        cached: false,
                        error: Some(error.message),
                    },
                }
            })
            .collect();
        Ok(ThumbnailBatchResponse { thumbnails })
    })
    .await
    .map_err(|error| thumbnail_error(format!("thumbnail batch task failed: {error}")))?
}

pub(super) fn get_asset_thumbnail_blocking(
    state_dir: PathBuf,
    request: ThumbnailRequest,
) -> Result<ThumbnailResponse, DesktopError> {
    let source_path = PathBuf::from(request.source_path);
    let metadata = fs::metadata(&source_path)
        .map_err(|error| thumbnail_error(format!("source image is not readable: {error}")))?;
    if !metadata.is_file() {
        return Err(thumbnail_error("source image is not a file"));
    }

    let max_edge = request.max_edge.unwrap_or(512).clamp(160, 1280);
    let quality = request.quality.unwrap_or_default();
    let cache_dir = state_dir
        .join("thumb-cache")
        .join("v5")
        .join(quality.cache_dir_name());
    fs::create_dir_all(&cache_dir).map_err(|error| {
        thumbnail_error(format!("thumbnail cache could not be created: {error}"))
    })?;
    let cache_key = thumbnail_cache_key(&source_path, &metadata, max_edge);
    let output_path = cache_dir.join(format!("{cache_key}.jpg"));
    if output_path.is_file() {
        return Ok(ThumbnailResponse {
            path: output_path.to_string_lossy().into_owned(),
            cached: true,
            quality,
        });
    }

    write_thumbnail_with_quality(&source_path, &output_path, max_edge, quality)?;
    Ok(ThumbnailResponse {
        path: output_path.to_string_lossy().into_owned(),
        cached: false,
        quality,
    })
}

pub(super) fn get_asset_original_preview_blocking(
    state_dir: PathBuf,
    request: OriginalPreviewRequest,
) -> Result<OriginalPreviewResponse, DesktopError> {
    let source_path = PathBuf::from(request.source_path);
    let metadata = fs::metadata(&source_path)
        .map_err(|error| thumbnail_error(format!("source image is not readable: {error}")))?;
    if !metadata.is_file() {
        return Err(thumbnail_error("source image is not a file"));
    }

    if is_browser_original_extension(&source_path) {
        return Ok(OriginalPreviewResponse {
            path: source_path.to_string_lossy().into_owned(),
            cached: true,
            direct_source: true,
        });
    }

    let cache_dir = state_dir.join("preview-cache").join("v1").join("original");
    fs::create_dir_all(&cache_dir).map_err(|error| {
        thumbnail_error(format!(
            "original preview cache could not be created: {error}"
        ))
    })?;
    let cache_key = original_preview_cache_key(&source_path, &metadata);
    let output_path = cache_dir.join(format!("{cache_key}.jpg"));
    if output_path.is_file() {
        return Ok(OriginalPreviewResponse {
            path: output_path.to_string_lossy().into_owned(),
            cached: true,
            direct_source: false,
        });
    }

    let mut image = if is_raw_extension(&source_path) {
        raw_thumbnail_image_from_file(&source_path)?
    } else {
        decoded_image_from_file(&source_path)?
    };
    write_original_preview_image(&mut image, &output_path)?;
    Ok(OriginalPreviewResponse {
        path: output_path.to_string_lossy().into_owned(),
        cached: false,
        direct_source: false,
    })
}

pub(super) fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

fn thumbnail_cache_key(source_path: &Path, metadata: &fs::Metadata, max_edge: u32) -> String {
    let canonical_path = source_path
        .canonicalize()
        .unwrap_or_else(|_| source_path.to_path_buf());
    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    canonical_path.to_string_lossy().hash(&mut hasher);
    metadata.len().hash(&mut hasher);
    modified_at.hash(&mut hasher);
    max_edge.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn original_preview_cache_key(source_path: &Path, metadata: &fs::Metadata) -> String {
    let canonical_path = source_path
        .canonicalize()
        .unwrap_or_else(|_| source_path.to_path_buf());
    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    canonical_path.to_string_lossy().hash(&mut hasher);
    metadata.len().hash(&mut hasher);
    modified_at.hash(&mut hasher);
    "original-preview-v1".hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub(super) fn write_thumbnail_with_quality(
    source_path: &Path,
    output_path: &Path,
    max_edge: u32,
    quality: ThumbnailQuality,
) -> Result<(), DesktopError> {
    if quality == ThumbnailQuality::Fast {
        if let Some(mut image) = embedded_jpeg_preview_image(source_path) {
            return write_thumbnail_image(&mut image, output_path, max_edge);
        }
    }

    if quality == ThumbnailQuality::Full && is_raw_extension(source_path) {
        if let Ok(mut image) = raw_thumbnail_image_from_file(source_path) {
            return write_thumbnail_image(&mut image, output_path, max_edge);
        }
    }

    let mut image = decoded_image_from_file(source_path)?;
    write_thumbnail_image(&mut image, output_path, max_edge)
}

fn decoded_image_from_file(source_path: &Path) -> Result<DynamicImage, DesktopError> {
    let mut decoder = ImageReader::open(source_path)
        .map_err(|error| thumbnail_error(format!("source image could not be opened: {error}")))?
        .with_guessed_format()
        .map_err(|error| {
            thumbnail_error(format!(
                "source image format could not be detected: {error}"
            ))
        })?
        .into_decoder()
        .map_err(|error| thumbnail_error(format!("source image could not be decoded: {error}")))?;
    let orientation = decoder.orientation().map_err(|error| {
        thumbnail_error(format!(
            "source image orientation could not be read: {error}"
        ))
    })?;
    let mut image = DynamicImage::from_decoder(decoder)
        .map_err(|error| thumbnail_error(format!("source image could not be decoded: {error}")))?;
    image.apply_orientation(orientation);
    Ok(image)
}

fn write_thumbnail_image(
    image: &mut DynamicImage,
    output_path: &Path,
    max_edge: u32,
) -> Result<(), DesktopError> {
    let thumbnail = image.thumbnail(max_edge, max_edge);
    let temporary_path = output_path.with_extension(format!("jpg.{}.tmp", current_time_ms()));
    let file = File::create(&temporary_path).map_err(|error| {
        thumbnail_error(format!("thumbnail file could not be created: {error}"))
    })?;
    let mut writer = BufWriter::new(file);
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, 78);
    encoder
        .encode_image(&thumbnail)
        .map_err(|error| thumbnail_error(format!("thumbnail could not be encoded: {error}")))?;
    writer.flush().map_err(|error| {
        thumbnail_error(format!("thumbnail file could not be flushed: {error}"))
    })?;
    drop(writer);
    match fs::rename(&temporary_path, output_path) {
        Ok(()) => Ok(()),
        Err(rename_error) => {
            fs::copy(&temporary_path, output_path).map_err(|copy_error| {
                thumbnail_error(format!(
                    "thumbnail file could not be stored: {copy_error}; rename failed with {rename_error}"
                ))
            })?;
            let _ = fs::remove_file(&temporary_path);
            Ok(())
        }
    }
}

pub(super) fn write_original_preview_image(
    image: &mut DynamicImage,
    output_path: &Path,
) -> Result<(), DesktopError> {
    let temporary_path = output_path.with_extension(format!("jpg.{}.tmp", current_time_ms()));
    let file = File::create(&temporary_path).map_err(|error| {
        thumbnail_error(format!(
            "original preview file could not be created: {error}"
        ))
    })?;
    let mut writer = BufWriter::new(file);
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, 90);
    encoder.encode_image(image).map_err(|error| {
        thumbnail_error(format!("original preview could not be encoded: {error}"))
    })?;
    writer.flush().map_err(|error| {
        thumbnail_error(format!(
            "original preview file could not be flushed: {error}"
        ))
    })?;
    drop(writer);
    match fs::rename(&temporary_path, output_path) {
        Ok(()) => Ok(()),
        Err(rename_error) => {
            fs::copy(&temporary_path, output_path).map_err(|copy_error| {
                thumbnail_error(format!(
                    "original preview file could not be stored: {copy_error}; rename failed with {rename_error}"
                ))
            })?;
            let _ = fs::remove_file(&temporary_path);
            Ok(())
        }
    }
}
