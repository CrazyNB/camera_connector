use std::fs::{self, File};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};

use crate::{ImporterError, ReceiveProgress, Result};

pub trait ReceiveStorage {
    type Upload: ReceiveUpload;

    fn begin_write(
        &self,
        transfer_id: impl Into<String>,
        relative_path: &str,
    ) -> Result<Self::Upload>;

    fn create_dir_all(&self, relative_path: &str) -> Result<()>;
}

pub trait ReceiveUpload: Write {
    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<()>;
    fn finish(self) -> Result<ReceiveProgress>;
}

#[derive(Debug, Clone)]
pub struct LocalFileSink {
    output_dir: PathBuf,
}

impl ReceiveStorage for LocalFileSink {
    type Upload = LocalFileUpload;

    fn begin_write(
        &self,
        transfer_id: impl Into<String>,
        relative_path: &str,
    ) -> Result<Self::Upload> {
        LocalFileSink::begin_write(self, transfer_id, relative_path)
    }

    fn create_dir_all(&self, relative_path: &str) -> Result<()> {
        LocalFileSink::create_dir_all(self, relative_path).map(|_| ())
    }
}

impl LocalFileSink {
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        Self {
            output_dir: output_dir.into(),
        }
    }

    pub fn write_complete(
        &self,
        transfer_id: impl Into<String>,
        relative_path: &str,
        bytes: &[u8],
    ) -> Result<ReceiveProgress> {
        let mut upload = self.begin_write(transfer_id, relative_path)?;
        upload.write_all(bytes)?;
        upload.finish()
    }

    pub fn begin_write(
        &self,
        transfer_id: impl Into<String>,
        relative_path: &str,
    ) -> Result<LocalFileUpload> {
        let safe_path = safe_relative_path(relative_path)?;
        let final_path = available_path(self.output_dir.join(&safe_path));
        let parent = final_path
            .parent()
            .ok_or(ImporterError::InvalidUploadPath)?;
        fs::create_dir_all(parent)?;

        let temp_path = temp_path_for(&final_path);
        if temp_path.exists() {
            fs::remove_file(&temp_path)?;
        }

        let file = File::create(&temp_path)?;

        Ok(LocalFileUpload {
            transfer_id: transfer_id.into(),
            output_dir: self.output_dir.clone(),
            final_path,
            temp_path,
            file,
            bytes_written: 0,
        })
    }

    pub fn create_dir_all(&self, relative_path: &str) -> Result<PathBuf> {
        let _ = safe_relative_path(relative_path)?;
        fs::create_dir_all(&self.output_dir)?;
        Ok(self.output_dir.clone())
    }
}

pub struct LocalFileUpload {
    transfer_id: String,
    output_dir: PathBuf,
    final_path: PathBuf,
    temp_path: PathBuf,
    file: File,
    bytes_written: u64,
}

impl LocalFileUpload {
    pub fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<()> {
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(bytes)?;
        self.bytes_written = self.bytes_written.max(offset + bytes.len() as u64);
        Ok(())
    }

    pub fn finish(mut self) -> Result<ReceiveProgress> {
        self.file.flush()?;
        let LocalFileUpload {
            transfer_id,
            output_dir,
            final_path,
            temp_path,
            file,
            bytes_written,
        } = self;
        drop(file);
        fs::rename(&temp_path, &final_path)?;

        Ok(ReceiveProgress::completed(
            transfer_id,
            relative_display_path(&output_dir, &final_path),
            bytes_written,
            final_path,
        ))
    }
}

impl ReceiveUpload for LocalFileUpload {
    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<()> {
        LocalFileUpload::write_at(self, offset, bytes)
    }

    fn finish(self) -> Result<ReceiveProgress> {
        LocalFileUpload::finish(self)
    }
}

impl Write for LocalFileUpload {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let offset = self.file.stream_position()?;
        let written = self.file.write(buf)?;
        self.bytes_written = self.bytes_written.max(offset + written as u64);
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

fn available_path(path: PathBuf) -> PathBuf {
    if !path.exists() && !temp_path_for(&path).exists() {
        return path;
    }

    let parent = path.parent().map(Path::to_path_buf).unwrap_or_default();
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("upload");
    let extension = path.extension().and_then(|value| value.to_str());

    for index in 1.. {
        let filename = match extension {
            Some(extension) => format!("{stem} ({index}).{extension}"),
            None => format!("{stem} ({index})"),
        };
        let candidate = parent.join(filename);
        if !candidate.exists() && !temp_path_for(&candidate).exists() {
            return candidate;
        }
    }

    unreachable!("unbounded duplicate filename search should always return");
}

fn temp_path_for(final_path: &Path) -> PathBuf {
    let mut temp_name = final_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("upload")
        .to_string();
    temp_name.push_str(".tmp");
    final_path.with_file_name(temp_name)
}

fn safe_relative_path(path: &str) -> Result<PathBuf> {
    let normalized = path.replace('\\', "/");
    let Some(filename) = normalized
        .split('/')
        .rev()
        .find(|part| !part.is_empty() && *part != "." && *part != "..")
    else {
        return Err(ImporterError::InvalidUploadPath);
    };

    let safe = PathBuf::from(safe_filename(filename));

    if safe.as_os_str().is_empty()
        || safe
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        Err(ImporterError::InvalidUploadPath)
    } else {
        Ok(safe)
    }
}

fn relative_display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str().map(ToOwned::to_owned),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn safe_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|ch| match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => ch,
        })
        .collect()
}
