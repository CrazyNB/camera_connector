use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use camera_connector_core::{
    append_transfer_record, read_connected_devices, read_transfer_log, scan_inbox_groups,
    ConnectedDevice, FtpPushServer, ImportSource, LocalFileSink, PushProtocol, PushReceiverConfig,
    ReceivedAsset, ReceiverAccount, Result, TransferRecord, TransferStatus,
};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

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
        username: Option<String>,
        #[arg(long)]
        advertised_host: Option<String>,
        #[arg(long)]
        source_name: Option<String>,
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
    },
    Transfers {
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        path: PathBuf,
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
        #[arg(long)]
        path: PathBuf,
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
    username: Option<String>,
    password: Option<String>,
    advertised_host: Option<String>,
    source_name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct AppConfig {
    #[serde(default)]
    accounts: BTreeMap<String, CameraAccountConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CameraAccountConfig {
    username: String,
    password: Option<String>,
    device_name: String,
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
            let final_path = progress.output_path.ok_or_else(|| {
                camera_connector_core::ImporterError::internal("missing output path")
            })?;
            let log_dir = final_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();
            append_transfer_record(
                &log_dir,
                &TransferRecord {
                    transfer_id: asset.id,
                    protocol: protocol_label.to_string(),
                    status: TransferStatus::Completed,
                    original_path: filename.to_string(),
                    final_filename: asset.filename,
                    final_path,
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
                username,
                password: None,
                advertised_host,
                source_name,
            })?;
            print_receiver_config(&config);
        }
        Some(Command::ServeFtp {
            config,
            bind_host,
            port,
            output,
            username,
            password,
            advertised_host,
            source_name,
        }) => {
            let config = build_config(ConfigArgs {
                config_path: config,
                protocol: PushProtocol::Ftp,
                bind_host,
                port,
                output,
                username,
                password,
                advertised_host,
                source_name,
            })?;
            let server = FtpPushServer::bind(config.clone()).await?;
            println!("ftp receiver listening on {}", server.local_addr());
            print_receiver_config(&config);
            server.run().await?;
        }
        Some(Command::Inbox { path, source }) => {
            let source = parse_source(&source)?;
            let groups = scan_inbox_groups(path, source)?;
            for group in groups {
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

                println!(
                    "{}\tprimary={}\tjpeg={}\traw={}\tvideo={}\t{} bytes",
                    group.group_key, group.primary.filename, jpeg, raw, video, total_bytes
                );
            }
        }
        Some(Command::Transfers {
            config,
            path,
            transfer_id,
            original_path,
            final_filename,
            source_name,
            remote_addr,
        }) => {
            let accounts = load_app_config(config.as_deref())?.accounts;
            for record in read_transfer_log(path)?
                .into_iter()
                .filter(|record| {
                    source_name
                        .as_ref()
                        .map(|expected| {
                            record_display_source(record, &accounts).as_ref() == Some(expected)
                        })
                        .unwrap_or(true)
                })
                .filter(|record| {
                    remote_addr
                        .as_ref()
                        .map(|expected| record.remote_addr.as_ref() == Some(expected))
                        .unwrap_or(true)
                })
                .filter(|record| {
                    transfer_id
                        .as_ref()
                        .map(|expected| record.transfer_id.contains(expected))
                        .unwrap_or(true)
                })
                .filter(|record| {
                    original_path
                        .as_ref()
                        .map(|expected| {
                            record
                                .original_path
                                .to_ascii_lowercase()
                                .contains(&expected.to_ascii_lowercase())
                        })
                        .unwrap_or(true)
                })
                .filter(|record| {
                    final_filename
                        .as_ref()
                        .map(|expected| {
                            record
                                .final_filename
                                .to_ascii_lowercase()
                                .contains(&expected.to_ascii_lowercase())
                        })
                        .unwrap_or(true)
                })
            {
                println!(
                    "{}\t{:?}\t{}\t{}\t{}\tremote={}\tsource={}\tdisplay={}",
                    record.transfer_id,
                    record.status,
                    record.final_filename,
                    record.original_path,
                    record.size_bytes,
                    record.remote_addr.as_deref().unwrap_or("-"),
                    record_display_source(&record, &accounts)
                        .as_deref()
                        .unwrap_or("-"),
                    record
                        .virtual_display_path(record_display_source(&record, &accounts).as_deref())
                );
            }
        }
        Some(Command::Account { config, action }) => {
            handle_account_command(config.as_deref(), action)?;
        }
        Some(Command::Devices {
            config,
            path,
            username,
            online,
        }) => {
            let accounts = load_app_config(config.as_deref())?.accounts;
            for device in read_connected_devices(path)?
                .into_iter()
                .filter(|device| device_matches_filters(device, username.as_deref(), online))
            {
                let display = device_display_source(&device, &accounts)
                    .unwrap_or_else(|| remote_addr_display_label(&device.remote_addr));
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
                    device_display_source(&device, &accounts)
                        .as_deref()
                        .unwrap_or("-"),
                    display,
                    device.last_seen_at_ms
                );
            }
        }
    }

    Ok(())
}

