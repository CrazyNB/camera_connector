use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use camera_connector_core::{
    AssetGroupQuery, CameraConnectorService, CvPolicy, EvaluationRunStatus, PreviewSample,
    ProjectEvaluationSettings, StoredAsset, SubjectAssessment, TechnicalAssessmentPolicy,
};
use image::{DynamicImage, GrayImage, ImageDecoder, ImageReader, RgbImage};
use pico_detect::{
    clusterize::Clusterizer, multiscale::Multiscaler, DetectMultiscale, Detector, Region,
};
use serde_json::json;
use tauri::{AppHandle, Emitter};

use super::thumbnails::{
    embedded_jpeg_preview_image, is_raw_extension, raw_thumbnail_image_from_file,
};
use super::{
    current_time_ms, desktop_error, thumbnail_error, DesktopCvAssessmentProgress,
    DesktopCvAssessmentRequest, DesktopCvAssessmentResponse, DesktopError,
};

const DESKTOP_FACE_DETECTOR_MODEL: &[u8] = include_bytes!("../assets/models/face.detector.bin");
const DESKTOP_FACE_DETECTOR_VERSION: &str = "pico-detect-0.7.0/face.detector.bin";
const DESKTOP_FACE_DETECTION_MAX_EDGE: u32 = 1024;
const DESKTOP_FACE_MIN_SIZE_RATIO: f32 = 0.08;
const DESKTOP_FACE_SCORE_THRESHOLD: f32 = 35.0;
const FACE_REGION_MAX_SAMPLES: usize = 4_000;

pub(super) fn run_desktop_cv_assessment_blocking(
    service: &CameraConnectorService,
    request: DesktopCvAssessmentRequest,
    app: Option<AppHandle>,
) -> Result<DesktopCvAssessmentResponse, DesktopError> {
    let limit = request.limit.unwrap_or(1000).clamp(1, 5000);
    let mut offset = 0usize;
    let page_size = 128usize.min(limit);
    let mut assessed_count = 0usize;
    let mut failed_count = 0usize;
    let mut skipped_count = 0usize;
    let mut subject_count = 0usize;
    let scope = if request
        .asset_group_ids
        .as_ref()
        .map(|ids| ids.iter().any(|id| !id.trim().is_empty()))
        .unwrap_or(false)
    {
        "group"
    } else {
        "project"
    };
    let project_settings = service
        .project_evaluation_settings(&request.project_id)
        .map_err(desktop_error)?;
    let subject_policy = technical_policy_for_project_settings(project_settings.as_ref());
    let should_schedule_subjects = service
        .should_schedule_subject_assessment(&request.project_id)
        .unwrap_or(false);

    if let Some(group_ids) = request.asset_group_ids.clone() {
        let requested_ids: Vec<_> = group_ids
            .into_iter()
            .map(|id| id.trim().to_string())
            .filter(|id| !id.is_empty())
            .take(limit)
            .collect();
        let total_count = requested_ids.len().max(1);
        for group_id in requested_ids {
            match assess_desktop_cv_group(
                service,
                &request.project_id,
                &group_id,
                subject_policy,
                should_schedule_subjects,
            ) {
                DesktopCvGroupOutcome::Assessed { subjects } => {
                    assessed_count += 1;
                    subject_count += subjects;
                }
                DesktopCvGroupOutcome::Skipped => skipped_count += 1,
                DesktopCvGroupOutcome::Failed => failed_count += 1,
            }
            emit_desktop_cv_progress(
                app.as_ref(),
                &request.project_id,
                scope,
                DesktopCvProgressCounts {
                    total_count,
                    assessed_count,
                    failed_count,
                    skipped_count,
                    subject_count,
                },
                Some(group_id),
            );
        }
        return Ok(DesktopCvAssessmentResponse {
            assessed_count,
            failed_count,
            skipped_count,
            subject_count,
        });
    }

    while assessed_count + failed_count + skipped_count < limit {
        let page = service
            .project_asset_group_page_with_query(
                &request.project_id,
                AssetGroupQuery::default(),
                offset,
                page_size,
            )
            .map_err(desktop_error)?;
        if page.groups.is_empty() {
            break;
        }
        let total_count = page.total_groups.min(limit).max(1);
        for group in page.groups {
            if assessed_count + failed_count + skipped_count >= limit {
                break;
            }
            let Some(group_id) = group.group_id.as_deref() else {
                skipped_count += 1;
                emit_desktop_cv_progress(
                    app.as_ref(),
                    &request.project_id,
                    scope,
                    DesktopCvProgressCounts {
                        total_count,
                        assessed_count,
                        failed_count,
                        skipped_count,
                        subject_count,
                    },
                    None,
                );
                continue;
            };
            match assess_desktop_cv_group(
                service,
                &request.project_id,
                group_id,
                subject_policy,
                should_schedule_subjects,
            ) {
                DesktopCvGroupOutcome::Assessed { subjects } => {
                    assessed_count += 1;
                    subject_count += subjects;
                }
                DesktopCvGroupOutcome::Skipped => skipped_count += 1,
                DesktopCvGroupOutcome::Failed => failed_count += 1,
            }
            emit_desktop_cv_progress(
                app.as_ref(),
                &request.project_id,
                scope,
                DesktopCvProgressCounts {
                    total_count,
                    assessed_count,
                    failed_count,
                    skipped_count,
                    subject_count,
                },
                Some(group_id.to_string()),
            );
        }
        if !page.has_more {
            break;
        }
        offset = offset.saturating_add(page.limit.max(1));
    }

    Ok(DesktopCvAssessmentResponse {
        assessed_count,
        failed_count,
        skipped_count,
        subject_count,
    })
}

