use camera_connector_core::{
    AssetGroupQuery, CameraConnectorService, ModelProviderKind, ModelProviderSettings,
    ModelSendMode, PublishTransferMetadata, StoredObjectLocation, TransferRecord, TransferStatus,
};
use camera_connector_ffi::{MobileCore, MobileReceiverSettingsPatch};
use serde_json::Value;

#[path = "mobile_core_tests/analysis.rs"]
mod analysis;
#[path = "mobile_core_tests/asset_groups.rs"]
mod asset_groups;
#[path = "mobile_core_tests/projects.rs"]
mod projects;
#[path = "mobile_core_tests/publish.rs"]
mod publish;
#[path = "mobile_core_tests/receiver.rs"]
mod receiver;
#[path = "mobile_core_tests/settings.rs"]
mod settings;

fn balanced_detail_sample_json(width: usize, height: usize) -> String {
    let mut luma = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let value = 80 + ((x * 17 + y * 23) % 96) as u8;
            luma.push(value);
        }
    }
    serde_json::json!({
        "width": width,
        "height": height,
        "luma": luma,
        "preview_source": "test"
    })
    .to_string()
}

struct MobileBurstFixture {
    _temp: tempfile::TempDir,
    core: MobileCore,
    project_id: String,
    burst_id: String,
    member_group_id: String,
}

fn three_member_burst_fixture(project_name: &str) -> MobileBurstFixture {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project(project_name).unwrap();
    for (transfer_id, path, completed_at_ms) in [
        ("ftp:three-decision-1", "DCIM/100/IMG_7601.JPG", 1000),
        ("ftp:three-decision-2", "DCIM/100/IMG_7602.JPG", 1100),
        ("ftp:three-decision-3", "DCIM/100/IMG_7603.JPG", 1200),
    ] {
        service
            .record_project_transfer(
                &project.project_id,
                completed_transfer(transfer_id, path, completed_at_ms),
            )
            .unwrap();
    }
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    core.drain_analysis_jobs_json(10).unwrap();
    let page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project.project_id.clone(), "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let burst_members = page["groups"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|group| group.get("burst").is_some())
        .collect::<Vec<_>>();
    let burst_id = burst_members[0]["burst"]["burst_group_id"]
        .as_str()
        .unwrap()
        .to_string();
    let member_group_id = burst_members[1]["group_id"].as_str().unwrap().to_string();

    MobileBurstFixture {
        _temp: temp,
        core,
        project_id: project.project_id,
        burst_id,
        member_group_id,
    }
}

fn completed_transfer(
    transfer_id: &str,
    original_path: &str,
    completed_at_ms: i64,
) -> TransferRecord {
    let final_filename = original_path
        .rsplit('/')
        .next()
        .expect("filename should exist")
        .to_string();
    TransferRecord {
        transfer_id: transfer_id.to_string(),
        protocol: "ftp".to_string(),
        status: TransferStatus::Completed,
        original_path: original_path.to_string(),
        final_filename: final_filename.clone(),
        final_location: Some(StoredObjectLocation::local_path(final_filename)),
        size_bytes: 100,
        username: Some("z5".to_string()),
        remote_addr: Some("192.168.137.56".to_string()),
        source_name: Some("Studio Z5".to_string()),
        started_at_ms: completed_at_ms - 1,
        completed_at_ms: Some(completed_at_ms),
        error: None,
    }
}
