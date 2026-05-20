use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;

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
    let output_dir = output_dir.as_ref();
    let remote_addr = remote_addr.as_ref();
    let now = current_time_ms();
    let mut devices = read_connected_devices(output_dir)?;

    if let Some(device) = devices
        .iter_mut()
        .find(|device| device.remote_addr == remote_addr)
    {
        device.last_seen_at_ms = now;
        device.last_remote_port = remote_port;
        device.active_connections = device.active_connections.saturating_add(1);
        device.online = true;
        if let Some(source_name) = source_name {
            device.source_name = Some(source_name.to_string());
        }
        if let Some(username) = username {
            device.username = Some(username.to_string());
        }
    } else {
        devices.push(ConnectedDevice {
            remote_addr: remote_addr.to_string(),
            source_name: source_name.map(ToOwned::to_owned),
            username: username.map(ToOwned::to_owned),
            first_seen_at_ms: now,
            last_seen_at_ms: now,
            last_disconnected_at_ms: None,
            last_remote_port: remote_port,
            active_connections: 1,
            online: true,
        });
    }

    write_connected_devices(output_dir, &devices)
}

pub fn record_device_disconnected(
    output_dir: impl AsRef<Path>,
    remote_addr: impl AsRef<str>,
) -> Result<()> {
    let output_dir = output_dir.as_ref();
    let remote_addr = remote_addr.as_ref();
    let now = current_time_ms();
    let mut devices = read_connected_devices(output_dir)?;

    if let Some(device) = devices
        .iter_mut()
        .find(|device| device.remote_addr == remote_addr)
    {
        device.active_connections = device.active_connections.saturating_sub(1);
        device.online = device.active_connections > 0;
        device.last_seen_at_ms = now;
        if !device.online {
            device.last_disconnected_at_ms = Some(now);
        }
    }

    write_connected_devices(output_dir, &devices)
}

pub fn record_device_authenticated(
    output_dir: impl AsRef<Path>,
    remote_addr: impl AsRef<str>,
    source_name: Option<&str>,
    username: Option<&str>,
) -> Result<()> {
    let output_dir = output_dir.as_ref();
    let remote_addr = remote_addr.as_ref();
    let now = current_time_ms();
    let mut devices = read_connected_devices(output_dir)?;

    if let Some(device) = devices
        .iter_mut()
        .find(|device| device.remote_addr == remote_addr)
    {
        device.last_seen_at_ms = now;
        if let Some(source_name) = source_name {
            device.source_name = Some(source_name.to_string());
        }
        if let Some(username) = username {
            device.username = Some(username.to_string());
        }
    }

    write_connected_devices(output_dir, &devices)
}

pub fn read_connected_devices(output_dir: impl AsRef<Path>) -> Result<Vec<ConnectedDevice>> {
    let path = connected_devices_path(output_dir);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let bytes = fs::read(path)?;
    let mut devices: Vec<ConnectedDevice> = serde_json::from_slice(&bytes)
        .map_err(|error| crate::ImporterError::internal(error.to_string()))?;
    devices.sort_by(|left, right| {
        right
            .online
            .cmp(&left.online)
            .then_with(|| right.last_seen_at_ms.cmp(&left.last_seen_at_ms))
            .then_with(|| left.remote_addr.cmp(&right.remote_addr))
    });
    Ok(devices)
}

fn write_connected_devices(output_dir: &Path, devices: &[ConnectedDevice]) -> Result<()> {
    fs::create_dir_all(output_dir)?;
    let path = connected_devices_path(output_dir);
    let temp_path = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(devices)
        .map_err(|error| crate::ImporterError::internal(error.to_string()))?;
    fs::write(&temp_path, bytes)?;
    fs::rename(temp_path, path)?;
    Ok(())
}

fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}