enum DesktopCvGroupOutcome {
    Assessed { subjects: usize },
    Skipped,
    Failed,
}

struct DesktopCvProgressCounts {
    total_count: usize,
    assessed_count: usize,
    failed_count: usize,
    skipped_count: usize,
    subject_count: usize,
}

fn assess_desktop_cv_group(
    service: &CameraConnectorService,
    project_id: &str,
    group_id: &str,
    subject_policy: TechnicalAssessmentPolicy,
    should_schedule_subjects: bool,
) -> DesktopCvGroupOutcome {
    let assets = match service.project_group_assets(project_id, group_id) {
        Ok(assets) => assets,
        Err(_) => return DesktopCvGroupOutcome::Failed,
    };
    let Some(asset) = best_asset_for_cv(&assets) else {
        return DesktopCvGroupOutcome::Skipped;
    };
    let Some(source_path) = asset_local_path(asset) else {
        return DesktopCvGroupOutcome::Skipped;
    };
    let sample = match preview_sample_from_source_path(&source_path, 768) {
        Ok(sample) => sample,
        Err(_) => return DesktopCvGroupOutcome::Failed,
    };
    if service
        .assess_asset_group_preview_with_provider_configured(
            group_id,
            sample,
            "desktop-cv-technical-v1",
            false,
        )
        .is_err()
    {
        return DesktopCvGroupOutcome::Failed;
    }
    let mut subjects = 0usize;
    if should_schedule_subjects {
        match save_desktop_face_assessment(
            service,
            project_id,
            group_id,
            &source_path,
            subject_policy,
        ) {
            Ok(saved) => subjects += saved,
            Err(_) => return DesktopCvGroupOutcome::Failed,
        }
    }
    DesktopCvGroupOutcome::Assessed { subjects }
}

fn emit_desktop_cv_progress(
    app: Option<&AppHandle>,
    project_id: &str,
    scope: &str,
    counts: DesktopCvProgressCounts,
    current_group_id: Option<String>,
) {
    let Some(app) = app else {
        return;
    };
    let _ = app.emit(
        "desktop-cv-assessment-progress",
        DesktopCvAssessmentProgress {
            project_id: project_id.to_string(),
            scope: scope.to_string(),
            total_count: counts.total_count,
            assessed_count: counts.assessed_count,
            failed_count: counts.failed_count,
            skipped_count: counts.skipped_count,
            subject_count: counts.subject_count,
            current_group_id,
        },
    );
}

#[derive(Debug, Clone)]
struct DesktopFaceDetection {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    score: f32,
    analysis: FaceRegionAnalysis,
}

#[derive(Debug, Clone, Copy, Default)]
struct FaceRegionAnalysis {
    area_ratio: f64,
    shadow_ratio: f64,
    highlight_ratio: f64,
    color_cast_strength: f64,
}

fn technical_policy_for_project_settings(
    settings: Option<&ProjectEvaluationSettings>,
) -> TechnicalAssessmentPolicy {
    if let Some(policy) = settings.and_then(|settings| settings.cv_policy_overrides) {
        return policy;
    }
    match settings
        .map(|settings| settings.cv_policy)
        .unwrap_or(CvPolicy::Standard)
    {
        CvPolicy::Loose => TechnicalAssessmentPolicy::loose(),
        CvPolicy::Strict => TechnicalAssessmentPolicy::strict(),
        CvPolicy::Standard => TechnicalAssessmentPolicy::standard(),
    }
}

