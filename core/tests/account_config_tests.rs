use camera_connector_core::{ReceiverAccountConfig, ReceiverPassword};

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
