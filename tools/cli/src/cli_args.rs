use std::path::PathBuf;

use camera_connector_core::{AssetGroupQuery, ImportSource, PushProtocol};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "camera-connector")]
#[command(about = "Push-mode wireless import receiver for cameras")]
pub(super) struct Cli {
    #[command(subcommand)]
    pub(super) command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub(super) enum Command {
    Version,
    ReceiveFile {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long, alias = "project")]
        project_id: String,
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
        #[arg(long, alias = "project")]
        project_id: String,
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
    Assets {
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, required_unless_present = "project_id")]
        path: Option<PathBuf>,
        #[arg(long, alias = "project")]
        project_id: Option<String>,
        #[arg(
            long,
            required_unless_present = "project_id",
            conflicts_with = "project_id"
        )]
        diagnostic: bool,
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
        #[arg(long, alias = "path", required_unless_present = "project_id")]
        state: Option<PathBuf>,
        #[arg(long, alias = "project")]
        project_id: Option<String>,
        #[arg(
            long,
            required_unless_present = "project_id",
            conflicts_with = "project_id"
        )]
        diagnostic: bool,
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
pub(super) enum AccountCommand {
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
pub(super) enum ProjectCommand {
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
    Archive {
        #[arg(long, alias = "project-id")]
        id: String,
    },
    Restore {
        #[arg(long, alias = "project-id")]
        id: String,
    },
    Rename {
        #[arg(long, alias = "project-id")]
        id: String,
        #[arg(long)]
        name: String,
    },
    GroupAssets {
        #[arg(long, alias = "project-id")]
        id: String,
        #[arg(long = "group-id")]
        group_id: String,
    },
}

pub(super) struct ConfigArgs {
    pub(super) config_path: Option<PathBuf>,
    pub(super) protocol: PushProtocol,
    pub(super) bind_host: String,
    pub(super) port: u16,
    pub(super) output: PathBuf,
    pub(super) state: Option<PathBuf>,
    pub(super) username: Option<String>,
    pub(super) password: Option<String>,
    pub(super) advertised_host: Option<String>,
    pub(super) source_name: Option<String>,
}

pub(super) struct ReceiverSettingsArgs {
    pub(super) protocol: Option<PushProtocol>,
    pub(super) bind_host: Option<String>,
    pub(super) ftp_port: Option<u16>,
    pub(super) sftp_port: Option<u16>,
    pub(super) output: Option<PathBuf>,
    pub(super) state: Option<PathBuf>,
    pub(super) advertised_host: Option<String>,
    pub(super) source_name: Option<String>,
}

pub(super) struct ReceiveFileArgs {
    pub(super) input: PathBuf,
    pub(super) output: PathBuf,
    pub(super) project_id: String,
    pub(super) state: Option<PathBuf>,
    pub(super) source: ImportSource,
    pub(super) username: Option<String>,
    pub(super) source_name: Option<String>,
}

pub(super) struct DashboardArgs {
    pub(super) config: Option<PathBuf>,
    pub(super) project_id: String,
    pub(super) query: AssetGroupQuery,
    pub(super) offset: usize,
    pub(super) limit: usize,
    pub(super) online_devices: bool,
}
