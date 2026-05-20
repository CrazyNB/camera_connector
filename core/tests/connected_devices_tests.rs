use camera_connector_core::{
    read_connected_devices, record_device_connected, record_device_disconnected,
};

#[test]
fn records_connected_and_disconnected_devices() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");

    record_device_connected(
        temp_dir.path(),
        "192.168.137.56",
        Some(51120),
        Some("Z5_2"),
        Some("z5"),
    )
    .expect("device should connect");
    let connected = read_connected_devices(temp_dir.path()).expect("devices should read");

    assert_eq!(connected.len(), 1);
    assert_eq!(connected[0].remote_addr, "192.168.137.56");
    assert_eq!(connected[0].last_remote_port, Some(51120));
    assert_eq!(connected[0].source_name.as_deref(), Some("Z5_2"));
    assert_eq!(connected[0].username.as_deref(), Some("z5"));
    assert_eq!(connected[0].active_connections, 1);
    assert!(connected[0].online);

    record_device_disconnected(temp_dir.path(), "192.168.137.56")
        .expect("device should disconnect");
    let disconnected = read_connected_devices(temp_dir.path()).expect("devices should read");

    assert_eq!(disconnected[0].active_connections, 0);
    assert!(!disconnected[0].online);
    assert!(disconnected[0].last_disconnected_at_ms.is_some());
}
