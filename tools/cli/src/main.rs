use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use camera_connector_core::{
    append_transfer_record, CameraConnectorRuntime, CameraConnectorService, ImportSource,
    LocalFileSink, PushProtocol, PushReceiverConfig, ReceivedAsset, ReceivedAssetGroup,
    ReceiverConfigRequest, Result, StoredObjectLocation, TransferQuery, TransferRecord,
    TransferRecordView, TransferStatus,
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
    ReceiverStatus {
        #[arg(long, alias = "path")]
        state: PathBuf,
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
        path: PathBuf,
        #[arg(long, default_value = "ftp")]
        source: String,
        #[arg(long)]
        from_transfers: bool,
    },
    Transfers {
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, alias = "path")]
        state: PathBuf,
        #[arg(long)]
        transfer_id: Option<String>,
        #[arg(long)]
        original_path: Option<String>,
        #[arg(long)]
        final_filename: Option<String>,
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
            source_name,
        }) => {
            let source = parse_source(&source)?;
            let filename = input
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or(camera_connector_core::ImporterError::InvalidUploadPath)?;
            let bytes = fs::read(&input)?;
            let started_at_ms = current_time_ms();
            let protocol_label = source_protocol_label(source);
            let progress = LocalFileSink::new(output).write_complete(
                format!("{protocol_label}:{started_at_ms}:{filename}"),
                filename,
                &bytes,
            )?;
            let asset = ReceivedAsset::new(
                progress.transfer_id,
                progress.filename,
                progress.bytes_written,
                source,
            );
            println!(
                "received {}\t{:?}\t{} bytes",
                asset.filename, asset.format, asset.size_bytes
            );
            let final_path = progress.output_path.clone().ok_or_else(|| {
                camera_connector_core::ImporterError::internal("missing output path")
            })?;
            let log_dir = state.unwrap_or_else(|| {
                final_path
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .to_path_buf()
            });
            append_transfer_record(
                &log_dir,
                &TransferRecord {
                    transfer_id: asset.id,
                    protocol: protocol_label.to_string(),
                    status: TransferStatus::Completed,
                    original_path: filename.to_string(),
                    final_filename: asset.filename,
                    final_path: Some(final_path),
                    final_location: progress.output_location,
                    size_bytes: progress.bytes_written,
                    remote_addr: None,
                    source_name,
                    started_at_ms,
                    completed_at_ms: Some(current_time_ms()),
                    error: None,
                },
            )?;
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
        Some(Command::ReceiverStatus { state }) => {
            let service = CameraConnectorService::new(None);
            match service.receiver_status(state)? {
                Some(status) => {
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
                None => {
                    println!("phase: Unknown");
                    println!("message: receiver status file not found");
                }
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
                protocol: PushProtocol::Ftp,
                bind_host,
                port,
                output_dir: output,
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
                protocol: PushProtocol::Sftp,
                bind_host,
                port,
                output_dir: output,
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
            path,
            source,
            from_transfers,
        }) => {
            let source = parse_source(&source)?;
            let service = CameraConnectorService::new(None);
            let groups = if from_transfers {
                service.transfer_asset_groups(path)?
            } else {
                service.inbox_groups(path, source)?
            };
            print_asset_groups(groups);
        }
        Some(Command::Transfers {
            config,
            state,
            transfer_id,
            original_path,
            final_filename,
            source_name,
            remote_addr,
        }) => {
            let service = CameraConnectorService::new(config);
            for view in service.transfers(
                state,
                TransferQuery {
                    transfer_id,
                    original_path,
                    final_filename,
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
        "{}\t{:?}\t{}\t{}\t{}\tremote={}\tsource={}\tdisplay={}\tlocation_kind={}\tlocation={}",
        record.transfer_id,
        record.status,
        record.final_filename,
        record.original_path,
        record.size_bytes,
        record.remote_addr.as_deref().unwrap_or("-"),
        view.display_source.as_deref().unwrap_or("-"),
        view.virtual_display_path,
        view.final_location_kind.as_deref().unwrap_or("-"),
        view.final_location_label.as_deref().unwrap_or("-")
    )
}

fn print_asset_groups(groups: Vec<ReceivedAssetGroup>) {
    for group in groups {
        println!("{}", asset_group_line(&group));
    }
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
        "{}\tprimary={}\tjpeg={}\traw={}\tvideo={}\t{} bytes\tprimary_location_kind={}\tprimary_location={}",
        group.group_key,
        group.primary.filename,
        jpeg,
        raw,
        video,
        total_bytes,
        primary_location
            .map(StoredObjectLocation::kind)
            .unwrap_or("-"),
        primary_location
            .map(StoredObjectLocation::display_label)
            .unwrap_or_else(|| "-".to_string())
    )
}

fn build_config(args: ConfigArgs) -> Result<PushReceiverConfig> {
    CameraConnectorService::new(args.config_path).receiver_config(ReceiverConfigRequest {
        protocol: args.protocol,
        bind_host: args.bind_host,
        port: args.port,
        output_dir: args.output,
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
    fn parses_inbox_from_transfers_command() {
        let cli = Cli::try_parse_from([
            "camera-connector",
            "inbox",
            "--path",
            "C:\\CameraConnector\\state",
            "--from-transfers",
        ])
        .expect("inbox from transfers command should parse");

        assert!(matches!(
            cli.command,
            Some(Command::Inbox {
                from_transfers: true,
                ..
            })
        ));
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

        assert!(line.contains("location_kind=document_uri"));
        assert!(line.contains("location=content://camera-connector/IMG_0001.DNG"));
    }

    #[test]
    fn asset_group_line_prints_primary_storage_location() {
        let asset = ReceivedAsset::new("ftp:1", "IMG_0001.CR3", 42, ImportSource::FtpPush)
            .with_storage_location(StoredObjectLocation::document_uri(
                "content://camera-connector/IMG_0001.CR3",
            ));
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
