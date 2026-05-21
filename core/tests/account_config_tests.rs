use camera_connector_core::{CameraConnectorConfig, ReceiverAccountConfig, ReceiverPassword};

#[test]
fn receiver_account_config_hashes_plain_passwords() {
    let account =
        ReceiverAccountConfig::new(" z5 ", Some("secret"), " Z5_2 ").expect("account builds");

    assert_eq!(account.username, "z5");
    assert_eq!(account.device_name, "Z5_2");
    assert!(account.password.is_none());
    let hash = account
        .password_hash
        .as_ref()
        .expect("password hash should exist");
    assert_ne!(hash, "secret");
    assert!(hash.starts_with("$argon2"));

    let receiver_account = account.into_receiver_account();
    assert!(receiver_account
        .password
        .as_ref()
        .expect("receiver password should exist")
        .verify("secret")
        .expect("password should verify"));
    assert!(!receiver_account
        .password
        .as_ref()
        .expect("receiver password should exist")
        .verify("wrong")
        .expect("wrong password should not verify"));
}

#[test]
fn receiver_password_rejects_invalid_hashes() {
    let password = ReceiverPassword::argon2id("not-a-valid-hash");

    assert!(password.validate().is_err());
}

#[test]
fn camera_connector_config_persists_accounts_without_plaintext_passwords() {
    let path = unique_temp_config_path("core-config");
    let mut config = CameraConnectorConfig::default();
    let account = config
        .set_account(" z5 ", Some("secret"), " Z5_2 ")
        .expect("account should save into config")
        .clone();

    assert_eq!(account.username, "z5");
    assert_eq!(account.device_name, "Z5_2");
    assert!(account.password_hash.is_some());

    let saved_path = config.save(Some(&path)).expect("config saves");
    assert_eq!(saved_path, path);
    let raw = std::fs::read_to_string(&path).expect("config should read");
    assert!(!raw.contains("secret"));
    assert!(raw.contains("password_hash"));

    let loaded = CameraConnectorConfig::load(Some(&path)).expect("config loads");
    let accounts = loaded
        .effective_accounts(None, None, None)
        .expect("receiver accounts should build");
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].username, "z5");
    assert!(accounts[0]
        .password
        .as_ref()
        .expect("receiver password should exist")
        .verify("secret")
        .expect("password should verify"));

    let _ = std::fs::remove_file(path);
}

#[test]
fn camera_connector_config_overrides_with_transient_account() {
    let mut config = CameraConnectorConfig::default();
    config
        .set_account("z5", Some("old-secret"), "Z5_2")
        .expect("configured account should save");

    let accounts = config
        .effective_accounts(Some("z5"), Some("new-secret"), Some("Field Body"))
        .expect("transient account should build");

    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].username, "z5");
    assert_eq!(accounts[0].device_name, "Field Body");
    assert!(accounts[0]
        .password
        .as_ref()
        .expect("receiver password should exist")
        .verify("new-secret")
        .expect("new password should verify"));
}

fn unique_temp_config_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "camera-connector-{name}-{}.json",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or_default()
    ))
}
