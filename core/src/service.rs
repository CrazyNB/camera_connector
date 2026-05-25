use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    group_received_assets, read_connected_devices, read_receiver_runtime_status, read_transfer_log,
    scan_inbox_groups, CameraConnectorConfig, ConnectedDevice, ImportSource, ObjectFormat,
    PushProtocol, PushReceiverConfig, ReceivedAsset, ReceivedAssetGroup, ReceiverAccountConfig,
    ReceiverRuntimeStatus, ReceiverSettingsConfig, Result, SqliteStore, TransferRecord,
    TransferStatus,
};

#[derive(Debug, Clone)]
pub struct CameraConnectorService {
    config_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ReceiverConfigRequest {
    pub protocol: Option<PushProtocol>,
    pub bind_host: Option<String>,
    pub port: Option<u16>,
    pub output_dir: Option<PathBuf>,
    pub state_dir: Option<PathBuf>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub advertised_host: Option<String>,
    pub source_name: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ReceiverSettingsUpdate {
    pub protocol: Option<PushProtocol>,
    pub bind_host: Option<String>,
    pub ftp_port: Option<u16>,
    pub sftp_port: Option<u16>,
    pub output_dir: Option<PathBuf>,
    pub state_dir: Option<PathBuf>,
    pub advertised_host: Option<String>,
    pub source_name: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TransferQuery {
    pub status: Option<TransferStatus>,
    pub transfer_id: Option<String>,
    pub original_path: Option<String>,
    pub final_filename: Option<String>,
    pub username: Option<String>,
    pub source_name: Option<String>,
    pub remote_addr: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AssetGroupQuery {
    pub username: Option<String>,
    pub source_name: Option<String>,
    pub original_path: Option<String>,
    pub remote_addr: Option<String>,
    pub format: Option<ObjectFormat>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuplicateInfo {
    pub index: usize,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferRecordView {
    pub record: TransferRecord,
    pub display_source: Option<String>,
    pub virtual_display_path: String,
    pub final_location_kind: Option<String>,
    pub final_location_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferSummary {
    pub total_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetFacetCount {
    pub value: String,
    pub group_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetGroupSummary {
    pub group_count: usize,
    pub asset_count: usize,
    pub groups_with_jpeg: usize,
    pub groups_with_raw: usize,
    pub groups_with_video: usize,
    pub source_counts: Vec<AssetFacetCount>,
    pub remote_addr_counts: Vec<AssetFacetCount>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetGroupPage {
    pub groups: Vec<ReceivedAssetGroup>,
    pub summary: AssetGroupSummary,
    pub offset: usize,
    pub limit: usize,
    pub total_groups: usize,
    pub has_more: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectedDeviceView {
    pub device: ConnectedDevice,
    pub display_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountView {
    pub username: String,
    pub device_name: String,
    pub password_configured: bool,
    pub online: bool,
    pub active_connections: u32,
    pub last_remote_addr: Option<String>,
    pub last_remote_port: Option<u16>,
    pub last_seen_at_ms: Option<i64>,
    pub last_disconnected_at_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemPathsView {
    pub config_path: PathBuf,
    pub state_dir: PathBuf,
    pub output_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CameraConnectorDashboard {
    pub receiver_status: Option<ReceiverRuntimeStatus>,
    pub receiver_settings: ReceiverSettingsConfig,
    pub paths: SystemPathsView,
    pub accounts: Vec<AccountView>,
    pub devices: Vec<ConnectedDeviceView>,
    pub transfers: TransferSummary,
    pub recent_failures: Vec<TransferRecordView>,
    pub assets: AssetGroupPage,
}

impl CameraConnectorService {
    pub fn new(config_path: Option<PathBuf>) -> Self {
        Self { config_path }
    }

    pub fn config_path(&self) -> PathBuf {
        CameraConnectorConfig::resolved_path(self.config_path.as_deref())
    }

    pub fn state_dir(&self) -> PathBuf {
        CameraConnectorConfig::default_state_dir(self.config_path.as_deref())
    }

    pub fn load_config(&self) -> Result<CameraConnectorConfig> {
        CameraConnectorConfig::load(self.config_path.as_deref())
    }

    pub fn save_config(&self, config: &CameraConnectorConfig) -> Result<PathBuf> {
        config.save(self.config_path.as_deref())
    }

    pub fn receiver_config(&self, request: ReceiverConfigRequest) -> Result<PushReceiverConfig> {
        let app_config = self.load_config()?;
        let receiver_settings = app_config.receiver.clone();
        let protocol = request.protocol.unwrap_or(receiver_settings.protocol);
        let bind_host = request
            .bind_host
            .unwrap_or_else(|| receiver_settings.bind_host.clone());
        let port = request.port.unwrap_or(match protocol {
            PushProtocol::Ftp => receiver_settings.ftp_port,
            PushProtocol::Sftp => receiver_settings.sftp_port,
        });
        let output_dir = request
            .output_dir
            .or(receiver_settings.output_dir.clone())
            .unwrap_or_else(CameraConnectorConfig::default_output_dir);
        let state_dir = request
            .state_dir
            .or(receiver_settings.state_dir.clone())
            .unwrap_or_else(|| self.state_dir());
        let mut config = PushReceiverConfig::new(protocol, bind_host, port, output_dir)
            .with_state_dir(state_dir);
        config.advertised_host = request
            .advertised_host
            .or(receiver_settings.advertised_host);
        config.source_name = request.source_name.or(receiver_settings.source_name);
        config.accounts = app_config.effective_accounts(
            request.username.as_deref(),
            request.password.as_deref(),
            config.source_name.as_deref(),
        )?;
        config.active_project_id = self.active_project()?.map(|project| project.project_id);
        Ok(config)
    }

    pub fn storage_store(&self) -> Result<SqliteStore> {
        SqliteStore::open_state_dir(self.state_dir())
    }

    pub fn create_project(&self, name: impl AsRef<str>) -> Result<crate::Project> {
        self.storage_store()?.create_project(name)
    }

    pub fn set_active_project(&self, project_id: &str) -> Result<()> {
        self.storage_store()?.set_active_project(project_id)
    }

    pub fn active_project(&self) -> Result<Option<crate::Project>> {
        self.storage_store()?.active_project()
    }

    pub fn list_projects(&self) -> Result<Vec<crate::Project>> {
        self.storage_store()?.list_projects()
    }

    pub fn record_project_transfer(&self, project_id: &str, record: TransferRecord) -> Result<()> {
        self.storage_store()?.record_transfer(project_id, record)
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

    pub fn set_receiver_settings(
        &self,
        update: ReceiverSettingsUpdate,
    ) -> Result<(ReceiverSettingsConfig, PathBuf)> {
        let mut config = self.load_config()?;
        if let Some(protocol) = update.protocol {
            config.receiver.protocol = protocol;
        }
        if let Some(bind_host) = update.bind_host {
            config.receiver.bind_host = bind_host;
        }
        if let Some(ftp_port) = update.ftp_port {
            config.receiver.ftp_port = ftp_port;
        }
        if let Some(sftp_port) = update.sftp_port {
            config.receiver.sftp_port = sftp_port;
        }
        if let Some(output_dir) = update.output_dir {
            config.receiver.output_dir = Some(output_dir);
        }
        if let Some(state_dir) = update.state_dir {
            config.receiver.state_dir = Some(state_dir);
        }
        if let Some(advertised_host) = update.advertised_host {
            config.receiver.advertised_host = Some(advertised_host);
        }
        if let Some(source_name) = update.source_name {
            config.receiver.source_name = Some(source_name);
        }
        let settings = config.receiver.clone();
        let path = self.save_config(&config)?;
        Ok((settings, path))
    }

    pub fn accounts(&self) -> Result<Vec<AccountView>> {
        Ok(self
            .load_config()?
            .accounts
            .into_values()
            .map(account_view)
            .collect())
    }

    pub fn inbox_groups(
        &self,
        output_dir: impl AsRef<Path>,
        source: ImportSource,
    ) -> Result<Vec<ReceivedAssetGroup>> {
        scan_inbox_groups(output_dir, source)
    }

    pub fn transfer_asset_groups(
        &self,
        state_dir: impl AsRef<Path>,
    ) -> Result<Vec<ReceivedAssetGroup>> {
        self.transfer_asset_groups_with_query(state_dir, AssetGroupQuery::default())
    }

    pub fn transfer_asset_groups_with_query(
        &self,
        state_dir: impl AsRef<Path>,
        query: AssetGroupQuery,
    ) -> Result<Vec<ReceivedAssetGroup>> {
        let accounts = self.load_config()?.accounts;
        let records = read_transfer_log(state_dir)?
            .into_iter()
            .filter(|record| record.status == TransferStatus::Completed)
            .collect::<Vec<_>>();
        let duplicates = duplicate_info_by_transfer_id(&records, &accounts);
        let assets = records
            .into_iter()
            .map(|record| asset_from_transfer_record(record, &accounts, &duplicates))
            .filter(|asset| asset.format.is_supported_media())
            .collect::<Vec<_>>();
        Ok(group_received_assets(assets)
            .into_iter()
            .filter(|group| asset_group_matches(group, &query))
            .collect())
    }

    pub fn transfer_asset_summary_with_query(
        &self,
        state_dir: impl AsRef<Path>,
        query: AssetGroupQuery,
    ) -> Result<AssetGroupSummary> {
        self.transfer_asset_groups_with_query(state_dir, query)
            .map(|groups| summarize_asset_groups(&groups))
    }

    pub fn transfer_asset_group_page_with_query(
        &self,
        state_dir: impl AsRef<Path>,
        query: AssetGroupQuery,
        offset: usize,
        limit: usize,
    ) -> Result<AssetGroupPage> {
        let groups = self.transfer_asset_groups_with_query(state_dir, query)?;
        let total_groups = groups.len();
        let summary = summarize_asset_groups(&groups);
        let page_groups = groups
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect::<Vec<_>>();
        Ok(AssetGroupPage {
            groups: page_groups,
            summary,
            offset,
            limit,
            total_groups,
            has_more: offset.saturating_add(limit) < total_groups,
        })
    }

    pub fn project_asset_group_page_with_query(
        &self,
        project_id: &str,
        query: AssetGroupQuery,
        offset: usize,
        limit: usize,
    ) -> Result<AssetGroupPage> {
        self.storage_store()?
            .asset_group_page(project_id, query, offset, limit)
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
        let accounts = self.load_config()?.accounts;
        let views = read_transfer_log(output_dir)?
            .into_iter()
            .filter(|record| transfer_matches(record, &query, &accounts))
            .map(|record| {
                let display_source = record_display_source(&record, &accounts);
                let virtual_display_path = record.virtual_display_path(display_source.as_deref());
                TransferRecordView {
                    final_location_kind: record.final_location_kind().map(ToOwned::to_owned),
                    final_location_label: record.final_location_label(),
                    record,
                    display_source,
                    virtual_display_path,
                }
            })
            .collect::<Vec<_>>();
        Ok(views)
    }

    pub fn transfer_summary_with_query(
        &self,
        output_dir: impl AsRef<Path>,
        query: TransferQuery,
    ) -> Result<TransferSummary> {
        let accounts = self.load_config()?.accounts;
        let records = read_transfer_log(output_dir)?
            .into_iter()
            .filter(|record| transfer_matches(record, &query, &accounts))
            .collect::<Vec<_>>();
        Ok(summarize_transfers(&records))
    }

    pub fn recent_failed_transfers(
        &self,
        output_dir: impl AsRef<Path>,
        query: TransferQuery,
        limit: usize,
    ) -> Result<Vec<TransferRecordView>> {
        let mut views = self.transfers(
            output_dir,
            TransferQuery {
                status: Some(TransferStatus::Failed),
                ..query
            },
        )?;
        views.sort_by(|left, right| {
            let left_at = left
                .record
                .completed_at_ms
                .unwrap_or(left.record.started_at_ms);
            let right_at = right
                .record
                .completed_at_ms
                .unwrap_or(right.record.started_at_ms);
            right_at
                .cmp(&left_at)
                .then_with(|| right.record.started_at_ms.cmp(&left.record.started_at_ms))
                .then_with(|| right.record.transfer_id.cmp(&left.record.transfer_id))
        });
        views.truncate(limit);
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

    pub fn dashboard(
        &self,
        state_dir: impl AsRef<Path>,
        asset_query: AssetGroupQuery,
        offset: usize,
        limit: usize,
        online_devices_only: bool,
    ) -> Result<CameraConnectorDashboard> {
        let state_dir = state_dir.as_ref();
        let config = self.load_config()?;
        let receiver_settings = config.receiver.clone();
        let receiver_status = self.receiver_status(state_dir)?;
        let devices = self.connected_devices(
            state_dir,
            asset_query.username.as_deref(),
            online_devices_only,
        )?;
        let accounts = accounts_with_devices(self.accounts()?, &devices);
        Ok(CameraConnectorDashboard {
            receiver_settings,
            paths: SystemPathsView {
                config_path: self.config_path(),
                state_dir: state_dir.to_path_buf(),
                output_dir: receiver_status
                    .as_ref()
                    .and_then(|status| status.output_dir.clone()),
            },
            receiver_status,
            accounts,
            devices,
            transfers: self.transfer_summary_with_query(
                state_dir,
                transfer_query_from_asset_query(&asset_query),
            )?,
            recent_failures: self.recent_failed_transfers(
                state_dir,
                transfer_query_from_asset_query(&asset_query),
                5,
            )?,
            assets: self.transfer_asset_group_page_with_query(
                state_dir,
                asset_query,
                offset,
                limit,
            )?,
        })
    }

    pub fn project_dashboard(
        &self,
        project_id: &str,
        asset_query: AssetGroupQuery,
        offset: usize,
        limit: usize,
        online_devices_only: bool,
    ) -> Result<CameraConnectorDashboard> {
        let state_dir = self.state_dir();
        let config = self.load_config()?;
        let receiver_settings = config.receiver.clone();
        let receiver_status = self.receiver_status(&state_dir)?;
        let devices = self.connected_devices(
            &state_dir,
            asset_query.username.as_deref(),
            online_devices_only,
        )?;
        let accounts = accounts_with_devices(self.accounts()?, &devices);
        let store = self.storage_store()?;
        let (total_count, completed_count, failed_count) = store.transfer_counts(project_id)?;
        Ok(CameraConnectorDashboard {
            receiver_settings,
            paths: SystemPathsView {
                config_path: self.config_path(),
                state_dir: state_dir.clone(),
                output_dir: receiver_status
                    .as_ref()
                    .and_then(|status| status.output_dir.clone()),
            },
            receiver_status,
            accounts,
            devices,
            transfers: TransferSummary {
                total_count,
                completed_count,
                failed_count,
            },
            recent_failures: Vec::new(),
            assets: store.asset_group_page(project_id, asset_query, offset, limit)?,
        })
    }
}

fn transfer_query_from_asset_query(query: &AssetGroupQuery) -> TransferQuery {
    TransferQuery {
        status: None,
        transfer_id: None,
        original_path: query.original_path.clone(),
        final_filename: None,
        username: query.username.clone(),
        source_name: query.source_name.clone(),
        remote_addr: query.remote_addr.clone(),
    }
}

fn summarize_transfers(records: &[TransferRecord]) -> TransferSummary {
    TransferSummary {
        total_count: records.len(),
        completed_count: records
            .iter()
            .filter(|record| record.status == TransferStatus::Completed)
            .count(),
        failed_count: records
            .iter()
            .filter(|record| record.status == TransferStatus::Failed)
            .count(),
    }
}

fn account_view(account: ReceiverAccountConfig) -> AccountView {
    let password_configured = account.password_configured();
    AccountView {
        username: account.username,
        device_name: account.device_name,
        password_configured,
        online: false,
        active_connections: 0,
        last_remote_addr: None,
        last_remote_port: None,
        last_seen_at_ms: None,
        last_disconnected_at_ms: None,
    }
}

fn accounts_with_devices(
    accounts: Vec<AccountView>,
    devices: &[ConnectedDeviceView],
) -> Vec<AccountView> {
    accounts
        .into_iter()
        .map(|mut account| {
            let matching_devices = devices
                .iter()
                .filter(|view| view.device.username.as_deref() == Some(account.username.as_str()));
            let mut active_connections = 0u32;
            let mut latest: Option<&ConnectedDevice> = None;
            for device in matching_devices.map(|view| &view.device) {
                active_connections = active_connections.saturating_add(device.active_connections);
                latest = Some(match latest {
                    Some(current)
                        if current.online && !device.online
                            || current.online == device.online
                                && current.last_seen_at_ms >= device.last_seen_at_ms =>
                    {
                        current
                    }
                    _ => device,
                });
            }
            if let Some(device) = latest {
                account.online = device.online;
                account.active_connections = active_connections;
                account.last_remote_addr = Some(device.remote_addr.clone());
                account.last_remote_port = device.last_remote_port;
                account.last_seen_at_ms = Some(device.last_seen_at_ms);
                account.last_disconnected_at_ms = device.last_disconnected_at_ms;
            }
            account
        })
        .collect()
}

fn summarize_asset_groups(groups: &[ReceivedAssetGroup]) -> AssetGroupSummary {
    let mut source_counts = BTreeMap::<String, usize>::new();
    let mut remote_addr_counts = BTreeMap::<String, usize>::new();

    for group in groups {
        if let Some(source) = group.primary.display_source.as_ref() {
            *source_counts.entry(source.clone()).or_default() += 1;
        }
        if let Some(remote_addr) = group.primary.remote_addr.as_ref() {
            *remote_addr_counts.entry(remote_addr.clone()).or_default() += 1;
        }
    }

    AssetGroupSummary {
        group_count: groups.len(),
        asset_count: groups.iter().map(|group| group_assets(group).len()).sum(),
        groups_with_jpeg: groups.iter().filter(|group| group.jpeg.is_some()).count(),
        groups_with_raw: groups.iter().filter(|group| group.raw.is_some()).count(),
        groups_with_video: groups.iter().filter(|group| group.video.is_some()).count(),
        source_counts: facet_counts(source_counts),
        remote_addr_counts: facet_counts(remote_addr_counts),
    }
}

fn facet_counts(counts: BTreeMap<String, usize>) -> Vec<AssetFacetCount> {
    counts
        .into_iter()
        .map(|(value, group_count)| AssetFacetCount { value, group_count })
        .collect()
}

fn asset_group_matches(group: &ReceivedAssetGroup, query: &AssetGroupQuery) -> bool {
    group_assets(group)
        .into_iter()
        .any(|asset| asset_matches(asset, query))
}

fn group_assets(group: &ReceivedAssetGroup) -> Vec<&ReceivedAsset> {
    let mut assets = Vec::new();
    push_unique_asset(&mut assets, &group.primary);
    if let Some(asset) = group.jpeg.as_ref() {
        push_unique_asset(&mut assets, asset);
    }
    if let Some(asset) = group.raw.as_ref() {
        push_unique_asset(&mut assets, asset);
    }
    if let Some(asset) = group.video.as_ref() {
        push_unique_asset(&mut assets, asset);
    }
    assets
}

fn push_unique_asset<'a>(assets: &mut Vec<&'a ReceivedAsset>, asset: &'a ReceivedAsset) {
    if !assets.iter().any(|existing| existing.id == asset.id) {
        assets.push(asset);
    }
}

fn asset_matches(asset: &ReceivedAsset, query: &AssetGroupQuery) -> bool {
    query
        .username
        .as_ref()
        .map(|expected| asset.username.as_ref() == Some(expected))
        .unwrap_or(true)
        && query
            .source_name
            .as_ref()
            .map(|expected| asset.display_source.as_ref() == Some(expected))
            .unwrap_or(true)
        && query
            .remote_addr
            .as_ref()
            .map(|expected| asset.remote_addr.as_ref() == Some(expected))
            .unwrap_or(true)
        && query
            .original_path
            .as_ref()
            .map(|expected| {
                asset
                    .original_path
                    .as_deref()
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .contains(&expected.to_ascii_lowercase())
            })
            .unwrap_or(true)
        && query
            .format
            .map(|expected| asset.format == expected)
            .unwrap_or(true)
}

fn asset_from_transfer_record(
    record: TransferRecord,
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
    duplicates: &BTreeMap<String, DuplicateInfo>,
) -> ReceivedAsset {
    let storage_location = record.resolved_final_location();
    let display_source = record_display_source(&record, accounts);
    let virtual_display_path = record.virtual_display_path(display_source.as_deref());
    let mut asset = ReceivedAsset::new(
        record.transfer_id.clone(),
        record.final_filename.clone(),
        record.size_bytes,
        import_source_from_protocol(&record.protocol),
    );
    asset.received_time_ms = record.completed_at_ms.or(Some(record.started_at_ms));
    asset.storage_location = storage_location;
    asset.original_path = Some(record.original_path);
    asset.username = record.username;
    asset.display_source = display_source;
    asset.remote_addr = record.remote_addr;
    asset.virtual_display_path = Some(virtual_display_path);
    if let Some(duplicate) = duplicates.get(&record.transfer_id) {
        asset.duplicate_index = Some(duplicate.index);
        asset.duplicate_count = Some(duplicate.count);
    }
    asset
}

fn duplicate_info_by_transfer_id(
    records: &[TransferRecord],
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
) -> BTreeMap<String, DuplicateInfo> {
    let mut duplicate_keys = BTreeMap::<String, Vec<&TransferRecord>>::new();
    for record in records {
        if let Some(key) = duplicate_key(record, accounts) {
            duplicate_keys.entry(key).or_default().push(record);
        }
    }

    let mut duplicates = BTreeMap::new();
    for duplicate_records in duplicate_keys.values_mut() {
        if duplicate_records.len() < 2 {
            continue;
        }
        duplicate_records.sort_by_key(|record| {
            (
                record.completed_at_ms.unwrap_or(record.started_at_ms),
                record.started_at_ms,
                record.transfer_id.clone(),
            )
        });
        let count = duplicate_records.len();
        for (index, record) in duplicate_records.iter().enumerate() {
            duplicates.insert(
                record.transfer_id.clone(),
                DuplicateInfo {
                    index: index + 1,
                    count,
                },
            );
        }
    }
    duplicates
}

fn duplicate_key(
    record: &TransferRecord,
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
) -> Option<String> {
    let original_path = normalized_duplicate_segment(&record.original_path)?;
    let identity = record
        .username
        .as_deref()
        .and_then(normalized_duplicate_segment)
        .or_else(|| {
            record_display_source(record, accounts)
                .and_then(|value| normalized_duplicate_segment(&value))
        })
        .or_else(|| {
            record
                .remote_addr
                .as_deref()
                .and_then(normalized_duplicate_segment)
        })
        .unwrap_or_else(|| "-".to_string());
    Some(format!("{identity}\t{original_path}"))
}

fn normalized_duplicate_segment(value: &str) -> Option<String> {
    let normalized = value.trim().replace('\\', "/").to_ascii_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn import_source_from_protocol(protocol: &str) -> ImportSource {
    match protocol.to_ascii_lowercase().as_str() {
        "ftp" => ImportSource::FtpPush,
        "sftp" => ImportSource::SftpPush,
        _ => ImportSource::ManualDrop,
    }
}

fn transfer_matches(
    record: &TransferRecord,
    query: &TransferQuery,
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
) -> bool {
    query
        .status
        .map(|expected| record.status == expected)
        .unwrap_or(true)
        && query
            .username
            .as_ref()
            .map(|expected| record.username.as_ref() == Some(expected))
            .unwrap_or(true)
        && query
            .source_name
            .as_ref()
            .map(|expected| record_display_source(record, accounts).as_ref() == Some(expected))
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

fn record_display_source(
    record: &TransferRecord,
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
) -> Option<String> {
    record
        .username
        .as_deref()
        .and_then(|username| accounts.get(username))
        .map(|account| account.device_name.clone())
        .or_else(|| record.source_name.clone())
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
