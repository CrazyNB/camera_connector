use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[cfg(test)]
use camera_connector_core::ReceiverSettingsConfig;
use camera_connector_core::{
    append_transfer_record, AssetFacetCount, AssetGroupPage, AssetGroupQuery, AssetGroupSummary,
    CameraConnectorDashboard, CameraConnectorRuntime, CameraConnectorService, ImportSource,
    LocalFileSink, ObjectFormat, Project, PushProtocol, PushReceiverConfig, ReceivedAsset,
    ReceivedAssetGroup, ReceiverConfigRequest, ReceiverRuntimeStatus, ReceiverSettingsUpdate,
    Result, SqliteStore, StoredObjectLocation, TransferQuery, TransferRecord, TransferRecordView,
    TransferStatus,
};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "camera-connector")]
#[command(about = "Push-mode wireless import receiver for cameras")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Version,
    ReceiveFile {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        state: Option<PathBuf>,
        #[arg(long, default_value = "manual")]
        source: String,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        source_name: Option<String>,
    },
    ReceiverConfig {
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, default_value = "ftp")]
        protocol: String,
        #[arg(long, default_value = "0.0.0.0")]
        bind_host: String,
        #[arg(long, default_value_t = 2121)]
        port: u16,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        state: Option<PathBuf>,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        advertised_host: Option<String>,
        #[arg(long)]
        source_name: Option<String>,
    },
    ReceiverSettings {
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        protocol: Option<String>,
        #[arg(long)]
        bind_host: Option<String>,
        #[arg(long)]
        ftp_port: Option<u16>,
        #[arg(long)]
        sftp_port: Option<u16>,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        state: Option<PathBuf>,
        #[arg(long)]
        advertised_host: Option<String>,
        #[arg(long)]
        source_name: Option<String>,
    },
    ReceiverStatus {
        #[arg(long, alias = "path")]
        state: PathBuf,
    },
    Dashboard {
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, alias = "path")]
        state: Option<PathBuf>,
        #[arg(long, alias = "project")]
        project_id: Option<String>,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        source_name: Option<String>,
        #[arg(long)]
        original_path: Option<String>,
        #[arg(long)]
        remote_addr: Option<String>,
        #[arg(long)]
        format: Option<String>,
        #[arg(long, default_value_t = 0)]
        offset: usize,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long)]
        online_devices: bool,
        #[arg(long)]
        json: bool,
    },
    ServeFtp {
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, default_value = "0.0.0.0")]
        bind_host: String,
        #[arg(long, default_value_t = 2121)]
        port: u16,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        state: Option<PathBuf>,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        password: Option<String>,
        #[arg(long)]
        advertised_host: Option<String>,
        #[arg(long)]
        source_name: Option<String>,
    },
    ServeSftp {
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, default_value = "0.0.0.0")]
        bind_host: String,
        #[arg(long, default_value_t = 2222)]
        port: u16,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        state: Option<PathBuf>,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        password: Option<String>,
        #[arg(long)]
        advertised_host: Option<String>,
        #[arg(long)]
        source_name: Option<String>,
    },
    Inbox {
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        path: PathBuf,
        #[arg(long, default_value = "ftp")]
        source: String,
        #[arg(long)]
        from_transfers: bool,
        #[arg(long)]
        summary: bool,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        source_name: Option<String>,
        #[arg(long)]
        original_path: Option<String>,
        #[arg(long)]
        remote_addr: Option<String>,
        #[arg(long)]
        format: Option<String>,
        #[arg(long, default_value_t = 0)]
        offset: usize,
        #[arg(long)]
        limit: Option<usize>,
    },
    Transfers {
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, alias = "path")]
        state: PathBuf,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        transfer_id: Option<String>,
        #[arg(long)]
        original_path: Option<String>,
        #[arg(long)]
        final_filename: Option<String>,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        source_name: Option<String>,
        #[arg(long)]
        remote_addr: Option<String>,
    },
    Devices {
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, alias = "path")]
        state: PathBuf,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        online: bool,
    },
    Account {
        #[arg(long)]
        config: Option<PathBuf>,
        #[command(subcommand)]
        action: AccountCommand,
    },
    Project {
        #[arg(long)]
        config: Option<PathBuf>,
        #[command(subcommand)]
        action: ProjectCommand,
    },
}

#[derive(Debug, Subcommand)]
enum AccountCommand {
    List,
    Set {
        #[arg(long)]
        username: String,
        #[arg(long)]
        password: Option<String>,
        #[arg(long = "device-name")]
        device_name: String,
    },
    Remove {
        #[arg(long)]
        username: String,
    },
}

#[derive(Debug, Subcommand)]
enum ProjectCommand {
    List,
    Create {
        #[arg(long)]
        name: String,
    },
    Active,
    Select {
        #[arg(long, alias = "project-id")]
        id: String,
    },
}

struct ConfigArgs {
    config_path: Option<PathBuf>,
    protocol: PushProtocol,
    bind_host: String,
    port: u16,
    output: PathBuf,
    state: Option<PathBuf>,
    username: Option<String>,
    password: Option<String>,
    advertised_host: Option<String>,
    source_name: Option<String>,
}

