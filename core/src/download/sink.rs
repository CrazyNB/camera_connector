use std::fs;
use std::path::{Path, PathBuf};

use crate::{DownloadProgress, Result};

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
        handle: u32,
        filename: &str,
        bytes: &[u8],
    ) -> Result<DownloadProgress> {
        fs::create_dir_all(&self.output_dir)?;
        let final_path = self.output_dir.join(safe_filename(filename));
        let temp_path = temp_path_for(&final_path);

        if temp_path.exists() {
            fs::remove_file(&temp_path)?;
        }

        fs::write(&temp_path, bytes)?;
        fs::rename(&temp_path, &final_path)?;

        Ok(DownloadProgress::completed(
            handle,
            filename,
            bytes.len() as u64,
            final_path,
        ))
    }
}

fn temp_path_for(final_path: &Path) -> PathBuf {
    let mut temp_name = final_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("download")
        .to_string();
    temp_name.push_str(".tmp");
    final_path.with_file_name(temp_name)
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
