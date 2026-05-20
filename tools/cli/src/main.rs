use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use camera_connector_core::{
    append_transfer_record, read_transfer_log, scan_inbox_groups, FtpPushServer, ImportSource,
    LocalFileSink, PushProtocol, PushReceiverConfig, ReceivedAsset, Result, TransferRecord,
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
        #[arg(long, default_value = "manual")]
        source: String,
        #[arg(long)]
        source_name: Option<String>,
    },
    ReceiverConfig {
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
        #[arg(long = "source-alias")]
        source_aliases: Vec<String>,
    },
    ServeFtp {
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
        #[arg(long = "source-alias")]
        source_aliases: Vec<String>,
    },
    Inbox {
        #[arg(long)]
        path: PathBuf,
        #[arg(long, default_value = "ftp")]
        source: String,
    },
    Transfers {
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
        #[arg(long = "source-alias")]
        source_aliases: Vec<String>,
    },
}

struct ConfigArgs {
    protocol: PushProtocol,
    bind_host: String,
    port: u16,
    output: PathBuf,
    username: Option<String>,
    password: Option<String>,
    advertised_host: Option<String>,
    source_name: Option<String>,
    source_aliases: Vec<String>,
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
            protocol,
            bind_host,
            port,
            output,
            username,
            advertised_host,
            source_name,
            source_aliases,
        }) => {
            let protocol = PushProtocol::from_str(&protocol)?;
            let config = build_config(ConfigArgs {
                protocol,
                bind_host,
                port,
                output,
                username,
                password: None,
                advertised_host,
                source_name,
                source_aliases,
            })?;
            print_receiver_config(&config);
        }
        Some(Command::ServeFtp {
            bind_host,
            port,
            output,
            username,
            password,
            advertised_host,
            source_name,
            source_aliases,
        }) => {
            let config = build_config(ConfigArgs {
                protocol: PushProtocol::Ftp,
                bind_host,
                port,
                output,
                username,
                password,
                advertised_host,
                source_name,
                source_aliases,
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
            path,
            transfer_id,
            original_path,
            final_filename,
            source_name,
            remote_addr,
            source_aliases,
        }) => {
            let source_aliases = parse_source_aliases(&source_aliases)?;
            for record in read_transfer_log(path)?
                .into_iter()
                .filter(|record| {
                    source_name
                        .as_ref()
                        .map(|expected| {
                            record_display_source(record, &source_aliases).as_ref()
                                == Some(expected)
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
                    record_display_source(&record, &source_aliases)
                        .as_deref()
                        .unwrap_or("-"),
                    record.virtual_display_path(
                        record_display_source(&record, &source_aliases).as_deref()
                    )
                );
            }
        }
    }

    Ok(())
}

fn build_config(args: ConfigArgs) -> Result<PushReceiverConfig> {
    let mut config = PushReceiverConfig::new(args.protocol, args.bind_host, args.port, args.output);
    config.username = args.username;
    config.password = args.password;
    config.advertised_host = args.advertised_host;
    config.source_name = args.source_name;
    config.source_aliases = parse_source_aliases(&args.source_aliases)?;
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
    println!(
        "username: {}",
        config.username.as_deref().unwrap_or("anonymous")
    );
    println!(
        "password: {}",
        if config.password.is_some() {
            "configured"
        } else {
            "not required"
        }
    );
    println!(
        "source_name: {}",
        config.source_name.as_deref().unwrap_or("-")
    );
    if config.source_aliases.is_empty() {
        println!("source_aliases: -");
    } else {
        let aliases = config
            .source_aliases
            .iter()
            .map(|(remote_addr, source_name)| format!("{remote_addr}={source_name}"))
            .collect::<Vec<_>>()
            .join(", ");
        println!("source_aliases: {aliases}");
    }
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

fn parse_source_aliases(values: &[String]) -> Result<BTreeMap<String, String>> {
    let mut aliases = BTreeMap::new();
    for alias in values {
        let (remote_addr, source_name) = alias.split_once('=').ok_or_else(|| {
            camera_connector_core::ImporterError::internal(format!(
                "invalid source alias '{alias}', expected IP=Name"
            ))
        })?;
        aliases.insert(
            remote_addr.trim().to_string(),
            source_name.trim().to_string(),
        );
    }
    Ok(aliases)
}

fn record_display_source(
    record: &TransferRecord,
    source_aliases: &BTreeMap<String, String>,
) -> Option<String> {
    record
        .remote_addr
        .as_deref()
        .and_then(|remote_addr| source_aliases.get(remote_addr).cloned())
        .or_else(|| record.source_name.clone())
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
    fn display_source_alias_applies_to_existing_transfer_record() {
        let aliases =
            parse_source_aliases(&["192.168.137.56=Z5_2".to_string()]).expect("alias parses");
        let record = TransferRecord {
            transfer_id: "ftp:1".to_string(),
            protocol: "ftp".to_string(),
            status: TransferStatus::Completed,
            original_path: "BB/DSC_2552.NEF".to_string(),
            final_filename: "DSC_2552.NEF".to_string(),
            final_path: PathBuf::from("DSC_2552.NEF"),
            size_bytes: 42,
            remote_addr: Some("192.168.137.56".to_string()),
            source_name: None,
            started_at_ms: 10,
            completed_at_ms: Some(20),
            error: None,
        };
        let source = record_display_source(&record, &aliases);

        assert_eq!(source.as_deref(), Some("Z5_2"));
        assert_eq!(
            record.virtual_display_path(source.as_deref()),
            "Z5_2/BB/DSC_2552.NEF"
        );
    }
}

fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}
