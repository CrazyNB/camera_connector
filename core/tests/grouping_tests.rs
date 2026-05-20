use nikon_importer_core::{group_received_assets, ImportSource, ObjectFormat, ReceivedAsset};

#[test]
fn groups_raw_and_jpeg_by_filename_stem() {
    let jpeg = ReceivedAsset::new("ftp-1", "DSC_1234.JPG", 8_700_000, ImportSource::FtpPush);
    let raw = ReceivedAsset::new("ftp-2", "DSC_1234.NEF", 39_500_000, ImportSource::FtpPush);

    let groups = group_received_assets(vec![raw, jpeg]);

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].group_key, "DSC_1234");
    assert_eq!(groups[0].primary.format, ObjectFormat::Jpeg);
    assert!(groups[0].jpeg.is_some());
    assert!(groups[0].raw.is_some());
    assert!(groups[0].video.is_none());
}

#[test]
fn standalone_video_remains_single_group() {
    let video = ReceivedAsset::new("ftp-3", "DSC_1236.MOV", 212_000_000, ImportSource::FtpPush);

    let groups = group_received_assets(vec![video]);

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].group_key, "DSC_1236");
    assert_eq!(groups[0].primary.format, ObjectFormat::Mov);
    assert!(groups[0].video.is_some());
}

#[test]
fn grouping_is_case_insensitive_for_filename_stem() {
    let jpeg = ReceivedAsset::new("ftp-4", "dsc_1237.jpg", 8_100_000, ImportSource::FtpPush);
    let raw = ReceivedAsset::new("ftp-5", "DSC_1237.NEF", 40_000_000, ImportSource::FtpPush);

    let groups = group_received_assets(vec![jpeg, raw]);

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].group_key, "DSC_1237");
    assert!(groups[0].jpeg.is_some());
    assert!(groups[0].raw.is_some());
}
