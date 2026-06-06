use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{Result, SqliteStore};

pub(crate) const CONNECTED_DEVICES_FILENAME: &str = "connected-devices.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectedDevice {
    pub remote_addr: String,
    pub source_name: Option<String>,
    pub username: Option<String>,
    pub first_seen_at_ms: i64,
    pub last_seen_at_ms: i64,
    pub last_disconnected_at_ms: Option<i64>,
    pub last_remote_port: Option<u16>,
    pub active_connections: u32,
    pub online: bool,
}

pub fn connected_devices_path(output_dir: impl AsRef<Path>) -> PathBuf {
    output_dir.as_ref().join(CONNECTED_DEVICES_FILENAME)
}

pub fn record_device_connected(
    output_dir: impl AsRef<Path>,
    remote_addr: impl AsRef<str>,
    remote_port: Option<u16>,
    source_name: Option<&str>,
    username: Option<&str>,
) -> Result<()> {
    SqliteStore::open_state_dir(output_dir.as_ref())?.record_connected_device(
        remote_addr.as_ref(),
        remote_port,
        source_name,
        username,
    )
}

pub fn record_device_disconnected(
    output_dir: impl AsRef<Path>,
    remote_addr: impl AsRef<str>,
) -> Result<()> {
    SqliteStore::open_state_dir(output_dir.as_ref())?
        .record_disconnected_device(remote_addr.as_ref())
}

pub fn mark_all_connected_devices_offline(output_dir: impl AsRef<Path>) -> Result<()> {
    SqliteStore::open_state_dir(output_dir.as_ref())?.mark_all_connected_devices_offline()
}

pub fn record_device_authenticated(
    output_dir: impl AsRef<Path>,
    remote_addr: impl AsRef<str>,
    source_name: Option<&str>,
    username: Option<&str>,
) -> Result<()> {
    SqliteStore::open_state_dir(output_dir.as_ref())?.record_authenticated_device(
        remote_addr.as_ref(),
        source_name,
        username,
    )
}

pub fn read_connected_devices(output_dir: impl AsRef<Path>) -> Result<Vec<ConnectedDevice>> {
    SqliteStore::open_state_dir(output_dir.as_ref())?.connected_devices()
}