struct ReceiverSettingsArgs {
    protocol: Option<PushProtocol>,
    bind_host: Option<String>,
    ftp_port: Option<u16>,
    sftp_port: Option<u16>,
    output: Option<PathBuf>,
    state: Option<PathBuf>,
    advertised_host: Option<String>,
    source_name: Option<String>,
}

struct ReceiveFileArgs {
    input: PathBuf,
    output: PathBuf,
    state: Option<PathBuf>,
    source: ImportSource,
    username: Option<String>,
    source_name: Option<String>,
}

struct DashboardArgs {
    config: Option<PathBuf>,
    state: Option<PathBuf>,
    project_id: Option<String>,
    query: AssetGroupQuery,
    offset: usize,
    limit: usize,
    online_devices: bool,
}

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
            state,
            source,
            username,
            source_name,
        }) => {
            handle_receive_file_command(ReceiveFileArgs {
                input,
                output,
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
            state,
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
                state,
                project_id,
                query: AssetGroupQuery {
                    username,
                    source_name,
                    original_path,
                    remote_addr,
                    format: format.as_deref().map(parse_object_format).transpose()?,
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
        Some(Command::Inbox {
            config,
            path,
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
            let service = CameraConnectorService::new(config);
            let query = AssetGroupQuery {
                username,
                source_name,
                original_path,
                remote_addr,
                format: format.as_deref().map(parse_object_format).transpose()?,
            };
            let groups = if from_transfers {
                if let Some(limit) = limit {
                    let page =
                        service.transfer_asset_group_page_with_query(path, query, offset, limit)?;
                    if summary {
                        println!("{}", asset_group_page_summary_line(&page));
                    }
                    page.groups
                } else {
                    if summary {
                        let summary =
                            service.transfer_asset_summary_with_query(&path, query.clone())?;
                        println!("{}", asset_group_summary_line(&summary));
                    }
                    service.transfer_asset_groups_with_query(path, query)?
                }
            } else {
                service.inbox_groups(path, source)?
            };
            print_asset_groups(groups);
        }
        Some(Command::Transfers {
            config,
            state,
            status,
            transfer_id,
            original_path,
            final_filename,
            username,
            source_name,
            remote_addr,
        }) => {
            let service = CameraConnectorService::new(config);
            for view in service.transfers(
                state,
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

fn transfer_view_line(view: &TransferRecordView) -> String {
    let record = &view.record;
    format!(
        "{}\t{:?}\t{}\t{}\t{}\tusername={}\tremote={}\tsource={}\tdisplay={}\tlocation_kind={}\tlocation={}\terror={}",
        record.transfer_id,
        record.status,
        record.final_filename,
        record.original_path,
        record.size_bytes,
        record.username.as_deref().unwrap_or("-"),
        record.remote_addr.as_deref().unwrap_or("-"),
        view.display_source.as_deref().unwrap_or("-"),
        view.virtual_display_path,
        view.final_location_kind.as_deref().unwrap_or("-"),
        view.final_location_label.as_deref().unwrap_or("-"),
        record.error.as_deref().unwrap_or("-")
    )
}

fn handle_receive_file_command(args: ReceiveFileArgs) -> Result<TransferRecord> {
    let filename = args
        .input
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(camera_connector_core::ImporterError::InvalidUploadPath)?;
    let bytes = fs::read(&args.input)?;
    let started_at_ms = current_time_ms();
    let protocol_label = source_protocol_label(args.source);
    let progress = LocalFileSink::new(args.output).write_complete(
        format!("{protocol_label}:{started_at_ms}:{filename}"),
        filename,
        &bytes,
    )?;
    let asset = ReceivedAsset::new(
        progress.transfer_id,
        progress.filename,
        progress.bytes_written,
        args.source,
    );
    println!(
        "received {}\t{:?}\t{} bytes",
        asset.filename, asset.format, asset.size_bytes
    );
    let final_path = progress
        .output_path
        .clone()
        .ok_or_else(|| camera_connector_core::ImporterError::internal("missing output path"))?;
    let log_dir = args.state.unwrap_or_else(|| {
        final_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    });
    let record = TransferRecord {
        transfer_id: asset.id,
        protocol: protocol_label.to_string(),
        status: TransferStatus::Completed,
        original_path: filename.to_string(),
        final_filename: asset.filename,
        final_path: Some(final_path),
        final_location: progress.output_location,
        size_bytes: progress.bytes_written,
        username: args.username,
        remote_addr: None,
        source_name: args.source_name,
        started_at_ms,
        completed_at_ms: Some(current_time_ms()),
        error: None,
    };
    append_transfer_record(&log_dir, &record)?;
    record_transfer_in_active_project(&log_dir, &record)?;
    Ok(record)
}

fn record_transfer_in_active_project(state_dir: &Path, record: &TransferRecord) -> Result<()> {
    let store = SqliteStore::open_state_dir(state_dir)?;
    let project = match store.active_project()? {
        Some(project) => project,
        None => {
            let project = store.ensure_inbox_project()?;
            store.set_active_project(&project.project_id)?;
            project
        }
    };
    store.record_transfer(&project.project_id, record.clone())
}

fn load_dashboard(args: DashboardArgs) -> Result<CameraConnectorDashboard> {
    let service = CameraConnectorService::new(args.config);
    match args.project_id {
        Some(project_id) => service.project_dashboard(
            &project_id,
            args.query,
            args.offset,
            args.limit,
            args.online_devices,
        ),
        None => {
            let state = args
                .state
                .ok_or(camera_connector_core::ImporterError::InvalidUploadPath)?;
            service.dashboard(
                state,
                args.query,
                args.offset,
                args.limit,
                args.online_devices,
            )
        }
    }
}

fn project_line(project: &Project, active_project_id: Option<&str>) -> String {
    format!(
        "project\tid={}\tname={}\tslug={}\tstatus={}\tactive={}",
        project.project_id,
        project.name,
        project.slug,
        project.status.as_str(),
        active_project_id == Some(project.project_id.as_str())
    )
}

fn print_asset_groups(groups: Vec<ReceivedAssetGroup>) {
    for group in groups {
        println!("{}", asset_group_line(&group));
    }
}

fn print_dashboard(dashboard: CameraConnectorDashboard) {
    match dashboard.receiver_status {
        Some(status) => println!("status\t{}", receiver_status_tab_fields(&status)),
        None => println!("status\tphase=Unknown\tmessage=receiver status file not found"),
    }
    println!(
        "paths\tconfig={}\tstate={}\toutput={}",
        dashboard.paths.config_path.display(),
        dashboard.paths.state_dir.display(),
        dashboard
            .paths
            .output_dir
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    for account in dashboard.accounts {
        println!(
            "account\tusername={}\tdevice={}\tpassword_configured={}\tonline={}\tconnections={}\tremote={}\tport={}\tlast_seen_ms={}\tlast_disconnected_ms={}",
            account.username,
            account.device_name,
            account.password_configured,
            account.online,
            account.active_connections,
            account.last_remote_addr.as_deref().unwrap_or("-"),
            account
                .last_remote_port
                .map(|port| port.to_string())
                .unwrap_or_else(|| "-".to_string()),
            account
                .last_seen_at_ms
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
            account
                .last_disconnected_at_ms
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string())
        );
    }
    for view in dashboard.devices {
        let device = view.device;
        println!(
            "device\t{}\tonline={}\tconnections={}\tport={}\tusername={}\tsource={}\tdisplay={}\tlast_seen_ms={}",
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
    println!(
        "transfers\ttotal={}\tcompleted={}\tfailed={}",
        dashboard.transfers.total_count,
        dashboard.transfers.completed_count,
        dashboard.transfers.failed_count
    );
    for view in dashboard.recent_failures {
        println!("failure\t{}", transfer_view_line(&view));
    }
    println!(
        "summary\t{}",
        asset_group_page_summary_line(&dashboard.assets).trim_start_matches("summary\t")
    );
    for group in dashboard.assets.groups {
        println!("asset\t{}", asset_group_line(&group));
    }
}

fn print_dashboard_json(dashboard: &CameraConnectorDashboard) -> Result<()> {
    println!("{}", dashboard_json(dashboard)?);
    Ok(())
}

fn dashboard_json(dashboard: &CameraConnectorDashboard) -> Result<String> {
    serde_json::to_string_pretty(dashboard)
        .map_err(|error| camera_connector_core::ImporterError::internal(error.to_string()))
}

fn print_receiver_status_lines(status: &ReceiverRuntimeStatus) {
    println!("phase: {:?}", status.phase);
    println!(
        "protocol: {}",
        status
            .protocol
            .map(|protocol| protocol.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!("auth_mode: {:?}", status.auth_mode);
    println!(
        "local_addr: {}",
        status
            .local_addr
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "output: {}",
        status
            .output_dir
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "state: {}",
        status
            .state_dir
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!("accounts: {}", status.account_count);
    println!("message: {}", status.message.as_deref().unwrap_or("-"));
}

fn receiver_status_tab_fields(status: &ReceiverRuntimeStatus) -> String {
    format!(
        "phase={:?}\tprotocol={}\tauth_mode={:?}\tlocal_addr={}\toutput={}\tstate={}\taccounts={}\tmessage={}",
        status.phase,
        status
            .protocol
            .map(|protocol| protocol.to_string())
            .unwrap_or_else(|| "-".to_string()),
        status.auth_mode,
        status
            .local_addr
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "-".to_string()),
        status
            .output_dir
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "-".to_string()),
        status
            .state_dir
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "-".to_string()),
        status.account_count,
        status.message.as_deref().unwrap_or("-")
    )
}

fn asset_group_summary_line(summary: &AssetGroupSummary) -> String {
    format!(
        "summary\tgroups={}\tassets={}\tjpeg_groups={}\traw_groups={}\tvideo_groups={}\tsources={}\tremotes={}",
        summary.group_count,
        summary.asset_count,
        summary.groups_with_jpeg,
        summary.groups_with_raw,
        summary.groups_with_video,
        facet_counts_label(&summary.source_counts),
        facet_counts_label(&summary.remote_addr_counts)
    )
}

fn asset_group_page_summary_line(page: &AssetGroupPage) -> String {
    format!(
        "{}\toffset={}\tlimit={}\ttotal_groups={}\thas_more={}",
        asset_group_summary_line(&page.summary),
        page.offset,
        page.limit,
        page.total_groups,
        page.has_more
    )
}

fn facet_counts_label(counts: &[AssetFacetCount]) -> String {
    if counts.is_empty() {
        return "-".to_string();
    }
    counts
        .iter()
        .map(|count| format!("{}:{}", count.value, count.group_count))
        .collect::<Vec<_>>()
        .join(",")
}

fn asset_group_line(group: &ReceivedAssetGroup) -> String {
    let jpeg = group
        .jpeg
        .as_ref()
        .map(|asset| asset.filename.as_str())
        .unwrap_or("-");
    let raw = group
        .raw
        .as_ref()
        .map(|asset| asset.filename.as_str())
        .unwrap_or("-");
    let video = group
        .video
        .as_ref()
        .map(|asset| asset.filename.as_str())
        .unwrap_or("-");
    let total_bytes = group
        .jpeg
        .iter()
        .chain(group.raw.iter())
        .chain(group.video.iter())
        .map(|asset| asset.size_bytes)
        .sum::<u64>();
    let primary_location = group.primary.storage_location.as_ref();

    format!(
        "{}\tprimary={}\tjpeg={}\traw={}\tvideo={}\t{} bytes\tusername={}\tsource={}\tremote={}\toriginal={}\tdisplay={}\tduplicate={}\tprimary_location_kind={}\tprimary_location={}",
        group.group_key,
        group.primary.filename,
        jpeg,
        raw,
        video,
        total_bytes,
        group.primary.username.as_deref().unwrap_or("-"),
        group.primary.display_source.as_deref().unwrap_or("-"),
        group.primary.remote_addr.as_deref().unwrap_or("-"),
        group.primary.original_path.as_deref().unwrap_or("-"),
        group.primary.virtual_display_path.as_deref().unwrap_or("-"),
        duplicate_label(&group.primary),
        primary_location
            .map(StoredObjectLocation::kind)
            .unwrap_or("-"),
        primary_location
            .map(StoredObjectLocation::display_label)
            .unwrap_or_else(|| "-".to_string())
    )
}

fn duplicate_label(asset: &ReceivedAsset) -> String {
    match (asset.duplicate_index, asset.duplicate_count) {
        (Some(index), Some(count)) => format!("{index}/{count}"),
        _ => "-".to_string(),
    }
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
    let format = match value.to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" => ObjectFormat::Jpeg,
        "nef" => ObjectFormat::Nef,
        "nrw" => ObjectFormat::Nrw,
        "cr2" => ObjectFormat::Cr2,
        "cr3" => ObjectFormat::Cr3,
        "arw" | "srf" | "sr2" => ObjectFormat::Arw,
        "raf" => ObjectFormat::Raf,
        "rw2" | "rwl" => ObjectFormat::Rw2,
        "orf" => ObjectFormat::Orf,
        "pef" => ObjectFormat::Pef,
        "dng" => ObjectFormat::Dng,
        "mov" => ObjectFormat::Mov,
        "mp4" => ObjectFormat::Mp4,
        "tif" | "tiff" => ObjectFormat::Tiff,
        "unknown" => ObjectFormat::Unknown,
        _ => return Err(camera_connector_core::ImporterError::InvalidUploadPath),
    };
    Ok(format)
}

fn handle_account_command(config_path: Option<&Path>, action: AccountCommand) -> Result<()> {
    let service = CameraConnectorService::new(config_path.map(Path::to_path_buf));
    match action {
        AccountCommand::List => {
            let config = service.load_config()?;
            println!("config: {}", service.config_path().display());
            if config.accounts.is_empty() {
                println!("accounts: -");
            } else {
                for account in config.accounts.values() {
                    println!(
                        "{}\tdevice={}\tpassword={}",
                        account.username,
                        account.device_name,
                        if account.password_configured() {
                            "configured"
                        } else {
                            "not required"
                        }
                    );
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
            let project = service.ensure_active_project()?;
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use camera_connector_core::{
        CameraConnectorConfig, ReceiverAccountConfig, StoredObjectLocation,
    };

    #[test]
    fn account_config_round_trips() {
        let path = unique_temp_config_path("round-trip");
        let mut config = CameraConnectorConfig::default();
        config.accounts.insert(
            "z5".to_string(),
            ReceiverAccountConfig::new("z5", Some("secret"), "Z5_2").expect("account should build"),
        );

        config.save(Some(&path)).expect("config saves");
        let raw = std::fs::read_to_string(&path).expect("config should read");
        assert!(!raw.contains("secret"));
        assert!(raw.contains("password_hash"));
        let loaded = CameraConnectorConfig::load(Some(&path)).expect("config loads");

        let account = loaded.accounts.get("z5").expect("account exists");
        assert!(account.password_hash.is_some());
        assert_eq!(account.device_name, "Z5_2");
        assert!(account
            .clone()
            .into_receiver_account()
            .password
            .as_ref()
            .expect("password should exist")
            .verify("secret")
            .expect("password should verify"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn configured_accounts_build_receiver_accounts() {
        let path = unique_temp_config_path("accounts");
        let mut config = CameraConnectorConfig::default();
        config.accounts.insert(
            "z5".to_string(),
            ReceiverAccountConfig::new("z5", Some("secret"), "Z5_2").expect("account should build"),
        );
        config.save(Some(&path)).expect("config saves");

        let accounts = CameraConnectorConfig::load(Some(&path))
            .expect("config loads")
            .effective_accounts(None, None, None)
            .expect("accounts should load from config");

        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].username, "z5");
        assert!(accounts[0]
            .password
            .as_ref()
            .expect("password should exist")
            .verify("secret")
            .expect("password should verify"));
        assert_eq!(accounts[0].device_name, "Z5_2");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn account_set_rejects_blank_identity() {
        let path = unique_temp_config_path("blank-account");

        let result = handle_account_command(
            Some(&path),
            AccountCommand::Set {
                username: "  ".to_string(),
                password: Some("secret".to_string()),
                device_name: "Z5_2".to_string(),
            },
        );

        assert!(result.is_err());
        assert!(!path.exists());
    }

    #[test]
    fn account_set_rejects_blank_device_name() {
        let path = unique_temp_config_path("blank-device-name");

        let result = handle_account_command(
            Some(&path),
            AccountCommand::Set {
                username: "z5".to_string(),
                password: Some("secret".to_string()),
                device_name: " ".to_string(),
            },
        );

        assert!(result.is_err());
        assert!(!path.exists());
    }

    #[test]
    fn parse_source_rejects_ftps() {
        let result = parse_source("ftps");

        assert!(result.is_err());
    }

    #[test]
    fn parses_serve_sftp_command() {
        let cli = Cli::try_parse_from([
            "camera-connector",
            "serve-sftp",
            "--output",
            "C:\\CameraConnector",
        ])
        .expect("serve-sftp command should parse");

        assert!(matches!(cli.command, Some(Command::ServeSftp { .. })));
    }

    #[test]
    fn receiver_settings_command_updates_config() {
        let path = unique_temp_config_path("receiver-settings");

        handle_receiver_settings_command(
            Some(&path),
            ReceiverSettingsArgs {
                protocol: Some(PushProtocol::Sftp),
                bind_host: Some("127.0.0.1".to_string()),
                ftp_port: Some(2122),
                sftp_port: Some(2223),
                output: Some(PathBuf::from("C:\\CameraConnector\\Inbox")),
                state: Some(PathBuf::from("C:\\CameraConnector\\State")),
                advertised_host: Some("192.168.137.1".to_string()),
                source_name: Some("Studio".to_string()),
            },
        )
        .expect("receiver settings command should save");

        let loaded = CameraConnectorConfig::load(Some(&path)).expect("config loads");
        assert_eq!(loaded.receiver.protocol, PushProtocol::Sftp);
        assert_eq!(loaded.receiver.bind_host, "127.0.0.1");
        assert_eq!(loaded.receiver.ftp_port, 2122);
        assert_eq!(loaded.receiver.sftp_port, 2223);
        assert_eq!(
            loaded.receiver.output_dir.as_deref(),
            Some(Path::new("C:\\CameraConnector\\Inbox"))
        );
        assert_eq!(
            loaded.receiver.state_dir.as_deref(),
            Some(Path::new("C:\\CameraConnector\\State"))
        );
        assert_eq!(
            loaded.receiver.advertised_host.as_deref(),
            Some("192.168.137.1")
        );
        assert_eq!(loaded.receiver.source_name.as_deref(), Some("Studio"));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn parses_inbox_from_transfers_command() {
        let cli = Cli::try_parse_from([
            "camera-connector",
            "inbox",
            "--config",
            "C:\\CameraConnector\\config.json",
            "--path",
            "C:\\CameraConnector\\state",
            "--from-transfers",
            "--summary",
            "--username",
            "z5",
            "--source-name",
            "Z5_2",
            "--original-path",
            "DCIM",
            "--remote-addr",
            "192.168.137.56",
            "--format",
            "nef",
            "--offset",
            "1",
            "--limit",
            "20",
        ])
        .expect("inbox from transfers command should parse");

        assert!(matches!(
            cli.command,
            Some(Command::Inbox {
                from_transfers: true,
                summary: true,
                username: Some(_),
                source_name: Some(_),
                original_path: Some(_),
                remote_addr: Some(_),
                format: Some(_),
                offset: 1,
                limit: Some(20),
                ..
            })
        ));
    }

    #[test]
    fn parses_transfers_status_filter_command() {
        let cli = Cli::try_parse_from([
            "camera-connector",
            "transfers",
            "--state",
            "C:\\CameraConnector\\state",
            "--status",
            "failed",
        ])
        .expect("transfers status command should parse");

        assert!(matches!(
            cli.command,
            Some(Command::Transfers {
                status: Some(_),
                ..
            })
        ));
    }

    #[test]
    fn parses_dashboard_command() {
        let cli = Cli::try_parse_from([
            "camera-connector",
            "dashboard",
            "--config",
            "C:\\CameraConnector\\config.json",
            "--state",
            "C:\\CameraConnector\\state",
            "--username",
            "z5",
            "--online-devices",
            "--json",
            "--offset",
            "0",
            "--limit",
            "25",
        ])
        .expect("dashboard command should parse");

        assert!(matches!(
            cli.command,
            Some(Command::Dashboard {
                username: Some(_),
                online_devices: true,
                json: true,
                offset: 0,
                limit: 25,
                ..
            })
        ));
    }

    #[test]
    fn parses_project_dashboard_command_without_state() {
        let cli = Cli::try_parse_from([
            "camera-connector",
            "dashboard",
            "--config",
            "C:\\CameraConnector\\config.json",
            "--project-id",
            "project-1",
            "--json",
        ])
        .expect("project dashboard command should parse");

        assert!(matches!(
            cli.command,
            Some(Command::Dashboard {
                state: None,
                project_id: Some(project_id),
                json: true,
                ..
            }) if project_id == "project-1"
        ));
    }

    #[test]
    fn dashboard_command_loads_project_dashboard_from_sqlite() {
        let root = std::env::temp_dir().join(format!(
            "camera-connector-project-dashboard-{}",
            current_time_ms()
        ));
        let config_path = root.join("config.json");
        let state_dir = root.join("state");
        std::fs::create_dir_all(&root).expect("temp root should create");
        let service = CameraConnectorService::new(Some(config_path.clone()));
        service
            .set_receiver_settings(ReceiverSettingsUpdate {
                state_dir: Some(state_dir.clone()),
                ..ReceiverSettingsUpdate::default()
            })
            .expect("receiver settings should save");
        let project = service
            .create_project("CLI Dashboard")
            .expect("project should create");
        service
            .record_project_transfer(
                &project.project_id,
                TransferRecord {
                    transfer_id: "ftp:cli-dashboard".to_string(),
                    protocol: "ftp".to_string(),
                    status: TransferStatus::Completed,
                    original_path: "DCIM/100/IMG_0101.CR3".to_string(),
                    final_filename: "IMG_0101.CR3".to_string(),
                    final_path: None,
                    final_location: Some(StoredObjectLocation::local_path(
                        root.join("IMG_0101.CR3"),
                    )),
                    size_bytes: 42,
                    username: Some("verify".to_string()),
                    remote_addr: None,
                    source_name: Some("Verify Camera".to_string()),
                    started_at_ms: 10,
                    completed_at_ms: Some(20),
                    error: None,
                },
            )
            .expect("project transfer should record");

        let dashboard = load_dashboard(DashboardArgs {
            config: Some(config_path.clone()),
            state: None,
            project_id: Some(project.project_id),
            query: AssetGroupQuery::default(),
            offset: 0,
            limit: 50,
            online_devices: false,
        })
        .expect("project dashboard should load");

        assert_eq!(dashboard.paths.state_dir, state_dir);
        assert_eq!(dashboard.assets.total_groups, 1);
        assert_eq!(dashboard.assets.summary.asset_count, 1);
        assert_eq!(
            dashboard.assets.groups[0].primary.display_source.as_deref(),
            Some("Verify Camera")
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn parses_project_create_command() {
        let cli = Cli::try_parse_from([
            "camera-connector",
            "project",
            "--config",
            "C:\\CameraConnector\\config.json",
            "create",
            "--name",
            "Verify Shoot",
        ])
        .expect("project create command should parse");

        assert!(matches!(
            cli.command,
            Some(Command::Project {
                config: Some(_),
                action: ProjectCommand::Create { name },
            }) if name == "Verify Shoot"
        ));
    }

    #[test]
    fn project_line_marks_active_project() {
        let project = camera_connector_core::Project {
            project_id: "project-1".to_string(),
            name: "Verify Shoot".to_string(),
            slug: "verify-shoot".to_string(),
            status: camera_connector_core::ProjectStatus::Active,
            created_at_ms: 10,
            updated_at_ms: 20,
            archived_at_ms: None,
            default_output_target_id: None,
            default_strategy_profile_id: None,
        };

        let line = project_line(&project, Some("project-1"));

        assert!(line.contains("project\tid=project-1"));
        assert!(line.contains("name=Verify Shoot"));
        assert!(line.contains("slug=verify-shoot"));
        assert!(line.contains("status=active"));
        assert!(line.contains("active=true"));
    }

    #[test]
    fn receive_file_command_indexes_upload_under_active_project() {
        let root = std::env::temp_dir().join(format!(
            "camera-connector-receive-file-{}",
            current_time_ms()
        ));
        let input = root.join("IMG_0001.CR3");
        let output = root.join("output");
        let state = root.join("state");
        std::fs::create_dir_all(&root).expect("temp root should create");
        std::fs::write(&input, [1_u8, 2, 3, 4]).expect("sample should write");

        let store = camera_connector_core::SqliteStore::open_state_dir(&state)
            .expect("storage should open");
        let project = store
            .create_project("CLI Shoot")
            .expect("project should create");
        store
            .set_active_project(&project.project_id)
            .expect("project should become active");

        let record = handle_receive_file_command(ReceiveFileArgs {
            input,
            output,
            state: Some(state.clone()),
            source: ImportSource::FtpPush,
            username: Some("verify".to_string()),
            source_name: Some("Verify Camera".to_string()),
        })
        .expect("receive-file should index upload");

        let page = store
            .asset_group_page(&project.project_id, AssetGroupQuery::default(), 0, 50)
            .expect("project assets should load");

        assert_eq!(record.final_filename, "IMG_0001.CR3");
        assert_eq!(page.summary.asset_count, 1);
        assert_eq!(page.groups[0].primary.filename, "IMG_0001.CR3");
        assert_eq!(page.groups[0].primary.username.as_deref(), Some("verify"));
        assert_eq!(
            page.groups[0].primary.display_source.as_deref(),
            Some("Verify Camera")
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn dashboard_json_output_contains_status_devices_and_assets() {
        let asset = ReceivedAsset::new("ftp:1", "IMG_0001.CR3", 42, ImportSource::FtpPush);
        let dashboard = CameraConnectorDashboard {
            receiver_status: Some(ReceiverRuntimeStatus {
                phase: camera_connector_core::ReceiverRuntimePhase::Stopped,
                protocol: Some(PushProtocol::Ftp),
                auth_mode: camera_connector_core::ReceiverAuthMode::Accounts,
                local_addr: None,
                output_dir: None,
                state_dir: None,
                account_count: 1,
                message: None,
            }),
            receiver_settings: ReceiverSettingsConfig::default(),
            paths: camera_connector_core::SystemPathsView {
                config_path: PathBuf::from("C:\\CameraConnector\\config.json"),
                state_dir: PathBuf::from("C:\\CameraConnector\\state"),
                output_dir: None,
            },
            accounts: vec![camera_connector_core::AccountView {
                username: "z5".to_string(),
                device_name: "Camera".to_string(),
                password_configured: true,
                online: true,
                active_connections: 1,
                last_remote_addr: Some("192.168.137.56".to_string()),
                last_remote_port: Some(50123),
                last_seen_at_ms: Some(20),
                last_disconnected_at_ms: None,
            }],
            transfers: camera_connector_core::TransferSummary {
                total_count: 2,
                completed_count: 1,
                failed_count: 1,
            },
            recent_failures: vec![TransferRecordView {
                record: TransferRecord {
                    transfer_id: "ftp:failed".to_string(),
                    protocol: "ftp".to_string(),
                    status: TransferStatus::Failed,
                    original_path: "IMG_0002.CR3".to_string(),
                    final_filename: "IMG_0002.CR3".to_string(),
                    final_path: None,
                    final_location: None,
                    size_bytes: 0,
                    username: Some("z5".to_string()),
                    remote_addr: Some("192.168.137.56".to_string()),
                    source_name: Some("Camera".to_string()),
                    started_at_ms: 11,
                    completed_at_ms: Some(21),
                    error: Some("connection reset".to_string()),
                },
                display_source: Some("Camera".to_string()),
                virtual_display_path: "Camera/IMG_0002.CR3".to_string(),
                final_location_kind: None,
                final_location_label: None,
            }],
            devices: vec![camera_connector_core::ConnectedDeviceView {
                device: camera_connector_core::ConnectedDevice {
                    remote_addr: "192.168.137.56".to_string(),
                    source_name: Some("Camera".to_string()),
                    username: Some("z5".to_string()),
                    online: true,
                    last_seen_at_ms: 20,
                    first_seen_at_ms: 10,
                    last_disconnected_at_ms: None,
                    active_connections: 1,
                    last_remote_port: Some(50123),
                },
                display_source: "Camera".to_string(),
            }],
            assets: AssetGroupPage {
                groups: vec![ReceivedAssetGroup {
                    group_key: "IMG_0001".to_string(),
                    primary: asset.clone(),
                    jpeg: None,
                    raw: Some(asset),
                    video: None,
                }],
                summary: AssetGroupSummary {
                    group_count: 1,
                    asset_count: 1,
                    groups_with_jpeg: 0,
                    groups_with_raw: 1,
                    groups_with_video: 0,
                    source_counts: Vec::new(),
                    remote_addr_counts: Vec::new(),
                },
                offset: 0,
                limit: 50,
                total_groups: 1,
                has_more: false,
            },
        };

        let json = dashboard_json(&dashboard).expect("dashboard should serialize");

        assert!(json.contains("\"receiver_status\""));
        assert!(json.contains("\"paths\""));
        assert!(json.contains("\"config_path\""));
        assert!(json.contains("\"state_dir\""));
        assert!(json.contains("\"accounts\""));
        assert!(json.contains("\"password_configured\": true"));
        assert!(json.contains("\"online\": true"));
        assert!(json.contains("\"last_remote_addr\": \"192.168.137.56\""));
        assert!(!json.contains("password_hash"));
        assert!(json.contains("\"transfers\""));
        assert!(json.contains("\"failed_count\": 1"));
        assert!(json.contains("\"recent_failures\""));
        assert!(json.contains("\"connection reset\""));
        assert!(json.contains("\"devices\""));
        assert!(json.contains("\"assets\""));
        assert!(json.contains("\"group_key\": \"IMG_0001\""));
    }

    #[test]
    fn transfer_view_line_prints_platform_location() {
        let view = TransferRecordView {
            record: TransferRecord {
                transfer_id: "sftp:1".to_string(),
                protocol: "sftp".to_string(),
                status: TransferStatus::Completed,
                original_path: "DCIM/IMG_0001.DNG".to_string(),
                final_filename: "IMG_0001.DNG".to_string(),
                final_path: None,
                final_location: Some(StoredObjectLocation::document_uri(
                    "content://camera-connector/IMG_0001.DNG",
                )),
                size_bytes: 42,
                username: None,
                remote_addr: Some("192.168.137.56".to_string()),
                source_name: Some("Camera".to_string()),
                started_at_ms: 10,
                completed_at_ms: Some(20),
                error: None,
            },
            display_source: Some("Camera".to_string()),
            virtual_display_path: "Camera/DCIM/IMG_0001.DNG".to_string(),
            final_location_kind: Some("document_uri".to_string()),
            final_location_label: Some("content://camera-connector/IMG_0001.DNG".to_string()),
        };

        let line = transfer_view_line(&view);

        assert!(line.contains("username=-"));
        assert!(line.contains("error=-"));
        assert!(line.contains("location_kind=document_uri"));
        assert!(line.contains("location=content://camera-connector/IMG_0001.DNG"));
    }

    #[test]
    fn transfer_view_line_prints_failure_error() {
        let view = TransferRecordView {
            record: TransferRecord {
                transfer_id: "ftp:failed".to_string(),
                protocol: "ftp".to_string(),
                status: TransferStatus::Failed,
                original_path: "IMG_0002.CR3".to_string(),
                final_filename: "IMG_0002.CR3".to_string(),
                final_path: None,
                final_location: None,
                size_bytes: 0,
                username: Some("z5".to_string()),
                remote_addr: Some("192.168.137.56".to_string()),
                source_name: Some("Camera".to_string()),
                started_at_ms: 11,
                completed_at_ms: Some(21),
                error: Some("connection reset".to_string()),
            },
            display_source: Some("Camera".to_string()),
            virtual_display_path: "Camera/IMG_0002.CR3".to_string(),
            final_location_kind: None,
            final_location_label: None,
        };

        let line = transfer_view_line(&view);

        assert!(line.contains("Failed"));
        assert!(line.contains("error=connection reset"));
    }

    #[test]
    fn asset_group_line_prints_primary_storage_location() {
        let asset = ReceivedAsset::new("ftp:1", "IMG_0001.CR3", 42, ImportSource::FtpPush)
            .with_storage_location(StoredObjectLocation::document_uri(
                "content://camera-connector/IMG_0001.CR3",
            ));
        let mut asset = asset;
        asset.display_source = Some("Z5_2".to_string());
        asset.username = Some("z5".to_string());
        asset.remote_addr = Some("192.168.137.56".to_string());
        asset.original_path = Some("DCIM/IMG_0001.CR3".to_string());
        asset.virtual_display_path = Some("Z5_2/DCIM/IMG_0001.CR3".to_string());
        let group = ReceivedAssetGroup {
            group_key: "IMG_0001".to_string(),
            primary: asset.clone(),
            jpeg: None,
            raw: Some(asset),
            video: None,
        };

        let line = asset_group_line(&group);

        assert!(line.contains("primary_location_kind=document_uri"));
        assert!(line.contains("primary_location=content://camera-connector/IMG_0001.CR3"));
        assert!(line.contains("username=z5"));
        assert!(line.contains("source=Z5_2"));
        assert!(line.contains("remote=192.168.137.56"));
        assert!(line.contains("original=DCIM/IMG_0001.CR3"));
        assert!(line.contains("display=Z5_2/DCIM/IMG_0001.CR3"));
        assert!(line.contains("duplicate=-"));
    }

    #[test]
    fn asset_group_summary_line_prints_filter_counts() {
        let summary = AssetGroupSummary {
            group_count: 2,
            asset_count: 3,
            groups_with_jpeg: 1,
            groups_with_raw: 1,
            groups_with_video: 1,
            source_counts: vec![AssetFacetCount {
                value: "Z5_2".to_string(),
                group_count: 2,
            }],
            remote_addr_counts: vec![AssetFacetCount {
                value: "192.168.137.56".to_string(),
                group_count: 2,
            }],
        };

        let line = asset_group_summary_line(&summary);

        assert!(line.contains("groups=2"));
        assert!(line.contains("raw_groups=1"));
        assert!(line.contains("sources=Z5_2:2"));
        assert!(line.contains("remotes=192.168.137.56:2"));
    }

    #[test]
    fn asset_group_page_summary_line_prints_paging_state() {
        let page = AssetGroupPage {
            groups: Vec::new(),
            summary: AssetGroupSummary {
                group_count: 3,
                asset_count: 4,
                groups_with_jpeg: 1,
                groups_with_raw: 2,
                groups_with_video: 1,
                source_counts: Vec::new(),
                remote_addr_counts: Vec::new(),
            },
            offset: 1,
            limit: 1,
            total_groups: 3,
            has_more: true,
        };

        let line = asset_group_page_summary_line(&page);

        assert!(line.contains("groups=3"));
        assert!(line.contains("offset=1"));
        assert!(line.contains("limit=1"));
        assert!(line.contains("total_groups=3"));
        assert!(line.contains("has_more=true"));
    }

    fn unique_temp_config_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "camera-connector-{name}-{}.json",
            current_time_ms()
        ))
    }
}

fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}
