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

impl TransferRecord {
    pub fn virtual_display_path(&self, source_override: Option<&str>) -> String {
        let mut parts = Vec::new();

        if let Some(source) = source_override
            .and_then(display_segment)
            .or_else(|| self.source_name.as_deref().and_then(display_segment))
            .or_else(|| self.remote_addr.as_deref().map(remote_addr_label))
        {
            parts.push(source);
        }

        let mut original_parts = self
            .original_path
            .replace('\\', "/")
            .split('/')
            .filter_map(display_segment)
            .collect::<Vec<_>>();
        if !original_parts.is_empty() {
            original_parts.pop();
        }

        parts.extend(original_parts);
        parts.push(
            display_segment(&self.final_filename).unwrap_or_else(|| self.final_filename.clone()),
        );
        parts.join("/")
    }
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

fn display_segment(segment: &str) -> Option<String> {
    let normalized = segment.trim().replace(['/', '\\'], "_");
    if normalized.is_empty() || normalized == "." || normalized == ".." {
        None
    } else {
        Some(normalized)
    }
}

fn remote_addr_label(remote_addr: &str) -> String {
    if let Some(last_octet) = remote_addr
        .rsplit('.')
        .next()
        .filter(|value| !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit()))
        .and_then(|value| value.parse::<u8>().ok())
    {
        return format!("IP-{last_octet:03}");
    }

    let digits = remote_addr
        .chars()
        .filter(char::is_ascii_digit)
        .collect::<String>();
    if digits.is_empty() {
        "IP".to_string()
    } else {
        let start = digits.len().saturating_sub(3);
        format!("IP-{:0>3}", &digits[start..])
    }
}
