use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiveState {
    Waiting,
    Receiving,
    Completed,
    Failed,
    Canceled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiveProgress {
    pub transfer_id: String,
    pub filename: String,
    pub bytes_written: u64,
    pub total_bytes: Option<u64>,
    pub state: ReceiveState,
    pub output_path: Option<PathBuf>,
}

impl ReceiveProgress {
    pub fn completed(
        transfer_id: impl Into<String>,
        filename: impl Into<String>,
        bytes_written: u64,
        output_path: PathBuf,
    ) -> Self {
        Self {
            transfer_id: transfer_id.into(),
            filename: filename.into(),
            bytes_written,
            total_bytes: Some(bytes_written),
            state: ReceiveState::Completed,
            output_path: Some(output_path),
        }
    }
}
