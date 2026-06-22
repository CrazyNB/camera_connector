use std::path::Path;
use std::str::FromStr;

#[cfg(test)]
use camera_connector_core::{
    AssetFacetCount, AssetGroupPage, AssetGroupSummary, CameraConnectorDashboard, ReceivedAsset,
    ReceivedAssetGroup, ReceiverRuntimeStatus, ReceiverSettingsConfig, TransferRecord,
    TransferRecordView,
};
use camera_connector_core::{
    AssetGroupQuery, CameraConnectorRuntime, CameraConnectorService, ImportSource, ObjectFormat,
    PushProtocol, PushReceiverConfig, ReceiverConfigRequest, ReceiverSettingsUpdate, Result,
    TransferQuery, TransferStatus,
};
use clap::Parser;

mod cli_args;
mod cli_support;

use cli_args::*;
use cli_support::*;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Version) | None => {
            println!("camera-connector {}", env!("CARGO_PKG_VERSION"));
        }
        Some(Command::ReceiveFile {
            input,
            output,
            project_id,
            state,
            source,
            username,
            source_name,
        }) => {
            handle_receive_file_command(ReceiveFileArgs {
                input,
                output,
                project_id,
                state,
                source: parse_source(&source)?,
                username,
                source_name,
            })?;
        }
        Some(Command::ReceiverConfig {
            config,
            protocol,
            bind_host,
            port,
            output,
            state,
            username,
            advertised_host,
            source_name,
        }) => {
            let protocol = PushProtocol::from_str(&protocol)?;
            let config = build_config(ConfigArgs {
                config_path: config,
                protocol,
                bind_host,
                port,
                output,
                state,
                username,
                password: None,
                advertised_host,
                source_name,
            })?;
            print_receiver_config(&config);
        }
        Some(Command::ReceiverSettings {
            config,
            protocol,
            bind_host,
            ftp_port,
            sftp_port,
            output,
            state,
            advertised_host,
            source_name,
        }) => {
            handle_receiver_settings_command(
                config.as_deref(),
                ReceiverSettingsArgs {
                    protocol: protocol
                        .as_deref()
                        .map(PushProtocol::from_str)
                        .transpose()?,
                    bind_host,
                    ftp_port,
                    sftp_port,
                    output,
                    state,
                    advertised_host,
                    source_name,
                },
            )?;
        }
        Some(Command::ReceiverStatus { state }) => {
            let service = CameraConnectorService::new(None);
            match service.receiver_status(state)? {
                Some(status) => print_receiver_status_lines(&status),
                None => {
                    println!("phase: Unknown");
                    println!("message: receiver status file not found");
                }
            }
        }
        Some(Command::Dashboard {
            config,
            project_id,
            username,
            source_name,
            original_path,
            remote_addr,
            format,
            offset,
            limit,
            online_devices,
            json,
        }) => {
            let dashboard = load_dashboard(DashboardArgs {
                config,
                project_id,
                query: AssetGroupQuery {
                    username,
                    source_name,
                    original_path,
                    remote_addr,
                    format: format.as_deref().map(parse_object_format).transpose()?,
                    role: None,
                    ..AssetGroupQuery::default()
                },
                offset,
                limit,
                online_devices,
            })?;
            if json {
                print_dashboard_json(&dashboard)?;
            } else {
                print_dashboard(dashboard);
            }
        }
        Some(Command::ServeFtp {
            config,
            bind_host,
            port,
            output,
            state,
            username,
            password,
            advertised_host,
            source_name,
        }) => {
            let service = CameraConnectorService::new(config.clone());
            let runtime = CameraConnectorRuntime::new(service.clone());
            let request = ReceiverConfigRequest {
                protocol: Some(PushProtocol::Ftp),
                bind_host: Some(bind_host),
                port: Some(port),
                output_dir: Some(output),
                state_dir: state,
                username,
                password,
                advertised_host,
                source_name,
                defer_publish: None,
            };
            let receiver_config = service.receiver_config(request.clone())?;
            let status = runtime.start_receiver(request).await?;
            let local_addr = status.local_addr.ok_or_else(|| {
                camera_connector_core::ImporterError::internal("missing local address")
            })?;
            println!("ftp receiver listening on {local_addr}");
            print_receiver_config(&receiver_config);
            tokio::signal::ctrl_c().await?;
            runtime.stop_receiver().await?;
        }
        Some(Command::ServeSftp {
            config,
            bind_host,
            port,
            output,
            state,
            username,
            password,
            advertised_host,
            source_name,
        }) => {
            let service = CameraConnectorService::new(config.clone());
            let runtime = CameraConnectorRuntime::new(service.clone());
            let request = ReceiverConfigRequest {
                protocol: Some(PushProtocol::Sftp),
                bind_host: Some(bind_host),
                port: Some(port),
                output_dir: Some(output),
                state_dir: state,
                username,
                password,
                advertised_host,
                source_name,
                defer_publish: None,
            };
            let receiver_config = service.receiver_config(request.clone())?;
            let status = runtime.start_receiver(request).await?;
            let local_addr = status.local_addr.ok_or_else(|| {
                camera_connector_core::ImporterError::internal("missing local address")
            })?;
            println!("sftp receiver listening on {local_addr}");
            print_receiver_config(&receiver_config);
            tokio::signal::ctrl_c().await?;
            runtime.stop_receiver().await?;
        }
        Some(Command::Assets {
            config,
            path,
            project_id,
            diagnostic: _,
            source,
            from_transfers,
            summary,
            username,
            source_name,
            original_path,
            remote_addr,
            format,
            offset,
            limit,
        }) => {
            let source = parse_source(&source)?;
            let service = CameraConnectorService::new(config.clone());
            let query = AssetGroupQuery {
                username,
                source_name,
                original_path,
                remote_addr,
                format: format.as_deref().map(parse_object_format).transpose()?,
                role: None,
                ..AssetGroupQuery::default()
            };
            let groups = if let Some(project_id) = project_id {
                let page = load_project_asset_page(
                    config,
                    &project_id,
                    query,
                    offset,
                    limit.unwrap_or(50),
                )?;
                if summary {
                    println!("{}", asset_group_page_summary_line(&page));
                }
                page.groups
            } else if from_transfers {
                let path = path.ok_or(camera_connector_core::ImporterError::InvalidUploadPath)?;
                if let Some(limit) = limit {
                    let page = service.diagnostic_transfer_asset_group_page_with_query(
                        path, query, offset, limit,
                    )?;
                    if summary {
                        println!("{}", asset_group_page_summary_line(&page));
                    }
                    page.groups
                } else {
                    if summary {
                        let summary = service
                            .diagnostic_transfer_asset_summary_with_query(&path, query.clone())?;
                        println!("{}", asset_group_summary_line(&summary));
                    }
                    service.diagnostic_transfer_asset_groups_with_query(path, query)?
                }
            } else {
                let path = path.ok_or(camera_connector_core::ImporterError::InvalidUploadPath)?;
                service.diagnostic_received_asset_groups(path, source)?
            };
            print_asset_groups(groups);
        }
        Some(Command::Transfers {
            config,
            state,
            project_id,
            diagnostic: _,
            status,
            transfer_id,
            original_path,
            final_filename,
            username,
            source_name,
            remote_addr,
        }) => {
            for view in load_transfers(
                config,
                state,
                project_id,
                TransferQuery {
                    status: status.as_deref().map(parse_transfer_status).transpose()?,
                    transfer_id,
                    original_path,
                    final_filename,
                    username,
                    source_name,
                    remote_addr,
                },
            )? {
                println!("{}", transfer_view_line(&view));
            }
        }
        Some(Command::Account { config, action }) => {
            handle_account_command(config.as_deref(), action)?;
        }
        Some(Command::Project { config, action }) => {
            handle_project_command(config.as_deref(), action)?;
        }
        Some(Command::Devices {
            config,
            state,
            username,
            online,
        }) => {
            let service = CameraConnectorService::new(config);
            for view in service.connected_devices(state, username.as_deref(), online)? {
                let device = view.device;
                println!(
                    "{}\tonline={}\tconnections={}\tport={}\tusername={}\tsource={}\tdisplay={}\tlast_seen_ms={}",
                    device.remote_addr,
                    device.online,
                    device.active_connections,
                    device
                        .last_remote_port
                        .map(|port| port.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    device.username.as_deref().unwrap_or("-"),
                    device.source_name.as_deref().unwrap_or("-"),
                    view.display_source,
                    device.last_seen_at_ms
                );
            }
        }
    }

    Ok(())
}