fn save_desktop_face_assessment(
    service: &CameraConnectorService,
    project_id: &str,
    group_id: &str,
    source_path: &Path,
    policy: TechnicalAssessmentPolicy,
) -> Result<usize, DesktopError> {
    let image = desktop_face_detection_image(source_path)?;
    let faces = detect_desktop_faces(&image, policy)?;
    let primary = faces
        .iter()
        .max_by(|a, b| a.analysis.area_ratio.total_cmp(&b.analysis.area_ratio))
        .map(|face| face.analysis)
        .unwrap_or_default();
    let exposure_risk = primary.shadow_ratio >= policy.face_exposure_warn_ratio
        || primary.highlight_ratio >= policy.face_exposure_warn_ratio;
    let color_cast_risk = primary.color_cast_strength >= policy.face_color_cast_warn_threshold;
    let gate_status = if faces.is_empty() || exposure_risk || color_cast_risk {
        "warn"
    } else {
        "pass"
    };
    let regions: Vec<_> = faces
        .iter()
        .map(|face| {
            json!({
                "kind": "face",
                "x": face.x,
                "y": face.y,
                "width": face.width,
                "height": face.height,
                "area_ratio": face.analysis.area_ratio,
                "score": face.score,
                "tracking_id": serde_json::Value::Null,
                "left_eye_open_probability": serde_json::Value::Null,
                "right_eye_open_probability": serde_json::Value::Null,
            })
        })
        .collect();
    let signals = json!({
        "face_count": faces.len(),
        "image_width": image.width(),
        "image_height": image.height(),
        "largest_face_area_ratio": primary.area_ratio,
        "eyes_open_probability_min": serde_json::Value::Null,
        "closed_eyes": false,
        "face_shadow_ratio": primary.shadow_ratio,
        "face_highlight_ratio": primary.highlight_ratio,
        "face_color_cast_strength": primary.color_cast_strength,
        "face_exposure_risk": exposure_risk,
        "face_color_cast_risk": color_cast_risk,
    });
    let now = current_time_ms();
    let assessment = SubjectAssessment {
        assessment_id: format!("subject:face:{project_id}:{group_id}"),
        project_id: project_id.to_string(),
        asset_group_id: group_id.to_string(),
        subject_type: "face".to_string(),
        detector_kind: "desktop_pico".to_string(),
        detector_version: DESKTOP_FACE_DETECTOR_VERSION.to_string(),
        status: EvaluationRunStatus::Ready,
        gate_status: gate_status.to_string(),
        regions_json: serde_json::to_string(&regions).map_err(|error| {
            thumbnail_error(format!("face regions could not be encoded: {error}"))
        })?,
        signals_json: serde_json::to_string(&signals).map_err(|error| {
            thumbnail_error(format!("face signals could not be encoded: {error}"))
        })?,
        summary: desktop_face_assessment_summary(
            faces.len(),
            exposure_risk,
            color_cast_risk,
            primary,
            policy,
        ),
        created_at_ms: now,
        updated_at_ms: now,
    };
    service
        .save_subject_assessment(assessment)
        .map(|_| 1)
        .map_err(desktop_error)
}

fn desktop_face_assessment_summary(
    face_count: usize,
    exposure_risk: bool,
    color_cast_risk: bool,
    primary: FaceRegionAnalysis,
    policy: TechnicalAssessmentPolicy,
) -> String {
    if face_count == 0 {
        return "未检测到人脸。".to_string();
    }
    if exposure_risk {
        return format!(
            "检测到人脸，面部死黑/死白占比达到 {:.0}% / {:.0}%，超过 {:.0}% 阈值。",
            primary.shadow_ratio * 100.0,
            primary.highlight_ratio * 100.0,
            policy.face_exposure_warn_ratio * 100.0,
        );
    }
    if color_cast_risk {
        return format!(
            "检测到人脸，面部偏色强度 {:.2}，超过 {:.2} 阈值。",
            primary.color_cast_strength, policy.face_color_cast_warn_threshold,
        );
    }
    "检测到人脸，面部曝光和偏色可用。".to_string()
}

fn desktop_face_detection_image(source_path: &Path) -> Result<RgbImage, DesktopError> {
    let image = decode_cv_image(source_path)?;
    let image = if image.width().max(image.height()) > DESKTOP_FACE_DETECTION_MAX_EDGE {
        image.thumbnail(
            DESKTOP_FACE_DETECTION_MAX_EDGE,
            DESKTOP_FACE_DETECTION_MAX_EDGE,
        )
    } else {
        image
    };
    Ok(image.to_rgb8())
}

