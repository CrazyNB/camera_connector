use super::*;
use camera_connector_core::{CameraConnectorConfig, ReceiverAccountConfig, StoredObjectLocation};
use std::path::PathBuf;

mod output;
mod projects;

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
fn source_protocol_label_includes_desktop_scan() {
    assert_eq!(
        source_protocol_label(ImportSource::DesktopScan),
        "desktop_scan"
    );
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
            output: Some(PathBuf::from("C:\\CameraConnector\\Received")),
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
        Some(Path::new("C:\\CameraConnector\\Received"))
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
fn parses_assets_from_transfers_command() {
    let cli = Cli::try_parse_from([
        "camera-connector",
        "assets",
        "--diagnostic",
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
    .expect("assets from transfers command should parse");

    assert!(matches!(
        cli.command,
        Some(Command::Assets {
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
fn diagnostic_assets_requires_explicit_flag() {
    let result = Cli::try_parse_from([
        "camera-connector",
        "assets",
        "--path",
        "C:\\CameraConnector\\Received",
        "--source",
        "ftp",
    ]);

    assert!(result.is_err());
}

#[test]
fn parses_diagnostic_assets_command() {
    let cli = Cli::try_parse_from([
        "camera-connector",
        "assets",
        "--diagnostic",
        "--path",
        "C:\\CameraConnector\\Received",
        "--source",
        "ftp",
    ])
    .expect("diagnostic assets command should parse");

    assert!(matches!(
        cli.command,
        Some(Command::Assets {
            project_id: None,
            path: Some(_),
            ..
        })
    ));
}

#[test]
fn parses_project_assets_command_without_path() {
    let cli = Cli::try_parse_from([
        "camera-connector",
        "assets",
        "--config",
        "C:\\CameraConnector\\config.json",
        "--project-id",
        "project-1",
        "--summary",
        "--offset",
        "1",
        "--limit",
        "20",
    ])
    .expect("project assets command should parse");

    assert!(matches!(
        cli.command,
        Some(Command::Assets {
            path: None,
            project_id: Some(project_id),
            summary: true,
            offset: 1,
            limit: Some(20),
            ..
        }) if project_id == "project-1"
    ));
}

#[test]
fn parses_project_group_assets_command() {
    let cli = Cli::try_parse_from([
        "camera-connector",
        "project",
        "--config",
        "C:\\CameraConnector\\config.json",
        "group-assets",
        "--id",
        "project-1",
        "--group-id",
        "group-1",
    ])
    .expect("project group assets command should parse");

    assert!(matches!(
        cli.command,
        Some(Command::Project {
            action: ProjectCommand::GroupAssets {
                id,
                group_id,
            },
            ..
        }) if id == "project-1" && group_id == "group-1"
    ));
}

#[test]
fn project_assets_load_asset_page_from_sqlite() {
    let root = std::env::temp_dir().join(format!(
        "camera-connector-project-assets-{}",
        current_time_ms()
    ));
    let config_path = root.join("config.json");
    let state_dir = root.join("state");
    std::fs::create_dir_all(&root).expect("temp root should create");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    service
        .set_receiver_settings(ReceiverSettingsUpdate {
            state_dir: Some(state_dir),
            ..ReceiverSettingsUpdate::default()
        })
        .expect("receiver settings should save");
    let project = service
        .create_project("project assets")
        .expect("project should create");
    service
        .record_project_transfer(
            &project.project_id,
            TransferRecord {
                transfer_id: "ftp:project-assets".to_string(),
                protocol: "ftp".to_string(),
                status: TransferStatus::Completed,
                original_path: "DCIM/100/IMG_0202.CR3".to_string(),
                final_filename: "IMG_0202.CR3".to_string(),
                final_location: Some(StoredObjectLocation::local_path(root.join("IMG_0202.CR3"))),
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

    let page = load_project_asset_page(
        Some(config_path.clone()),
        &project.project_id,
        AssetGroupQuery::default(),
        0,
        50,
    )
    .expect("project assets page should load");

    assert_eq!(page.total_groups, 1);
    assert_eq!(page.summary.asset_count, 1);
    assert_eq!(page.groups[0].primary.filename, "IMG_0202.CR3");
    assert_eq!(
        page.groups[0].primary.display_source.as_deref(),
        Some("Verify Camera")
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn project_group_assets_loads_members_from_sqlite() {
    let root = std::env::temp_dir().join(format!(
        "camera-connector-project-group-assets-{}",
        current_time_ms()
    ));
    let config_path = root.join("config.json");
    let state_dir = root.join("state");
    std::fs::create_dir_all(&root).expect("temp root should create");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    service
        .set_receiver_settings(ReceiverSettingsUpdate {
            state_dir: Some(state_dir),
            ..ReceiverSettingsUpdate::default()
        })
        .expect("receiver settings should save");
    let project = service
        .create_project("Project Group Assets")
        .expect("project should create");
    service
        .record_project_transfer(
            &project.project_id,
            TransferRecord {
                transfer_id: "ftp:project-group-jpg".to_string(),
                protocol: "ftp".to_string(),
                status: TransferStatus::Completed,
                original_path: "DCIM/100/IMG_0303.JPG".to_string(),
                final_filename: "IMG_0303.JPG".to_string(),
                final_location: Some(StoredObjectLocation::local_path(root.join("IMG_0303.JPG"))),
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
    let group_id = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 50)
        .expect("group page should query")
        .groups[0]
        .group_id
        .clone()
        .expect("group id should exist");

    let assets =
        load_project_group_assets(Some(config_path.clone()), &project.project_id, &group_id)
            .expect("project group assets should load");
    let ambiguous =
        load_project_group_assets(Some(config_path.clone()), &project.project_id, "IMG_0303")
            .expect("display key should not query members");

    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].final_filename, "IMG_0303.JPG");
    assert_eq!(assets[0].group_id.as_deref(), Some(group_id.as_str()));
    assert!(ambiguous.is_empty());

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn parses_transfers_status_filter_command() {
    let cli = Cli::try_parse_from([
        "camera-connector",
        "transfers",
        "--diagnostic",
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
fn diagnostic_transfers_require_explicit_flag() {
    let result = Cli::try_parse_from([
        "camera-connector",
        "transfers",
        "--state",
        "C:\\CameraConnector\\state",
        "--status",
        "failed",
    ]);

    assert!(result.is_err());
}

#[test]
fn parses_project_transfers_command_without_state() {
    let cli = Cli::try_parse_from([
        "camera-connector",
        "transfers",
        "--config",
        "C:\\CameraConnector\\config.json",
        "--project-id",
        "project-1",
        "--status",
        "failed",
    ])
    .expect("project transfers command should parse");

    assert!(matches!(
        cli.command,
        Some(Command::Transfers {
            state: None,
            project_id: Some(project_id),
            status: Some(_),
            ..
        }) if project_id == "project-1"
    ));
}

#[test]
fn project_transfers_load_from_sqlite() {
    let root = std::env::temp_dir().join(format!(
        "camera-connector-project-transfers-{}",
        current_time_ms()
    ));
    let config_path = root.join("config.json");
    let state_dir = root.join("state");
    std::fs::create_dir_all(&root).expect("temp root should create");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    service
        .set_receiver_settings(ReceiverSettingsUpdate {
            state_dir: Some(state_dir),
            ..ReceiverSettingsUpdate::default()
        })
        .expect("receiver settings should save");
    let project = service
        .create_project("Project Transfers")
        .expect("project should create");
    let mut failed = TransferRecord {
        transfer_id: "ftp:project-failed".to_string(),
        protocol: "ftp".to_string(),
        status: TransferStatus::Failed,
        original_path: "DCIM/100/IMG_0303.CR3".to_string(),
        final_filename: "IMG_0303.CR3".to_string(),
        final_location: Some(StoredObjectLocation::local_path(root.join("IMG_0303.CR3"))),
        size_bytes: 42,
        username: Some("verify".to_string()),
        remote_addr: None,
        source_name: Some("Verify Camera".to_string()),
        started_at_ms: 10,
        completed_at_ms: Some(20),
        error: Some("simulated failure".to_string()),
    };
    service
        .record_project_transfer(&project.project_id, failed.clone())
        .expect("failed transfer should record");
    failed.transfer_id = "ftp:other".to_string();
    failed.status = TransferStatus::Completed;
    service
        .record_project_transfer(&project.project_id, failed)
        .expect("completed transfer should record");

    let transfers = load_transfers(
        Some(config_path.clone()),
        None,
        Some(project.project_id),
        TransferQuery {
            status: Some(TransferStatus::Failed),
            ..TransferQuery::default()
        },
    )
    .expect("project transfers should load");

    assert_eq!(transfers.len(), 1);
    assert_eq!(transfers[0].record.transfer_id, "ftp:project-failed");
    assert_eq!(
        transfers[0].display_source.as_deref(),
        Some("Verify Camera")
    );
    assert_eq!(
        transfers[0].record.error.as_deref(),
        Some("simulated failure")
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn parses_dashboard_command() {
    let cli = Cli::try_parse_from([
        "camera-connector",
        "dashboard",
        "--config",
        "C:\\CameraConnector\\config.json",
        "--project-id",
        "project-1",
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
            project_id,
            online_devices: true,
            json: true,
            offset: 0,
            limit: 25,
            ..
        }) if project_id == "project-1"
    ));
}

#[test]
fn dashboard_requires_project_id() {
    let result = Cli::try_parse_from([
        "camera-connector",
        "dashboard",
        "--config",
        "C:\\CameraConnector\\config.json",
        "--username",
        "z5",
    ]);

    assert!(result.is_err());
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
            project_id,
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
                final_location: Some(StoredObjectLocation::local_path(root.join("IMG_0101.CR3"))),
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
        project_id: project.project_id,
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

fn unique_temp_config_path(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "camera-connector-{name}-{}-{}",
        std::process::id(),
        current_time_ms()
    ));
    std::fs::create_dir_all(&root).expect("temp config directory should create");
    root.join("config.json")
}
