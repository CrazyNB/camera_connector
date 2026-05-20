use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use clap::{Parser, Subcommand};
use nikon_importer_core::{
    scan_inbox_groups, FtpPushServer, ImportSource, LocalFileSink, PushProtocol,
    PushReceiverConfig, ReceivedAsset, Result,
};

#[derive(Debug, Parser)]
#[command(name = "nikon-importer")]
#[command(about = "Push-mode wireless import receiver for Nikon cameras")]
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
    },
    Inbox {
        #[arg(long)]
        path: PathBuf,
        #[arg(long, default_value = "ftp")]
        source: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Version) | None => {
            println!("nikon-importer {}", env!("CARGO_PKG_VERSION"));
        }
        Some(Command::ReceiveFile {
            input,
            output,
            source,
        }) => {
            let source = parse_source(&source)?;
            let filename = input
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or(nikon_importer_core::ImporterError::InvalidUploadPath)?;
            let bytes = fs::read(&input)?;
            let progress = LocalFileSink::new(output).write_complete(
                format!("{source:?}:{filename}"),
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
        }
        Some(Command::ReceiverConfig {
            protocol,
            bind_host,
            port,
            output,
            username,
            advertised_host,
        }) => {
            let protocol = PushProtocol::from_str(&protocol)?;
            let config = build_config(
                protocol,
                bind_host,
                port,
                output,
                username,
                None,
                advertised_host,
            );
            print_receiver_config(&config);
        }
        Some(Command::ServeFtp {
            bind_host,
            port,
            output,
            username,
            password,
            advertised_host,
        }) => {
            let config = build_config(
                PushProtocol::Ftp,
                bind_host,
                port,
                output,
                username,
                password,
                advertised_host,
            );
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
    }

    Ok(())
}

fn build_config(
    protocol: PushProtocol,
    bind_host: String,
    port: u16,
    output: PathBuf,
    username: Option<String>,
    password: Option<String>,
    advertised_host: Option<String>,
) -> PushReceiverConfig {
    let mut config = PushReceiverConfig::new(protocol, bind_host, port, output);
    config.username = username;
    config.password = password;
    config.advertised_host = advertised_host;
    config
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
}

fn parse_source(value: &str) -> Result<ImportSource> {
    match value.to_ascii_lowercase().as_str() {
        "ftp" | "ftp-push" => Ok(ImportSource::FtpPush),
        "sftp" | "sftp-push" => Ok(ImportSource::SftpPush),
        "ftps" | "ftps-push" => Ok(ImportSource::FtpsPush),
        "manual" | "manual-drop" => Ok(ImportSource::ManualDrop),
        _ => Err(nikon_importer_core::ImporterError::UnsupportedProtocol),
    }
}