fn detect_desktop_faces(
    image: &RgbImage,
    policy: TechnicalAssessmentPolicy,
) -> Result<Vec<DesktopFaceDetection>, DesktopError> {
    let width = image.width();
    let height = image.height();
    let min_dim = width.min(height);
    if min_dim < 40 {
        return Ok(Vec::new());
    }
    let detector = desktop_face_detector()?;
    let gray = rgb_to_luma_image(image);
    let min_size = ((min_dim as f32) * DESKTOP_FACE_MIN_SIZE_RATIO)
        .round()
        .clamp(24.0, min_dim as f32) as u32;
    let max_size = ((min_dim as f32) * 0.85).round().max(min_size as f32) as u32;
    let runner = DetectMultiscale::builder()
        .multiscaler(
            Multiscaler::new(min_size, max_size, 0.1, 1.1).map_err(|error| {
                thumbnail_error(format!("face multiscale setup failed: {error}"))
            })?,
        )
        .clusterizer(Clusterizer::default().score_threshold(DESKTOP_FACE_SCORE_THRESHOLD))
        .build()
        .map_err(|error| thumbnail_error(format!("face detector setup failed: {error}")))?;
    let mut faces: Vec<_> = runner
        .run(detector, &gray)
        .into_iter()
        .filter_map(|detection| {
            let region = detection.region();
            let (x, y, width, height) = clamp_face_bounds(
                region.left(),
                region.top(),
                region.width(),
                region.height(),
                image,
            )?;
            Some(DesktopFaceDetection {
                x,
                y,
                width,
                height,
                score: detection.score(),
                analysis: analyze_face_region(image, x, y, width, height, policy),
            })
        })
        .collect();
    faces.sort_by(|a, b| b.score.total_cmp(&a.score));
    faces.truncate(20);
    Ok(faces)
}

fn desktop_face_detector() -> Result<&'static Detector, DesktopError> {
    static DETECTOR: OnceLock<Result<Detector, String>> = OnceLock::new();
    match DETECTOR.get_or_init(|| {
        Detector::load(Cursor::new(DESKTOP_FACE_DETECTOR_MODEL))
            .map_err(|error| format!("face detector model could not be loaded: {error}"))
    }) {
        Ok(detector) => Ok(detector),
        Err(message) => Err(thumbnail_error(message.clone())),
    }
}

fn rgb_to_luma_image(image: &RgbImage) -> GrayImage {
    let mut luma = GrayImage::new(image.width(), image.height());
    for (x, y, pixel) in image.enumerate_pixels() {
        let [r, g, b] = pixel.0;
        let value = (0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b))
            .round()
            .clamp(0.0, 255.0) as u8;
        luma.put_pixel(x, y, image::Luma([value]));
    }
    luma
}

fn clamp_face_bounds(
    left: i32,
    top: i32,
    width: u32,
    height: u32,
    image: &RgbImage,
) -> Option<(u32, u32, u32, u32)> {
    let image_width = image.width() as i32;
    let image_height = image.height() as i32;
    let right = left.saturating_add(width as i32).clamp(0, image_width);
    let bottom = top.saturating_add(height as i32).clamp(0, image_height);
    let left = left.clamp(0, image_width);
    let top = top.clamp(0, image_height);
    if right <= left || bottom <= top {
        return None;
    }
    Some((
        left as u32,
        top as u32,
        (right - left) as u32,
        (bottom - top) as u32,
    ))
}

