use camera_connector_core::{
    PushProtocol, ReceiverAccountConfig, ReceiverAuthMode, ReceiverRuntimePhase,
    ReceiverRuntimeStatus, SqliteStore,
};
use rusqlite::Connection;

#[test]
fn sqlite_store_upserts_receiver_accounts() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");

    let created = store
        .upsert_receiver_account(
            ReceiverAccountConfig::new(" z5 ", Some("secret"), " Z5 II ")
                .expect("account should build"),
        )
        .expect("account should upsert");

    assert_eq!(created.username, "z5");
    assert_eq!(created.device_name, "Z5 II");
    assert!(created.enabled);
    assert!(created
        .password_hash
        .as_deref()
        .expect("password hash should exist")
        .starts_with("$argon2id$"));
    assert!(created.created_at_ms > 0);
    assert!(created.updated_at_ms >= created.created_at_ms);

    let updated = store
        .upsert_receiver_account(
            ReceiverAccountConfig::new("z5", Some("new-secret"), "Studio Z5")
                .expect("account should build"),
        )
        .expect("account should update");
    let accounts = store
        .receiver_accounts()
        .expect("accounts should list from sqlite");

    assert_eq!(updated.username, "z5");
    assert_eq!(updated.device_name, "Studio Z5");
    assert_eq!(updated.created_at_ms, created.created_at_ms);
    assert!(updated.updated_at_ms >= created.updated_at_ms);
    assert_ne!(updated.password_hash, created.password_hash);
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0], updated);
}

#[test]
fn sqlite_store_removes_receiver_accounts() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    store
        .upsert_receiver_account(
            ReceiverAccountConfig::new("z5", Some("secret"), "Z5 II")
                .expect("account should build"),
        )
        .expect("account should upsert");
    store
        .record_connected_device("192.168.137.56", Some(51120), None, None)
        .expect("device should connect");
    store
        .record_authenticated_device("192.168.137.56", Some("Z5 II"), Some("z5"))
        .expect("device should authenticate");

    assert!(store
        .remove_receiver_account("z5")
        .expect("account should remove"));
    assert!(!store
        .remove_receiver_account("z5")
        .expect("missing account should be reported"));
    assert!(store
        .receiver_accounts()
        .expect("accounts should list")
        .is_empty());
    let devices = store
        .connected_devices()
        .expect("connected devices should list");
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].remote_addr, "192.168.137.56");
    assert_eq!(devices[0].source_name.as_deref(), Some("Z5 II"));
    assert_eq!(devices[0].username, None);
}

#[test]
fn sqlite_store_records_connected_device_state() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open_state_dir(temp_dir.path()).expect("store should open");

    store
        .record_connected_device("192.168.137.56", Some(51120), None, None)
        .expect("device should connect");
    store
        .record_authenticated_device("192.168.137.56", Some("Studio Z5"), Some("z5"))
        .expect("device should authenticate");
    store
        .record_connected_device("192.168.137.44", Some(51121), None, None)
        .expect("new IP should connect");
    store
        .record_authenticated_device("192.168.137.44", Some("Studio Z5"), Some("z5"))
        .expect("new IP should authenticate");

    let devices = store
        .connected_devices()
        .expect("connected devices should load");

    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].remote_addr, "192.168.137.44");
    assert_eq!(devices[0].last_remote_port, Some(51121));
    assert_eq!(devices[0].username.as_deref(), Some("z5"));
    assert_eq!(devices[0].source_name.as_deref(), Some("Studio Z5"));
    assert_eq!(devices[0].active_connections, 1);
    assert!(devices[0].online);

    store
        .record_disconnected_device("192.168.137.44")
        .expect("device should disconnect");
    let disconnected = store
        .connected_devices()
        .expect("connected devices should load");
    assert_eq!(disconnected[0].active_connections, 0);
    assert!(!disconnected[0].online);
    assert!(disconnected[0].last_disconnected_at_ms.is_some());
}

#[test]
fn sqlite_store_persists_receiver_runtime_status() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open_state_dir(temp_dir.path()).expect("store should open");
    let status = ReceiverRuntimeStatus {
        phase: ReceiverRuntimePhase::Running,
        protocol: Some(PushProtocol::Ftp),
        auth_mode: ReceiverAuthMode::Accounts,
        local_addr: Some("127.0.0.1:2121".parse().expect("addr should parse")),
        output_dir: Some(temp_dir.path().join("output")),
        state_dir: Some(temp_dir.path().to_path_buf()),
        account_count: 1,
        message: None,
    };

    store
        .write_receiver_runtime_status(&status)
        .expect("runtime status should write");
    let loaded = store
        .read_receiver_runtime_status()
        .expect("runtime status should read")
        .expect("runtime status should exist");

    assert_eq!(loaded.phase, ReceiverRuntimePhase::Running);
    assert_eq!(loaded.protocol, Some(PushProtocol::Ftp));
    assert_eq!(loaded.auth_mode, ReceiverAuthMode::Accounts);
    assert_eq!(loaded.local_addr, status.local_addr);
    assert_eq!(loaded.output_dir, status.output_dir);
    assert_eq!(loaded.state_dir, status.state_dir);
    assert_eq!(loaded.account_count, 1);
}

#[test]
fn sqlite_store_persists_receiver_runtime_status_as_columns() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let db_path = temp_dir.path().join("state.sqlite");
    let store = SqliteStore::open(&db_path).expect("store should open");
    let status = ReceiverRuntimeStatus {
        phase: ReceiverRuntimePhase::Running,
        protocol: Some(PushProtocol::Sftp),
        auth_mode: ReceiverAuthMode::Accounts,
        local_addr: Some("127.0.0.1:2222".parse().expect("addr should parse")),
        output_dir: Some(temp_dir.path().join("output")),
        state_dir: Some(temp_dir.path().join("state")),
        account_count: 2,
        message: Some("ready".to_string()),
    };

    store
        .write_receiver_runtime_status(&status)
        .expect("runtime status should write");

    let connection = Connection::open(db_path).expect("sqlite should open");
    let columns = connection
        .prepare("PRAGMA table_info(receiver_status)")
        .expect("pragma should prepare")
        .query_map([], |row| row.get::<_, String>(1))
        .expect("columns should query")
        .collect::<Result<Vec<_>, _>>()
        .expect("columns should collect");
    let row = connection
        .query_row(
            "SELECT phase, protocol, auth_mode, local_addr, output_dir, state_dir,
                    account_count, message
             FROM receiver_status
             WHERE key = 'current'",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, Option<String>>(7)?,
                ))
            },
        )
        .expect("receiver status row should query");

    assert!(!columns.iter().any(|column| column == "payload"));
    assert_eq!(row.0, "running");
    assert_eq!(row.1.as_deref(), Some("sftp"));
    assert_eq!(row.2, "accounts");
    assert_eq!(row.3.as_deref(), Some("127.0.0.1:2222"));
    assert_eq!(row.6, 2);
    assert_eq!(row.7.as_deref(), Some("ready"));
}