fn device_matches_filters(device: &ConnectedDevice, username: Option<&str>, online: bool) -> bool {
    (!online || device.online)
        && username
            .map(|expected| device.username.as_deref() == Some(expected))
            .unwrap_or(true)
}

fn build_config(args: ConfigArgs) -> Result<PushReceiverConfig> {
    let mut config = PushReceiverConfig::new(args.protocol, args.bind_host, args.port, args.output);
    config.advertised_host = args.advertised_host;
    config.source_name = args.source_name;
    config.accounts = effective_accounts(
        args.config_path.as_deref(),
        args.username.as_deref(),
        args.password.as_deref(),
        config.source_name.as_deref(),
    )?;
    Ok(config)
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
        "ftps" | "ftps-push" => Ok(ImportSource::FtpsPush),
        "manual" | "manual-drop" => Ok(ImportSource::ManualDrop),
        _ => Err(camera_connector_core::ImporterError::UnsupportedProtocol),
    }
}

fn effective_accounts(
    config_path: Option<&Path>,
    username: Option<&str>,
    password: Option<&str>,
    device_name: Option<&str>,
) -> Result<Vec<ReceiverAccount>> {
    let mut accounts = load_app_config(config_path)?
        .accounts
        .into_values()
        .map(CameraAccountConfig::into_receiver_account)
        .collect::<Vec<_>>();

    if let Some(username) = username {
        let transient = validate_account_config(CameraAccountConfig {
            username: username.to_string(),
            password: password.map(ToOwned::to_owned),
            device_name: device_name.unwrap_or(username).to_string(),
        })?
        .into_receiver_account();
        accounts.retain(|account| account.username != username);
        accounts.push(transient);
    }

    Ok(accounts)
}

