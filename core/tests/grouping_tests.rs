use nikon_importer_core::{group_camera_objects, CameraObject, ObjectFormat};

#[test]
fn groups_raw_and_jpeg_by_filename_stem() {
    let jpeg = CameraObject::new(1, 1, "DSC_1234.JPG", 8_700_000);
    let raw = CameraObject::new(2, 1, "DSC_1234.NEF", 39_500_000);

    let groups = group_camera_objects(vec![raw, jpeg]);

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].group_key, "DSC_1234");
    assert_eq!(groups[0].primary.format, ObjectFormat::Jpeg);
    assert!(groups[0].jpeg.is_some());
    assert!(groups[0].raw.is_some());
    assert!(groups[0].video.is_none());
}

#[test]
fn standalone_video_remains_single_group() {
    let video = CameraObject::new(3, 1, "DSC_1236.MOV", 212_000_000);

    let groups = group_camera_objects(vec![video]);

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].group_key, "DSC_1236");
    assert_eq!(groups[0].primary.format, ObjectFormat::Mov);
    assert!(groups[0].video.is_some());
}

#[test]
fn grouping_is_case_insensitive_for_filename_stem() {
    let jpeg = CameraObject::new(4, 1, "dsc_1237.jpg", 8_100_000);
    let raw = CameraObject::new(5, 1, "DSC_1237.NEF", 40_000_000);

    let groups = group_camera_objects(vec![jpeg, raw]);

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].group_key, "DSC_1237");
    assert!(groups[0].jpeg.is_some());
    assert!(groups[0].raw.is_some());
}
