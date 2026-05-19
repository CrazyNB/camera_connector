use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadState {
    Waiting,
    Downloading,
    Completed,
    Failed,
    Canceled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadProgress {
    pub handle: u32,
    pub filename: String,
    pub bytes_written: u64,
    pub total_bytes: Option<u64>,
    pub state: DownloadState,
    pub output_path: Option<PathBuf>,
}

impl DownloadProgress {
    pub fn completed(
        handle: u32,
        filename: impl Into<String>,
        bytes_written: u64,
        output_path: PathBuf,
    ) -> Self {
        Self {
            handle,
            filename: filename.into(),
            bytes_written,
            total_bytes: Some(bytes_written),
            state: DownloadState::Completed,
            output_path: Some(output_path),
        }
    }
}
