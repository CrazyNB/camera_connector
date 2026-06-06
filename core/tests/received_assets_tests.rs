use camera_connector_core::{
    group_received_assets, scan_received_asset_groups, scan_received_assets, ImportSource,
    ObjectFormat,
};

#[test]
fn scans_received_files_and_skips_temporary_uploads() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    std::fs::write(temp_dir.path().join("DSC_2467.JPG"), [1, 2, 3]).unwrap();
    std::fs::write(temp_dir.path().join("DSC_2467.NEF"), [4, 5]).unwrap();
    std::fs::write(temp_dir.path().join("DSC_2467.NEF.tmp"), [6]).unwrap();
    std::fs::write(temp_dir.path().join("README.TXT"), [6]).unwrap();
    std::fs::create_dir(temp_dir.path().join("CARD1")).unwrap();
    std::fs::write(
        temp_dir.path().join("CARD1").join("DSC_2468.MOV"),
        [7, 8, 9, 10],
    )
    .unwrap();

    let assets =
        scan_received_assets(temp_dir.path(), ImportSource::FtpPush).expect("assets should scan");

    assert_eq!(assets.len(), 3);
    assert!(assets.iter().all(|asset| !asset.filename.ends_with(".tmp")));
    assert!(assets
        .iter()
        .all(|asset| asset.format != ObjectFormat::Unknown));
    assert!(assets.iter().any(|asset| asset.filename == "DSC_2467.JPG"
        && asset.format == ObjectFormat::Jpeg
        && asset.size_bytes == 3));
    assert!(assets.iter().any(|asset| asset.filename == "DSC_2467.NEF"
        && asset.format == ObjectFormat::Nef
        && asset.size_bytes == 2));
    assert!(assets
        .iter()
        .any(|asset| asset.filename == "CARD1/DSC_2468.MOV"
            && asset.format == ObjectFormat::Mov
            && asset.size_bytes == 4));
}

#[test]
fn scans_received_files_and_skips_transfer_log() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    std::fs::write(temp_dir.path().join("IMG_1001.CR3"), [1, 2, 3]).unwrap();
    std::fs::write(temp_dir.path().join("transfer-log.jsonl"), []).unwrap();
    std::fs::write(temp_dir.path().join("connected-devices.json"), []).unwrap();
    std::fs::write(temp_dir.path().join("receiver-status.json"), []).unwrap();
    std::fs::write(temp_dir.path().join("sftp-host-key"), []).unwrap();

    let assets =
        scan_received_assets(temp_dir.path(), ImportSource::FtpPush).expect("assets should scan");

    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].filename, "IMG_1001.CR3");
}

#[test]
fn scans_received_groups_raw_jpeg_pairs() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    std::fs::write(temp_dir.path().join("DSC_2467.JPG"), [1]).unwrap();
    std::fs::write(temp_dir.path().join("DSC_2467.NEF"), [2]).unwrap();
    std::fs::write(temp_dir.path().join("DSC_2468.JPG"), [3]).unwrap();

    let groups = scan_received_asset_groups(temp_dir.path(), ImportSource::FtpPush)
        .expect("groups should scan");

    assert_eq!(groups.len(), 2);
    let pair = groups
        .iter()
        .find(|group| group.group_key == "DSC_2467")
        .expect("raw+jpeg pair should be grouped");
    assert!(pair.jpeg.is_some());
    assert!(pair.raw.is_some());

    let direct_groups = group_received_assets(
        scan_received_assets(temp_dir.path(), ImportSource::FtpPush).expect("assets should scan"),
    );
    assert_eq!(direct_groups, groups);
}