fn build_config(args: ConfigArgs) -> Result<PushReceiverConfig> {
    CameraConnectorService::new(args.config_path).receiver_config(ReceiverConfigRequest {
        protocol: Some(args.protocol),
        bind_host: Some(args.bind_host),
        port: Some(args.port),
        output_dir: Some(args.output),
        state_dir: args.state,
        username: args.username,
        password: args.password,
        advertised_host: args.advertised_host,
        source_name: args.source_name,
        defer_publish: None,
    })
}

fn print_receiver_config(config: &PushReceiverConfig) {
    println!("protocol: {}", config.protocol);
    println!(
        "host: {}",
        config
            .advertised_host
            .as_deref()
            .unwrap_or(&config.bind_host)
    );
    println!("port: {}", config.port);
    println!("output: {}", config.output_dir.display());
    println!("state: {}", config.state_dir.display());
    println!("accounts: {}", config.accounts.len());
    for account in &config.accounts {
        println!(
            "account: {}\tdevice={}\tpassword={}",
            account.username,
            account.device_name,
            if account.password.is_some() {
                "configured"
            } else {
                "not required"
            }
        );
    }
    if config.accounts.is_empty() {
        println!("username: anonymous");
        println!("password: not required");
    }
    println!(
        "source_name: {}",
        config.source_name.as_deref().unwrap_or("-")
    );
}