fn analyze_face_region(
    image: &RgbImage,
    left: u32,
    top: u32,
    width: u32,
    height: u32,
    policy: TechnicalAssessmentPolicy,
) -> FaceRegionAnalysis {
    if width == 0 || height == 0 || image.width() == 0 || image.height() == 0 {
        return FaceRegionAnalysis::default();
    }
    let stride = face_region_sample_stride(width, height);
    let mut samples = 0usize;
    let mut shadow_pixels = 0usize;
    let mut highlight_pixels = 0usize;
    let mut red_sum = 0.0;
    let mut green_sum = 0.0;
    let mut blue_sum = 0.0;
    let right = left.saturating_add(width).min(image.width());
    let bottom = top.saturating_add(height).min(image.height());
    for y in (top..bottom).step_by(stride) {
        for x in (left..right).step_by(stride) {
            let [r, g, b] = image.get_pixel(x, y).0;
            let luma = 0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b);
            if luma <= f64::from(policy.shadow_clip_threshold) {
                shadow_pixels += 1;
            }
            if luma >= f64::from(policy.highlight_clip_threshold) {
                highlight_pixels += 1;
            }
            red_sum += f64::from(r);
            green_sum += f64::from(g);
            blue_sum += f64::from(b);
            samples += 1;
        }
    }
    let safe_samples = samples.max(1) as f64;
    let red_mean = red_sum / safe_samples;
    let green_mean = green_sum / safe_samples;
    let blue_mean = blue_sum / safe_samples;
    let mean = (red_mean + green_mean + blue_mean) / 3.0;
    let color_cast_strength = if mean <= 1.0 {
        0.0
    } else {
        (red_mean - mean)
            .abs()
            .max((green_mean - mean).abs())
            .max((blue_mean - mean).abs())
            / mean
    };
    let image_area = f64::from(image.width()) * f64::from(image.height());
    FaceRegionAnalysis {
        area_ratio: f64::from(width) * f64::from(height) / image_area.max(1.0),
        shadow_ratio: shadow_pixels as f64 / safe_samples,
        highlight_ratio: highlight_pixels as f64 / safe_samples,
        color_cast_strength,
    }
}

fn face_region_sample_stride(width: u32, height: u32) -> usize {
    let area = f64::from(width.max(1)) * f64::from(height.max(1));
    (area / FACE_REGION_MAX_SAMPLES as f64)
        .sqrt()
        .round()
        .max(1.0) as usize
}

fn best_asset_for_cv(assets: &[StoredAsset]) -> Option<&StoredAsset> {
    assets
        .iter()
        .filter(|asset| is_photo_asset_for_cv(asset))
        .filter(|asset| asset_local_path(asset).is_some())
        .min_by_key(|asset| cv_asset_rank(asset))
}

fn is_photo_asset_for_cv(asset: &StoredAsset) -> bool {
    asset.format.is_photo()
}

fn cv_asset_rank(asset: &StoredAsset) -> u8 {
    match asset.group_role.as_str() {
        "jpeg" => 0,
        "raw" => 1,
        _ if asset.format.role().as_str() == "jpeg" => 0,
        _ if asset.format.is_raw() => 1,
        _ => 2,
    }
}

fn asset_local_path(asset: &StoredAsset) -> Option<PathBuf> {
    asset
        .final_location
        .as_ref()
        .and_then(|location| location.as_local_path())
        .map(PathBuf::from)
        .or_else(|| {
            let original_path = PathBuf::from(&asset.original_path);
            original_path.is_absolute().then_some(original_path)
        })
        .filter(|path| path.is_file())
}

fn preview_sample_from_source_path(
    source_path: &Path,
    max_edge: u32,
) -> Result<PreviewSample, DesktopError> {
    let mut image = decode_cv_image(source_path)?;
    Ok(preview_sample_from_image(
        &mut image,
        Some(source_path.to_string_lossy().into_owned()),
        max_edge,
    ))
}

fn decode_cv_image(source_path: &Path) -> Result<DynamicImage, DesktopError> {
    if is_raw_extension(source_path) {
        if let Ok(image) = raw_thumbnail_image_from_file(source_path) {
            return Ok(image);
        }
        if let Some(image) = embedded_jpeg_preview_image(source_path) {
            return Ok(image);
        }
    }

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

fn preview_sample_from_image(
    image: &mut DynamicImage,
    preview_source: Option<String>,
    max_edge: u32,
) -> PreviewSample {
    let source_width = image.width();
    let source_height = image.height();
    let thumbnail = if source_width.max(source_height) > max_edge {
        image.thumbnail(max_edge, max_edge)
    } else {
        image.clone()
    }
    .to_rgb8();
    let width = thumbnail.width() as usize;
    let height = thumbnail.height() as usize;
    let capacity = width.saturating_mul(height);
    let mut luma = Vec::with_capacity(capacity);
    let mut red = Vec::with_capacity(capacity);
    let mut green = Vec::with_capacity(capacity);
    let mut blue = Vec::with_capacity(capacity);
    for pixel in thumbnail.pixels() {
        let [r, g, b] = pixel.0;
        red.push(r);
        green.push(g);
        blue.push(b);
        let y = (0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b))
            .round()
            .clamp(0.0, 255.0) as u8;
        luma.push(y);
    }
    PreviewSample {
        width,
        height,
        luma,
        red: Some(red),
        green: Some(green),
        blue: Some(blue),
        preview_source,
    }
}

#[cfg(test)]
#[path = "desktop_cv/tests.rs"]
mod tests;
