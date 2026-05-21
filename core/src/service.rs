use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::{
    read_connected_devices, read_receiver_runtime_status, read_transfer_log, scan_inbox_groups,
    CameraConnectorConfig, ConnectedDevice, ImportSource, PushProtocol, PushReceiverConfig,
    ReceivedAssetGroup, ReceiverAccountConfig, ReceiverRuntimeStatus, Result, TransferRecord,
};

#[derive(Debug, Clone)]
pub struct CameraConnectorService {
    config_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ReceiverConfigRequest {
    pub protocol: PushProtocol,
    pub bind_host: String,
    pub port: u16,
    pub output_dir: PathBuf,
    pub username: Option<String>,
    pub password: Option<String>,
    pub advertised_host: Option<String>,
    pub source_name: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TransferQuery {
    pub transfer_id: Option<String>,
    pub original_path: Option<String>,
    pub final_filename: Option<String>,
    pub source_name: Option<String>,
    pub remote_addr: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferRecordView {
    pub record: TransferRecord,
    pub display_source: Option<String>,
    pub virtual_display_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectedDeviceView {
    pub device: ConnectedDevice,
    pub display_source: String,
}

impl CameraConnectorService {
    pub fn new(config_path: Option<PathBuf>) -> Self {
        Self { config_path }
    }

    pub fn config_path(&self) -> PathBuf {
        CameraConnectorConfig::resolved_path(self.config_path.as_deref())
    }

    pub fn load_config(&self) -> Result<CameraConnectorConfig> {
        CameraConnectorConfig::load(self.config_path.as_deref())
    }

    pub fn save_config(&self, config: &CameraConnectorConfig) -> Result<PathBuf> {
        config.save(self.config_path.as_deref())
    }

    pub fn receiver_config(&self, request: ReceiverConfigRequest) -> Result<PushReceiverConfig> {
        let mut config = PushReceiverConfig::new(
            request.protocol,
            request.bind_host,
            request.port,
            request.output_dir,
        );
        config.advertised_host = request.advertised_host;
        config.source_name = request.source_name;
        config.accounts = self.load_config()?.effective_accounts(
            request.username.as_deref(),
            request.password.as_deref(),
            config.source_name.as_deref(),
        )?;
        Ok(config)
    }

    pub fn set_account(
        &self,
        username: impl Into<String>,
        password: Option<&str>,
        device_name: impl Into<String>,
    ) -> Result<(ReceiverAccountConfig, PathBuf)> {
        let mut config = self.load_config()?;
        let account = config.set_account(username, password, device_name)?.clone();
        let path = self.save_config(&config)?;
        Ok((account, path))
    }

    pub fn remove_account(&self, username: &str) -> Result<(bool, PathBuf)> {
        let mut config = self.load_config()?;
        let removed = config.remove_account(username).is_some();
        let path = self.save_config(&config)?;
        Ok((removed, path))
    }

    pub fn inbox_groups(
        &self,
        output_dir: impl AsRef<Path>,
        source: ImportSource,
    ) -> Result<Vec<ReceivedAssetGroup>> {
        scan_inbox_groups(output_dir, source)
    }

    pub fn receiver_status(
        &self,
        output_dir: impl AsRef<Path>,
    ) -> Result<Option<ReceiverRuntimeStatus>> {
        read_receiver_runtime_status(output_dir)
    }

    pub fn transfers(
        &self,
        output_dir: impl AsRef<Path>,
        query: TransferQuery,
    ) -> Result<Vec<TransferRecordView>> {
        let views = read_transfer_log(output_dir)?
            .into_iter()
            .filter(|record| transfer_matches(record, &query))
            .map(|record| {
                let display_source = record_display_source(&record);
                let virtual_display_path = record.virtual_display_path(display_source.as_deref());
                TransferRecordView {
                    record,
                    display_source,
                    virtual_display_path,
                }
            })
            .collect::<Vec<_>>();
        Ok(views)
    }

    pub fn connected_devices(
        &self,
        output_dir: impl AsRef<Path>,
        username: Option<&str>,
        online: bool,
    ) -> Result<Vec<ConnectedDeviceView>> {
        let accounts = self.load_config()?.accounts;
        let views = read_connected_devices(output_dir)?
            .into_iter()
            .filter(|device| device_matches(device, username, online))
            .map(|device| {
                let display_source = device_display_source(&device, &accounts)
                    .unwrap_or_else(|| remote_addr_display_label(&device.remote_addr));
                ConnectedDeviceView {
                    device,
                    display_source,
                }
            })
            .collect::<Vec<_>>();
        Ok(views)
    }
}

fn transfer_matches(record: &TransferRecord, query: &TransferQuery) -> bool {
    query
        .source_name
        .as_ref()
        .map(|expected| record_display_source(record).as_ref() == Some(expected))
        .unwrap_or(true)
        && query
            .remote_addr
            .as_ref()
            .map(|expected| record.remote_addr.as_ref() == Some(expected))
            .unwrap_or(true)
        && query
            .transfer_id
            .as_ref()
            .map(|expected| record.transfer_id.contains(expected))
            .unwrap_or(true)
        && query
            .original_path
            .as_ref()
            .map(|expected| {
                record
                    .original_path
                    .to_ascii_lowercase()
                    .contains(&expected.to_ascii_lowercase())
            })
            .unwrap_or(true)
        && query
            .final_filename
            .as_ref()
            .map(|expected| {
                record
                    .final_filename
                    .to_ascii_lowercase()
                    .contains(&expected.to_ascii_lowercase())
            })
            .unwrap_or(true)
}

fn record_display_source(record: &TransferRecord) -> Option<String> {
    record.source_name.clone()
}

fn device_matches(device: &ConnectedDevice, username: Option<&str>, online: bool) -> bool {
    (!online || device.online)
        && username
            .map(|expected| device.username.as_deref() == Some(expected))
            .unwrap_or(true)
}

fn device_display_source(
    device: &ConnectedDevice,
    accounts: &BTreeMap<String, crate::ReceiverAccountConfig>,
) -> Option<String> {
    device
        .username
        .as_deref()
        .and_then(|username| accounts.get(username))
        .map(|account| account.device_name.clone())
        .or_else(|| device.source_name.clone())
}

fn remote_addr_display_label(remote_addr: &str) -> String {
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
