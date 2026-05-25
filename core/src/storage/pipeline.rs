use std::fs::{self, File};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};

use crate::{ImporterError, ReceiveProgress, Result};

#[derive(Debug, Clone)]
pub struct LocalStagingStore {
    staging_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct LocalFolderObjectStore {
    output_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedObject {
    pub transfer_id: String,
    pub final_filename: String,
    pub staged_path: PathBuf,
    pub bytes_written: u64,
}

pub struct LocalStagedUpload {
    transfer_id: String,
    final_filename: String,
    staged_path: PathBuf,
    file: File,
    bytes_written: u64,
}

impl LocalStagingStore {
    pub fn new(staging_dir: impl Into<PathBuf>) -> Self {
        Self {
            staging_dir: staging_dir.into(),
        }
    }

    pub fn begin_write(
        &self,
        transfer_id: impl Into<String>,
        relative_path: &str,
    ) -> Result<LocalStagedUpload> {
        let transfer_id = transfer_id.into();
        let final_filename = safe_relative_filename(relative_path)?;
        fs::create_dir_all(&self.staging_dir)?;
        let staged_path = self.staging_dir.join(format!(
            "{}-{final_filename}.staged",
            stable_key(&transfer_id)
        ));
        if staged_path.exists() {
            fs::remove_file(&staged_path)?;
        }
        let file = File::create(&staged_path)?;
        Ok(LocalStagedUpload {
            transfer_id,
            final_filename,
            staged_path,
            file,
            bytes_written: 0,
        })
    }

    pub fn cleanup_stale(&self) -> Result<usize> {
        if !self.staging_dir.exists() {
            return Ok(0);
        }
        let mut removed = 0;
        for entry in fs::read_dir(&self.staging_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) == Some("staged") {
                fs::remove_file(path)?;
                removed += 1;
            }
        }
        Ok(removed)
    }
}

impl LocalStagedUpload {
    pub fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<()> {
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(bytes)?;
        self.bytes_written = self.bytes_written.max(offset + bytes.len() as u64);
        Ok(())
    }

    pub fn finish(mut self) -> Result<StagedObject> {
        self.file.flush()?;
        let Self {
            transfer_id,
            final_filename,
            staged_path,
            file,
            bytes_written,
        } = self;
        drop(file);
        Ok(StagedObject {
            transfer_id,
            final_filename,
            staged_path,
            bytes_written,
        })
    }
}

impl Write for LocalStagedUpload {
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

impl LocalFolderObjectStore {
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        Self {
            output_dir: output_dir.into(),
        }
    }

    pub fn publish(&self, staged: StagedObject) -> Result<ReceiveProgress> {
        fs::create_dir_all(&self.output_dir)?;
        let final_path = available_path(self.output_dir.join(&staged.final_filename));
        let temp_path = temp_path_for(&final_path);
        if temp_path.exists() {
            fs::remove_file(&temp_path)?;
        }
        fs::copy(&staged.staged_path, &temp_path)?;
        fs::rename(&temp_path, &final_path)?;
        fs::remove_file(&staged.staged_path)?;
        let filename = final_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or(&staged.final_filename)
            .to_string();
        Ok(ReceiveProgress::completed(
            staged.transfer_id,
            filename,
            staged.bytes_written,
            final_path,
        ))
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

fn safe_relative_filename(path: &str) -> Result<String> {
    let normalized = path.replace('\\', "/");
    let Some(filename) = normalized
        .split('/')
        .rev()
        .find(|part| !part.is_empty() && *part != "." && *part != "..")
    else {
        return Err(ImporterError::InvalidUploadPath);
    };

    let safe = safe_filename(filename);
    let safe_path = PathBuf::from(&safe);
    if safe.is_empty()
        || safe_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        Err(ImporterError::InvalidUploadPath)
    } else {
        Ok(safe)
    }
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

fn stable_key(value: &str) -> String {
    let mut hash = 1469598103934665603_u64;
    for byte in value.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{hash:016x}")
}
