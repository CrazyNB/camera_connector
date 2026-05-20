use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;

pub(crate) const TRANSFER_LOG_FILENAME: &str = "transfer-log.jsonl";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferStatus {
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferRecord {
    pub transfer_id: String,
    pub protocol: String,
    pub status: TransferStatus,
    pub original_path: String,
    pub final_filename: String,
    pub final_path: PathBuf,
    pub size_bytes: u64,
    pub remote_addr: Option<String>,
    pub source_name: Option<String>,
    pub started_at_ms: i64,
    pub completed_at_ms: Option<i64>,
    pub error: Option<String>,
}

pub fn transfer_log_path(output_dir: impl AsRef<Path>) -> PathBuf {
    output_dir.as_ref().join(TRANSFER_LOG_FILENAME)
}

pub fn append_transfer_record(output_dir: impl AsRef<Path>, record: &TransferRecord) -> Result<()> {
    fs::create_dir_all(output_dir.as_ref())?;
    let path = transfer_log_path(output_dir);
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(record)
        .map_err(|error| crate::ImporterError::internal(error.to_string()))?;
    writeln!(file, "{line}")?;
    Ok(())
}

pub fn read_transfer_log(output_dir: impl AsRef<Path>) -> Result<Vec<TransferRecord>> {
    let path = transfer_log_path(output_dir);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record = serde_json::from_str(&line)
            .map_err(|error| crate::ImporterError::internal(error.to_string()))?;
        records.push(record);
    }

    Ok(records)
}