fn handle_account_command(config_path: Option<&Path>, action: AccountCommand) -> Result<()> {
    let mut config = load_app_config(config_path)?;
    match action {
        AccountCommand::List => {
            println!("config: {}", resolved_config_path(config_path).display());
            if config.accounts.is_empty() {
                println!("accounts: -");
            } else {
                for account in config.accounts.values() {
                    println!(
                        "{}\tdevice={}\tpassword={}",
                        account.username,
                        account.device_name,
                        if account.password.is_some() {
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
            let account = validate_account_config(CameraAccountConfig {
                username,
                password,
                device_name,
            })?;
            config
                .accounts
                .insert(account.username.clone(), account.clone());
            let path = save_app_config(config_path, &config)?;
            println!(
                "saved account {}\tdevice={}",
                account.username, account.device_name
            );
            println!("config: {}", path.display());
        }
        AccountCommand::Remove { username } => {
            let removed = config.accounts.remove(&username);
            let path = save_app_config(config_path, &config)?;
            println!(
                "{} {username}",
                if removed.is_some() {
                    "removed"
                } else {
                    "not_found"
                }
            );
            println!("config: {}", path.display());
        }
    }
    Ok(())
}

fn load_app_config(config_path: Option<&Path>) -> Result<AppConfig> {
    let path = resolved_config_path(config_path);
    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let bytes = fs::read(&path)?;
    let mut config: AppConfig = serde_json::from_slice(&bytes)
        .map_err(|error| camera_connector_core::ImporterError::internal(error.to_string()))?;
    config.accounts = config
        .accounts
        .into_values()
        .map(validate_account_config)
        .map(|result| result.map(|account| (account.username.clone(), account)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    Ok(config)
}

fn save_app_config(config_path: Option<&Path>, config: &AppConfig) -> Result<PathBuf> {
    let path = resolved_config_path(config_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_vec_pretty(config)
        .map_err(|error| camera_connector_core::ImporterError::internal(error.to_string()))?;
    fs::write(&path, json)?;
    Ok(path)
}

fn resolved_config_path(config_path: Option<&Path>) -> PathBuf {
    config_path
        .map(Path::to_path_buf)
        .unwrap_or_else(default_config_path)
}

fn default_config_path() -> PathBuf {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        return PathBuf::from(appdata)
            .join("CameraConnector")
            .join("config.json");
    }
    if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
        return PathBuf::from(home)
            .join(".camera-connector")
            .join("config.json");
    }
    PathBuf::from("camera-connector-config.json")
}

fn record_display_source(
    record: &TransferRecord,
    _accounts: &BTreeMap<String, CameraAccountConfig>,
) -> Option<String> {
    record.source_name.clone()
}

fn device_display_source(
    device: &ConnectedDevice,
    accounts: &BTreeMap<String, CameraAccountConfig>,
) -> Option<String> {
    device
        .username
        .as_deref()
        .and_then(|username| accounts.get(username))
        .map(|account| account.device_name.clone())
        .or_else(|| device.source_name.clone())
}

impl CameraAccountConfig {
    fn into_receiver_account(self) -> ReceiverAccount {
        ReceiverAccount {
            username: self.username,
            password: self.password,
            device_name: self.device_name,
        }
    }
}

fn validate_account_config(mut account: CameraAccountConfig) -> Result<CameraAccountConfig> {
    account.username = normalized_required("username", &account.username)?;
    account.device_name = normalized_required("device name", &account.device_name)?;
    account.clone().into_receiver_account().validate()?;
    Ok(account)
}

fn normalized_required(field: &str, value: &str) -> Result<String> {
    let normalized = value.trim().to_string();
    if normalized.is_empty() {
        return Err(camera_connector_core::ImporterError::internal(format!(
            "{field} cannot be empty"
        )));
    }
    Ok(normalized)
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

fn source_protocol_label(source: ImportSource) -> &'static str {
    match source {
        ImportSource::FtpPush => "ftp",
        ImportSource::SftpPush => "sftp",
        ImportSource::FtpsPush => "ftps",
        ImportSource::ManualDrop => "manual",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_display_source_uses_record_source_name_not_ip_binding() {
        let accounts = BTreeMap::new();
        let record = TransferRecord {
            transfer_id: "ftp:1".to_string(),
            protocol: "ftp".to_string(),
            status: TransferStatus::Completed,
            original_path: "BB/DSC_2552.NEF".to_string(),
            final_filename: "DSC_2552.NEF".to_string(),
            final_path: PathBuf::from("DSC_2552.NEF"),
            size_bytes: 42,
            remote_addr: Some("192.168.137.56".to_string()),
            source_name: Some("Z5_2".to_string()),
            started_at_ms: 10,
            completed_at_ms: Some(20),
            error: None,
        };
        let source = record_display_source(&record, &accounts);

        assert_eq!(source.as_deref(), Some("Z5_2"));
        assert_eq!(
            record.virtual_display_path(source.as_deref()),
            "Z5_2/BB/DSC_2552.NEF"
        );
    }

    #[test]
    fn account_config_round_trips() {
        let path = unique_temp_config_path("round-trip");
        let mut config = AppConfig::default();
        config.accounts.insert(
            "z5".to_string(),
            CameraAccountConfig {
                username: "z5".to_string(),
                password: Some("secret".to_string()),
                device_name: "Z5_2".to_string(),
            },
        );

        save_app_config(Some(&path), &config).expect("config saves");
        let loaded = load_app_config(Some(&path)).expect("config loads");

        let account = loaded.accounts.get("z5").expect("account exists");
        assert_eq!(account.password.as_deref(), Some("secret"));
        assert_eq!(account.device_name, "Z5_2");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn configured_accounts_build_receiver_accounts() {
        let path = unique_temp_config_path("accounts");
        let mut config = AppConfig::default();
        config.accounts.insert(
            "z5".to_string(),
            CameraAccountConfig {
                username: "z5".to_string(),
                password: Some("secret".to_string()),
                device_name: "Z5_2".to_string(),
            },
        );
        save_app_config(Some(&path), &config).expect("config saves");

        let accounts = effective_accounts(Some(&path), None, None, None)
            .expect("accounts should load from config");

        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].username, "z5");
        assert_eq!(accounts[0].password.as_deref(), Some("secret"));
        assert_eq!(accounts[0].device_name, "Z5_2");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn device_display_source_uses_authenticated_username() {
        let mut accounts = BTreeMap::new();
        accounts.insert(
            "z5".to_string(),
            CameraAccountConfig {
                username: "z5".to_string(),
                password: Some("secret".to_string()),
                device_name: "Z5_2".to_string(),
            },
        );
        let device = ConnectedDevice {
            remote_addr: "192.168.137.56".to_string(),
            source_name: None,
            username: Some("z5".to_string()),
            first_seen_at_ms: 10,
            last_seen_at_ms: 20,
            last_disconnected_at_ms: None,
            last_remote_port: Some(51120),
            active_connections: 1,
            online: true,
        };

        assert_eq!(
            device_display_source(&device, &accounts).as_deref(),
            Some("Z5_2")
        );
        assert_eq!(remote_addr_display_label(&device.remote_addr), "IP-056");
    }

    #[test]
    fn device_filter_matches_username_and_online_state() {
        let online_device = ConnectedDevice {
            remote_addr: "192.168.137.56".to_string(),
            source_name: Some("Z5_2".to_string()),
            username: Some("z5".to_string()),
            first_seen_at_ms: 10,
            last_seen_at_ms: 20,
            last_disconnected_at_ms: None,
            last_remote_port: Some(51120),
            active_connections: 1,
            online: true,
        };
        let offline_device = ConnectedDevice {
            online: false,
            active_connections: 0,
            last_disconnected_at_ms: Some(30),
            ..online_device.clone()
        };

        assert!(device_matches_filters(&online_device, Some("z5"), true));
        assert!(!device_matches_filters(&offline_device, Some("z5"), true));
        assert!(!device_matches_filters(&online_device, Some("xt5"), false));
        assert!(device_matches_filters(&offline_device, Some("z5"), false));
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