fn parse_source(value: &str) -> Result<ImportSource> {
    match value.to_ascii_lowercase().as_str() {
        "ftp" | "ftp-push" => Ok(ImportSource::FtpPush),
        "sftp" | "sftp-push" => Ok(ImportSource::SftpPush),
        "manual" | "manual-drop" => Ok(ImportSource::ManualDrop),
        _ => Err(camera_connector_core::ImporterError::UnsupportedProtocol),
    }
}

fn parse_transfer_status(value: &str) -> Result<TransferStatus> {
    match value.to_ascii_lowercase().as_str() {
        "completed" | "complete" | "ok" => Ok(TransferStatus::Completed),
        "failed" | "fail" | "error" => Ok(TransferStatus::Failed),
        _ => Err(camera_connector_core::ImporterError::InvalidUploadPath),
    }
}

fn parse_object_format(value: &str) -> Result<ObjectFormat> {
    ObjectFormat::from_str(value)
        .map_err(|_| camera_connector_core::ImporterError::InvalidUploadPath)
}

fn handle_account_command(config_path: Option<&Path>, action: AccountCommand) -> Result<()> {
    let service = CameraConnectorService::new(config_path.map(Path::to_path_buf));
    match action {
        AccountCommand::List => {
            let accounts = service.accounts()?;
            println!("config: {}", service.config_path().display());
            if accounts.is_empty() {
                println!("accounts: -");
            } else {
                for account in accounts {
                    println!("{}", account_view_line(&account));
                }
            }
        }
        AccountCommand::Set {
            username,
            password,
            device_name,
        } => {
            let (account, path) =
                service.set_account(username, password.as_deref(), device_name)?;
            println!(
                "saved account {}\tdevice={}",
                account.username, account.device_name
            );
            println!("config: {}", path.display());
        }
        AccountCommand::Remove { username } => {
            let (removed, path) = service.remove_account(&username)?;
            println!(
                "{} {username}",
                if removed { "removed" } else { "not_found" }
            );
            println!("config: {}", path.display());
        }
    }
    Ok(())
}

