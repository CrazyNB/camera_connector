use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::{ImporterError, ReceiveProgress, Result};

#[derive(Debug, Clone)]
pub struct LocalFileSink {
    output_dir: PathBuf,
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

        fs::write(&temp_path, bytes)?;
        fs::rename(&temp_path, &final_path)?;

        Ok(ReceiveProgress::completed(
            transfer_id,
            relative_display_path(&self.output_dir, &final_path),
            bytes.len() as u64,
            final_path,
        ))
    }

    pub fn create_dir_all(&self, relative_path: &str) -> Result<PathBuf> {
        let safe_path = safe_relative_path(relative_path)?;
        let final_path = self.output_dir.join(safe_path);
        fs::create_dir_all(&final_path)?;
        Ok(final_path)
    }
}

fn available_path(path: PathBuf) -> PathBuf {
    if !path.exists() {
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
        if !candidate.exists() {
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
    let mut safe = PathBuf::new();

    for part in normalized.split('/') {
        if part.is_empty() || part == "." || part == ".." {
            continue;
        }
        let sanitized = safe_filename(part);
        if !sanitized.is_empty() {
            safe.push(sanitized);
        }
    }

    if safe.as_os_str().is_empty()
        || safe
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(ImporterError::InvalidUploadPath);
    }

    Ok(safe)
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
