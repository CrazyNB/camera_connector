use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::{
    read_connected_devices, read_receiver_runtime_status, CameraConnectorConfig, PushProtocol,
    PushReceiverConfig, ReceiverAccountConfig, ReceiverRuntimeStatus, ReceiverSettingsConfig,
    Result, SqliteStore,
};

use super::asset_groups::{device_display_source, device_matches, remote_addr_display_label};
use super::support::{
    account_view, accounts_with_devices, receiver_account_configs_from_state_dir,
};
use super::{
    AccountView, CameraConnectorService, ConnectedDeviceView, ReceiverConfigRequest,
    ReceiverSettingsUpdate,
};

impl CameraConnectorService {
    pub fn receiver_config(&self, request: ReceiverConfigRequest) -> Result<PushReceiverConfig> {
        let mut app_config = self.load_config()?;
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
        config.defer_publish = request
            .defer_publish
            .unwrap_or(receiver_settings.defer_publish);
        let account_state_dir = receiver_settings
            .state_dir
            .clone()
            .unwrap_or_else(|| self.state_dir());
        let active_project_id = app_config.active_project_id.clone();
        app_config.accounts = receiver_account_configs_from_state_dir(account_state_dir)?;
        config.accounts = app_config.effective_accounts(
            request.username.as_deref(),
            request.password.as_deref(),
            config.source_name.as_deref(),
        )?;
        config.active_project_id = active_project_id;
        Ok(config)
    }

    pub fn storage_state_dir(&self) -> Result<PathBuf> {
        Ok(self
            .load_config()?
            .receiver
            .state_dir
            .unwrap_or_else(|| self.state_dir()))
    }

    pub fn storage_store(&self) -> Result<SqliteStore> {
        SqliteStore::open_state_dir(self.storage_state_dir()?)
    }

    pub(super) fn receiver_account_configs(
        &self,
    ) -> Result<BTreeMap<String, ReceiverAccountConfig>> {
        receiver_account_configs_from_state_dir(self.storage_state_dir()?)
    }

    pub fn set_account(
        &self,
        username: impl Into<String>,
        password: Option<&str>,
        device_name: impl Into<String>,
    ) -> Result<(ReceiverAccountConfig, PathBuf)> {
        let account = ReceiverAccountConfig::new(username, password, device_name)?;
        let stored = self.storage_store()?.upsert_receiver_account(account)?;
        Ok((stored.into_account_config()?, self.config_path()))
    }

    pub fn remove_account(&self, username: &str) -> Result<(bool, PathBuf)> {
        let removed = self.storage_store()?.remove_receiver_account(username)?;
        Ok((removed, self.config_path()))
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
        if let Some(defer_publish) = update.defer_publish {
            config.receiver.defer_publish = defer_publish;
        }
        let settings = config.receiver.clone();
        let path = self.save_config(&config)?;
        Ok((settings, path))
    }

    pub fn accounts(&self) -> Result<Vec<AccountView>> {
        let state_dir = self.storage_state_dir()?;
        let accounts = self
            .receiver_account_configs()?
            .into_values()
            .map(account_view)
            .collect();
        let devices = self.connected_devices(&state_dir, None, false)?;
        Ok(accounts_with_devices(accounts, &devices))
    }

    pub fn receiver_status(
        &self,
        output_dir: impl AsRef<Path>,
    ) -> Result<Option<ReceiverRuntimeStatus>> {
        read_receiver_runtime_status(output_dir)
    }

    pub fn connected_devices(
        &self,
        output_dir: impl AsRef<Path>,
        username: Option<&str>,
        online: bool,
    ) -> Result<Vec<ConnectedDeviceView>> {
        let accounts = self.receiver_account_configs()?;
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
