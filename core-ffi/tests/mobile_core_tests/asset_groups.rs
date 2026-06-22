use super::*;

#[test]
fn mobile_core_returns_project_group_assets_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Mobile Members").unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:mobile-jpg", "DCIM/100/IMG_5001.JPG", 20),
        )
        .unwrap();
    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .unwrap();
    let group_id = page.groups[0].group_id.clone().unwrap();

    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    let json = core
        .project_group_assets_json(project.project_id.clone(), group_id.clone())
        .unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();
    let assets = value.as_array().unwrap();
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0]["project_id"], project.project_id);
    assert_eq!(assets[0]["group_id"], group_id);
    assert_eq!(assets[0]["final_filename"], "IMG_5001.JPG");

    let ambiguous_json = core
        .project_group_assets_json(project.project_id, "IMG_5001".to_string())
        .unwrap();
    let ambiguous: Value = serde_json::from_str(&ambiguous_json).unwrap();
    assert!(ambiguous.as_array().unwrap().is_empty());
}

#[test]
fn mobile_core_returns_project_asset_group_page_json_with_query() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Mobile Query").unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:mobile-query-jpg", "DCIM/100/IMG_7001.JPG", 20),
        )
        .unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:mobile-query-raw", "DCIM/100/IMG_7001.NEF", 21),
        )
        .unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:mobile-query-video", "DCIM/100/VID_7002.MP4", 22),
        )
        .unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let raw_page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(
                project.project_id.clone(),
                r#"{"role":"raw"}"#.to_string(),
                0,
                25,
            )
            .unwrap(),
    )
    .unwrap();
    let video_page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(
                project.project_id,
                r#"{"role":"video"}"#.to_string(),
                0,
                25,
            )
            .unwrap(),
    )
    .unwrap();

    assert_eq!(raw_page["total_groups"], 1);
    assert_eq!(raw_page["groups"][0]["group_key"], "IMG_7001");
    assert_eq!(video_page["total_groups"], 1);
    assert_eq!(video_page["groups"][0]["group_key"], "VID_7002");
}

#[test]
fn mobile_core_splits_burst_member_json() {
    let fixture = three_member_burst_fixture("Mobile Split Decision");

    let updated: Value = serde_json::from_str(
        &fixture
            .core
            .split_burst_member_json(
                fixture.burst_id.to_string(),
                fixture.member_group_id.to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    let page: Value = serde_json::from_str(
        &fixture
            .core
            .project_asset_group_page_json(fixture.project_id, "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let split_group = page["groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["group_id"].as_str() == Some(fixture.member_group_id.as_str()))
        .expect("split group should remain visible");

    assert_eq!(updated["member_count"], 2);
    assert_eq!(updated["recommendation_status"], "pending");
    assert!(split_group.get("burst").is_none());
}
