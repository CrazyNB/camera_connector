use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use nikon_importer_core::{
    scanner::scan_subnet_for_ptp, CameraEndpoint, EndpointSource, LocalFileSink, NikonCameraClient,
    Result,
};
use tokio::time::Duration;

#[derive(Debug, Parser)]
#[command(name = "nikon-importer")]
#[command(about = "Wireless import validation CLI for Nikon PTP/IP cameras")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Version,
    Info(ConnectionArgs),
    List(ConnectionArgs),
    Scan {
        #[arg(long)]
        subnet: String,
        #[arg(long, default_value_t = 15740)]
        port: u16,
        #[arg(long, default_value_t = 500)]
        timeout_ms: u64,
        #[arg(long, default_value_t = 64)]
        concurrency: usize,
    },
    Thumb {
        #[command(flatten)]
        connection: ConnectionArgs,
        #[arg(long)]
        handle: u32,
        #[arg(long)]
        output: PathBuf,
    },
    Pull {
        #[command(flatten)]
        connection: ConnectionArgs,
        #[arg(long)]
        handle: u32,
        #[arg(long)]
        output: PathBuf,
    },
    Endpoint(ConnectionArgs),
}

#[derive(Debug, Parser, Clone)]
struct ConnectionArgs {
    #[arg(long)]
    host: String,
    #[arg(long, default_value_t = 15740)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Version) | None => {
            println!("nikon-importer {}", env!("CARGO_PKG_VERSION"));
        }
        Some(Command::Endpoint(connection)) => {
            let endpoint = endpoint(connection);
            println!("{}", endpoint.socket_addr());
        }
        Some(Command::Info(connection)) => {
            let mut client = NikonCameraClient::connect(endpoint(connection)).await?;
            let info = client.get_camera_info().await?;
            println!("manufacturer: {}", info.manufacturer);
            println!("model: {}", info.model);
            println!(
                "firmware: {}",
                info.firmware_version.as_deref().unwrap_or("unknown")
            );
            println!("supported_operations: {}", info.supported_operations.len());
            let _ = client.close().await;
        }
        Some(Command::List(connection)) => {
            let mut client = NikonCameraClient::connect(endpoint(connection)).await?;
            for object in client.list_objects().await? {
                println!(
                    "{}\t{}\t{:?}\t{} bytes",
                    object.handle, object.filename, object.format, object.size_bytes
                );
            }
            let _ = client.close().await;
        }
        Some(Command::Scan {
            subnet,
            port,
            timeout_ms,
            concurrency,
        }) => {
            let endpoints = scan_subnet_for_ptp(
                &subnet,
                port,
                Duration::from_millis(timeout_ms),
                concurrency,
            )
            .await?;
            for endpoint in endpoints {
                println!("{}", endpoint.socket_addr());
            }
        }
        Some(Command::Thumb {
            connection,
            handle,
            output,
        }) => {
            let mut client = NikonCameraClient::connect(endpoint(connection)).await?;
            let bytes = client.get_thumbnail(handle).await?;
            fs::write(&output, bytes)?;
            println!("wrote {}", output.display());
            let _ = client.close().await;
        }
        Some(Command::Pull {
            connection,
            handle,
            output,
        }) => {
            let mut client = NikonCameraClient::connect(endpoint(connection)).await?;
            let filename = client
                .list_objects()
                .await?
                .into_iter()
                .find(|object| object.handle == handle)
                .map(|object| object.filename)
                .unwrap_or_else(|| format!("object-{handle}.bin"));
            let bytes = client.get_object(handle).await?;
            let progress = LocalFileSink::new(output).write_complete(handle, &filename, &bytes)?;
            println!(
                "wrote {}",
                progress
                    .output_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| filename)
            );
            let _ = client.close().await;
        }
    }
    Ok(())
}

fn endpoint(connection: ConnectionArgs) -> CameraEndpoint {
    CameraEndpoint::new(connection.host, connection.port, EndpointSource::Manual)
}
