use camera_connector_core::{group_received_assets, ImportSource, ObjectFormat, ReceivedAsset};

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

#[test]
fn recognizes_common_raw_formats_across_camera_brands() {
    let raw_cases = [
        ("CANON.CR2", ObjectFormat::Cr2),
        ("CANON.CR3", ObjectFormat::Cr3),
        ("SONY.ARW", ObjectFormat::Arw),
        ("FUJI.RAF", ObjectFormat::Raf),
        ("PANASONIC.RW2", ObjectFormat::Rw2),
        ("OLYMPUS.ORF", ObjectFormat::Orf),
        ("PENTAX.PEF", ObjectFormat::Pef),
        ("LEICA.DNG", ObjectFormat::Dng),
        ("RAW.NEF", ObjectFormat::Nef),
    ];

    for (filename, format) in raw_cases {
        let detected = ObjectFormat::from_filename(filename);
        assert_eq!(detected, format, "{filename}");
        assert!(detected.is_raw(), "{filename}");
    }
}

#[test]
fn groups_brand_raw_files_with_matching_jpeg() {
    let jpeg = ReceivedAsset::new("ftp-jpg", "IMG_1001.JPG", 8_700_000, ImportSource::FtpPush);
    let raw = ReceivedAsset::new("ftp-raw", "IMG_1001.CR3", 42_000_000, ImportSource::FtpPush);

    let groups = group_received_assets(vec![raw, jpeg]);

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].group_key, "IMG_1001");
    assert_eq!(groups[0].primary.format, ObjectFormat::Jpeg);
    assert!(groups[0].jpeg.is_some());
    assert!(groups[0].raw.is_some());
}

#[test]
fn groups_sort_by_latest_received_time_when_capture_time_is_missing() {
    let mut older = ReceivedAsset::new("ftp-old", "IMG_0001.CR3", 10, ImportSource::FtpPush);
    older.received_time_ms = Some(100);
    let mut newer = ReceivedAsset::new("ftp-new", "IMG_0002.CR3", 10, ImportSource::FtpPush);
    newer.received_time_ms = Some(200);

    let groups = group_received_assets(vec![older, newer]);

    assert_eq!(groups[0].group_key, "IMG_0002");
    assert_eq!(groups[1].group_key, "IMG_0001");
}