fn handle_project_command(config_path: Option<&Path>, action: ProjectCommand) -> Result<()> {
    let service = CameraConnectorService::new(config_path.map(Path::to_path_buf));
    match action {
        ProjectCommand::List => {
            let active_project = service.active_project()?;
            let active_project_id = active_project
                .as_ref()
                .map(|project| project.project_id.as_str());
            let projects = service.list_projects()?;
            if projects.is_empty() {
                println!("projects: -");
            } else {
                for project in projects {
                    println!("{}", project_line(&project, active_project_id));
                }
            }
        }
        ProjectCommand::Create { name } => {
            let project = service.create_project(name)?;
            service.set_active_project(&project.project_id)?;
            println!(
                "{}",
                project_line(&project, Some(project.project_id.as_str()))
            );
        }
        ProjectCommand::Active => {
            let project = service.active_project()?.ok_or_else(|| {
                camera_connector_core::ImporterError::internal(
                    "no active project selected; create and select a project first",
                )
            })?;
            println!(
                "{}",
                project_line(&project, Some(project.project_id.as_str()))
            );
        }
        ProjectCommand::Select { id } => {
            service.set_active_project(&id)?;
            let project = service.active_project()?.ok_or_else(|| {
                camera_connector_core::ImporterError::internal("active project was not set")
            })?;
            println!(
                "{}",
                project_line(&project, Some(project.project_id.as_str()))
            );
        }
        ProjectCommand::Archive { id } => {
            let project = service.archive_project(&id)?;
            println!("{}", project_line(&project, None));
        }
        ProjectCommand::Restore { id } => {
            let project = service.restore_project(&id)?;
            let active_project = service.active_project()?;
            let active_project_id = active_project
                .as_ref()
                .map(|project| project.project_id.as_str());
            println!("{}", project_line(&project, active_project_id));
        }
        ProjectCommand::Rename { id, name } => {
            let project = service.rename_project(&id, name)?;
            let active_project = service.active_project()?;
            let active_project_id = active_project
                .as_ref()
                .map(|project| project.project_id.as_str());
            println!("{}", project_line(&project, active_project_id));
        }
        ProjectCommand::GroupAssets { id, group_id } => {
            let assets =
                load_project_group_assets(config_path.map(Path::to_path_buf), &id, &group_id)?;
            print_stored_assets(assets);
        }
    }
    Ok(())
}

fn handle_receiver_settings_command(
    config_path: Option<&Path>,
    args: ReceiverSettingsArgs,
) -> Result<()> {
    let service = CameraConnectorService::new(config_path.map(Path::to_path_buf));
    let (settings, path) = service.set_receiver_settings(ReceiverSettingsUpdate {
        protocol: args.protocol,
        bind_host: args.bind_host,
        ftp_port: args.ftp_port,
        sftp_port: args.sftp_port,
        output_dir: args.output,
        state_dir: args.state,
        advertised_host: args.advertised_host,
        source_name: args.source_name,
        defer_publish: None,
    })?;
    println!("config: {}", path.display());
    println!("protocol: {}", settings.protocol);
    println!("bind_host: {}", settings.bind_host);
    println!("ftp_port: {}", settings.ftp_port);
    println!("sftp_port: {}", settings.sftp_port);
    println!(
        "output: {}",
        settings
            .output_dir
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "state: {}",
        settings
            .state_dir
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "advertised_host: {}",
        settings.advertised_host.as_deref().unwrap_or("-")
    );
    println!(
        "source_name: {}",
        settings.source_name.as_deref().unwrap_or("-")
    );
    Ok(())
}

fn source_protocol_label(source: ImportSource) -> &'static str {
    match source {
        ImportSource::FtpPush => "ftp",
        ImportSource::SftpPush => "sftp",
        ImportSource::ManualDrop => "manual",
        ImportSource::DesktopScan => "desktop_scan",
    }
}

fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
