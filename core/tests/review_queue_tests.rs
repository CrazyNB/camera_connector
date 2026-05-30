use camera_connector_core::{
    AssetFormatRole, AssetGroupQuery, CameraConnectorService, PreviewSample,
    SelectionRecommendationStatus, StoredObjectLocation, TransferRecord, TransferStatus,
};

#[test]
fn review_summary_counts_burst_units_low_scores_and_unsupported_singles() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Review Queue")
        .expect("project should create");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:review-1", "DCIM/100/IMG_8101.JPG", 1000),
        )
        .expect("first burst transfer should record");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:review-2", "DCIM/100/IMG_8102.JPG", 1100),
        )
        .expect("second burst transfer should record");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:review-single", "DCIM/100/IMG_9101.JPG", 5000),
        )
        .expect("single transfer should record");
    service
        .drain_analysis_jobs(10)
        .expect("burst analysis should drain");

    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 25)
        .expect("page should load");
    let burst_members = page
        .groups
        .iter()
        .filter(|group| group.burst.is_some())
        .collect::<Vec<_>>();
    let single = page
        .groups
        .iter()
        .find(|group| group.burst.is_none())
        .expect("single frame group should exist");
    let burst_id = burst_members[0]
        .burst
        .as_ref()
        .expect("burst should exist")
        .burst_group_id
        .clone();

    service
        .score_asset_group_preview(
            burst_members[0].group_id.as_deref().expect("group id"),
            flat_sample(16, 16, 128),
            "local-v1",
        )
        .expect("soft score should save");
    service
        .score_asset_group_preview(
            burst_members[1].group_id.as_deref().expect("group id"),
            checkerboard_sample(16, 16),
            "local-v1",
        )
        .expect("sharp score should save");
    service
        .recommend_burst_group(&burst_id, None)
        .expect("recommendation should save");
    service
        .score_asset_group_preview(
            single.group_id.as_deref().expect("group id"),
            PreviewSample {
                width: 0,
                height: 0,
                luma: Vec::new(),
                preview_source: Some("missing".to_string()),
            },
            "local-v1",
        )
        .expect("unsupported score should save");

    let summary = service
        .project_review_queue_summary(&project.project_id, None)
        .expect("summary should load");

    assert_eq!(summary.total_units, 2);
    assert_eq!(summary.unconfirmed_best_count, 1);
    assert_eq!(summary.low_score_candidate_count, 1);
    assert_eq!(summary.unsupported_count, 1);
    assert_eq!(summary.needs_review_count, 1);
    assert_eq!(summary.queue_count("unconfirmed_best"), Some(1));
    assert_eq!(summary.queue_count("unsupported"), Some(1));
}

#[test]
fn accept_recommended_best_removes_group_from_unconfirmed_queue() {
    let fixture = recommended_burst_fixture();
    let before = fixture
        .service
        .project_review_queue_summary(&fixture.project_id, None)
        .expect("summary should load before decision");
    assert_eq!(before.unconfirmed_best_count, 1);

    let accepted = fixture
        .service
        .accept_recommended_best(&fixture.burst_id, None)
        .expect("recommended best should be accepted");
    let after = fixture
        .service
        .project_review_queue_summary(&fixture.project_id, None)
        .expect("summary should load after decision");

    assert_eq!(accepted.status, SelectionRecommendationStatus::Accepted);
    assert_eq!(after.unconfirmed_best_count, 0);
    assert_eq!(after.needs_review_count, 0);
    assert_eq!(after.user_overridden_count, 0);
}

#[test]
fn restore_automatic_recommendation_undoes_latest_review_decision() {
    let fixture = recommended_burst_fixture();
    fixture
        .service
        .accept_recommended_best(&fixture.burst_id, None)
        .expect("recommended best should be accepted");
    let accepted = fixture
        .service
        .project_review_queue_summary(&fixture.project_id, None)
        .expect("summary should load after accept");
    assert_eq!(accepted.unconfirmed_best_count, 0);

    let restored = fixture
        .service
        .restore_automatic_recommendation(&fixture.burst_id, None)
        .expect("recommendation should restore to automatic ready state");
    let after = fixture
        .service
        .project_review_queue_summary(&fixture.project_id, None)
        .expect("summary should load after undo");

    assert_eq!(restored.status, SelectionRecommendationStatus::Ready);
    assert_eq!(after.unconfirmed_best_count, 1);
    assert_eq!(after.needs_review_count, 0);
}

#[test]
fn override_recommended_best_records_user_choice_and_selects_it() {
    let fixture = recommended_burst_fixture();

    let overridden = fixture
        .service
        .override_recommended_best(&fixture.burst_id, &fixture.alternate_group_id, None)
        .expect("manual best override should save");
    let summary = fixture
        .service
        .project_review_queue_summary(&fixture.project_id, None)
        .expect("summary should load after override");
    let selects = fixture
        .service
        .project_selects_asset_group_page(&fixture.project_id, None, 0, 25)
        .expect("selects should include manual best");

    assert_eq!(
        overridden.status,
        SelectionRecommendationStatus::UserOverridden
    );
    assert_eq!(
        overridden.best_asset_group_id.as_deref(),
        Some(fixture.alternate_group_id.as_str()),
    );
    assert_eq!(summary.unconfirmed_best_count, 0);
    assert_eq!(summary.user_overridden_count, 1);
    assert_eq!(selects.total_groups, 1);
    assert_eq!(
        selects.groups[0].group_id.as_deref(),
        Some(fixture.alternate_group_id.as_str()),
    );
}

#[test]
fn accepted_recommendations_appear_in_selects_collection() {
    let fixture = recommended_burst_fixture();
    fixture
        .service
        .accept_recommended_best(&fixture.burst_id, None)
        .expect("recommended best should be accepted");

    let selects = fixture
        .service
        .project_selects_asset_group_page(&fixture.project_id, None, 0, 25)
        .expect("selects page should load");

    assert_eq!(selects.total_groups, 1);
    assert_eq!(selects.groups.len(), 1);
    assert_eq!(
        selects.groups[0].group_id.as_deref(),
        Some(fixture.best_group_id.as_str()),
    );
}

#[test]
fn mark_needs_review_moves_group_to_needs_review_queue() {
    let fixture = recommended_burst_fixture();
    let before = fixture
        .service
        .project_review_queue_summary(&fixture.project_id, None)
        .expect("summary should load before decision");
    assert_eq!(before.unconfirmed_best_count, 1);

    let marked = fixture
        .service
        .mark_burst_needs_review(&fixture.burst_id, None)
        .expect("burst should be marked for review");
    let after = fixture
        .service
        .project_review_queue_summary(&fixture.project_id, None)
        .expect("summary should load after decision");

    assert_eq!(marked.status, SelectionRecommendationStatus::NeedsReview);
    assert_eq!(after.unconfirmed_best_count, 0);
    assert_eq!(after.needs_review_count, 1);
}

#[test]
fn clear_recommendation_removes_best_and_marks_group_for_review() {
    let fixture = recommended_burst_fixture();

    let cleared = fixture
        .service
        .clear_recommendation(&fixture.burst_id, None)
        .expect("recommendation should clear");
    let summary = fixture
        .service
        .project_review_queue_summary(&fixture.project_id, None)
        .expect("summary should load after clear");
    let selects = fixture
        .service
        .project_selects_asset_group_page(&fixture.project_id, None, 0, 25)
        .expect("selects should load after clear");

    assert_eq!(cleared.status, SelectionRecommendationStatus::Cleared);
    assert_eq!(cleared.best_asset_group_id, None);
    assert_eq!(summary.unconfirmed_best_count, 0);
    assert_eq!(summary.needs_review_count, 1);
    assert_eq!(selects.total_groups, 0);
}

#[test]
fn keep_all_candidates_resolves_unconfirmed_without_selecting_single_best() {
    let fixture = recommended_burst_fixture();

    let kept = fixture
        .service
        .keep_all_candidates(&fixture.burst_id, None)
        .expect("recommendation should record keep all");
    let summary = fixture
        .service
        .project_review_queue_summary(&fixture.project_id, None)
        .expect("summary should load after keep all");
    let selects = fixture
        .service
        .project_selects_asset_group_page(&fixture.project_id, None, 0, 25)
        .expect("selects should not include keep-all decisions");

    assert_eq!(kept.status, SelectionRecommendationStatus::KeptAll);
    assert_eq!(summary.unconfirmed_best_count, 0);
    assert_eq!(summary.needs_review_count, 0);
    assert_eq!(selects.total_groups, 0);
}

#[test]
fn hide_low_score_candidates_removes_group_from_low_score_queue() {
    let fixture = low_score_burst_fixture();
    let before = fixture
        .service
        .project_review_queue_summary(&fixture.project_id, None)
        .expect("summary should load before hide");
    assert_eq!(before.low_score_candidate_count, 1);

    let hidden = fixture
        .service
        .hide_low_score_candidates(&fixture.burst_id, None)
        .expect("low-score candidates should hide");
    let after = fixture
        .service
        .project_review_queue_summary(&fixture.project_id, None)
        .expect("summary should load after hide");

    assert_eq!(hidden.status, SelectionRecommendationStatus::LowScoreHidden);
    assert_eq!(hidden.low_score_asset_group_ids, Vec::<String>::new());
    assert_eq!(after.low_score_candidate_count, 0);
}

#[test]
fn split_burst_member_invalidates_recommendation_and_returns_burst_to_pending() {
    let fixture = three_member_recommended_burst_fixture();
    let before = fixture
        .service
        .project_review_queue_summary(&fixture.project_id, None)
        .expect("summary should load before split");
    assert_eq!(before.unconfirmed_best_count, 1);

    let updated = fixture
        .service
        .split_burst_member(&fixture.burst_id, &fixture.alternate_group_id)
        .expect("member should split")
        .expect("remaining burst should still exist");
    let after = fixture
        .service
        .project_review_queue_summary(&fixture.project_id, None)
        .expect("summary should load after split");
    let page = fixture
        .service
        .project_asset_group_page_with_query(&fixture.project_id, Default::default(), 0, 25)
        .expect("asset page should load after split");
    let split_group = page
        .groups
        .iter()
        .find(|group| group.group_id.as_deref() == Some(fixture.alternate_group_id.as_str()))
        .expect("split group should remain visible");

    assert_eq!(updated.member_count, 2);
    assert_eq!(updated.recommendation_status, "pending");
    assert_eq!(after.unconfirmed_best_count, 0);
    assert_eq!(after.pending_count, 2);
    assert_eq!(after.user_overridden_count, 1);
    assert!(split_group.burst.is_none());
}

#[test]
fn merge_burst_member_invalidates_source_and_target_recommendations() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Merge Review Decision")
        .expect("project should create");
    for (transfer_id, path, started_at_ms) in [
        ("ftp:merge-a-1", "DCIM/230/IMG_8231.JPG", 1000),
        ("ftp:merge-a-2", "DCIM/230/IMG_8232.JPG", 1100),
        ("ftp:merge-b-1", "DCIM/231/IMG_9231.JPG", 5000),
        ("ftp:merge-b-2", "DCIM/231/IMG_9232.JPG", 5100),
    ] {
        service
            .record_project_transfer(
                &project.project_id,
                completed_transfer(transfer_id, path, started_at_ms),
            )
            .expect("burst transfer should record");
    }
    service
        .drain_analysis_jobs(10)
        .expect("burst analysis should drain");

    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 25)
        .expect("page should load");
    let target = page
        .groups
        .iter()
        .find(|group| {
            group
                .primary
                .original_path
                .as_deref()
                .map(|path| path.ends_with("IMG_8231.JPG"))
                .unwrap_or(false)
        })
        .expect("target member should exist");
    let source = page
        .groups
        .iter()
        .find(|group| {
            group
                .primary
                .original_path
                .as_deref()
                .map(|path| path.ends_with("IMG_9231.JPG"))
                .unwrap_or(false)
        })
        .expect("source member should exist");
    let target_burst_id = target
        .burst
        .as_ref()
        .expect("target burst should exist")
        .burst_group_id
        .clone();
    let source_member_group_id = source
        .group_id
        .clone()
        .expect("source group id should exist");
    let mut recommended_burst_ids = Vec::new();
    for burst_id in page.groups.iter().filter_map(|group| {
        group
            .burst
            .as_ref()
            .map(|burst| burst.burst_group_id.clone())
    }) {
        if recommended_burst_ids.iter().any(|seen| seen == &burst_id) {
            continue;
        }
        let member_group_ids = page
            .groups
            .iter()
            .filter(|group| {
                group
                    .burst
                    .as_ref()
                    .map(|burst| burst.burst_group_id.as_str() == burst_id.as_str())
                    .unwrap_or(false)
            })
            .filter_map(|group| group.group_id.as_ref())
            .cloned()
            .collect::<Vec<_>>();
        for group_id in member_group_ids {
            service
                .score_asset_group_preview(&group_id, checkerboard_sample(16, 16), "local-v1")
                .expect("score should save");
        }
        service
            .recommend_burst_group(&burst_id, None)
            .expect("recommendation should save");
        recommended_burst_ids.push(burst_id);
    }
    let before = service
        .project_review_queue_summary(&project.project_id, None)
        .expect("summary should load before merge");
    assert_eq!(before.unconfirmed_best_count, 2);

    let merged = service
        .merge_burst_member(&target_burst_id, &source_member_group_id)
        .expect("member should merge")
        .expect("merged burst should remain");
    let after = service
        .project_review_queue_summary(&project.project_id, None)
        .expect("summary should load after merge");

    assert_eq!(merged.member_count, 4);
    assert_eq!(merged.recommendation_status, "pending");
    assert_eq!(merged.user_override_state.as_deref(), Some("merge"));
    assert_eq!(after.unconfirmed_best_count, 0);
    assert_eq!(after.pending_count, 1);
    assert_eq!(after.user_overridden_count, 1);
}

#[test]
fn review_queue_page_collapses_burst_to_recommended_best_asset_group() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Review Unit Page")
        .expect("project should create");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:unit-1", "DCIM/300/IMG_8301.JPG", 1000),
        )
        .expect("first burst transfer should record");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:unit-2", "DCIM/300/IMG_8302.JPG", 1100),
        )
        .expect("second burst transfer should record");
    service
        .drain_analysis_jobs(10)
        .expect("burst analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 25)
        .expect("asset page should load");
    let burst_members = page
        .groups
        .iter()
        .filter(|group| group.burst.is_some())
        .collect::<Vec<_>>();
    let burst_id = burst_members[0]
        .burst
        .as_ref()
        .unwrap()
        .burst_group_id
        .clone();
    service
        .score_asset_group_preview(
            burst_members[0].group_id.as_deref().expect("group id"),
            flat_sample(16, 16, 128),
            "local-v1",
        )
        .expect("soft score should save");
    service
        .score_asset_group_preview(
            burst_members[1].group_id.as_deref().expect("group id"),
            checkerboard_sample(16, 16),
            "local-v1",
        )
        .expect("sharp score should save");
    let recommendation = service
        .recommend_burst_group(&burst_id, None)
        .expect("recommendation should save");

    let queue_page = service
        .project_review_queue_asset_group_page(&project.project_id, None, "unconfirmed_best", 0, 25)
        .expect("queue page should load");

    assert_eq!(queue_page.total_groups, 1);
    assert_eq!(queue_page.groups.len(), 1);
    assert_eq!(
        queue_page.groups[0].group_id.as_deref(),
        recommendation.best_asset_group_id.as_deref(),
    );
    assert_eq!(
        queue_page.groups[0]
            .burst
            .as_ref()
            .map(|burst| burst.member_count),
        Some(2),
    );
}

#[test]
fn review_queue_page_respects_username_and_role_filters() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Review Queue Filters")
        .expect("project should create");
    for (username, source_name, base_time, stem) in [
        ("z6", "Nikon Z6", 1000, "IMG_840"),
        ("z7", "Nikon Z7", 3000, "IMG_940"),
    ] {
        for index in 1..=2 {
            let captured_at = base_time + (index - 1) * 100;
            for extension in ["JPG", "NEF"] {
                service
                    .record_project_transfer(
                        &project.project_id,
                        completed_transfer_from(
                            &format!("ftp:{username}:{index}:{extension}"),
                            &format!("DCIM/400/{stem}{index}.{extension}"),
                            captured_at,
                            username,
                            source_name,
                        ),
                    )
                    .expect("transfer should record");
            }
        }
    }
    service
        .drain_analysis_jobs(20)
        .expect("burst analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 25)
        .expect("asset page should load");
    let mut burst_ids = std::collections::BTreeSet::new();
    for group in page.groups.iter().filter(|group| group.burst.is_some()) {
        service
            .score_asset_group_preview(
                group.group_id.as_deref().expect("group id"),
                checkerboard_sample(16, 16),
                "local-v1",
            )
            .expect("score should save");
        if let Some(burst_id) = group
            .burst
            .as_ref()
            .map(|burst| burst.burst_group_id.clone())
        {
            burst_ids.insert(burst_id);
        }
    }
    for burst_id in burst_ids {
        service
            .recommend_burst_group(&burst_id, None)
            .expect("recommendation should save");
    }

    let filtered = service
        .project_asset_group_page_with_query(
            &project.project_id,
            AssetGroupQuery {
                username: Some("z7".to_string()),
                role: Some(AssetFormatRole::Raw),
                review_queue: Some("unconfirmed_best".to_string()),
                ..AssetGroupQuery::default()
            },
            0,
            25,
        )
        .expect("filtered review queue should load");
    let missing_account = service
        .project_asset_group_page_with_query(
            &project.project_id,
            AssetGroupQuery {
                username: Some("missing".to_string()),
                review_queue: Some("unconfirmed_best".to_string()),
                ..AssetGroupQuery::default()
            },
            0,
            25,
        )
        .expect("missing account review queue should load");

    assert_eq!(filtered.total_groups, 1);
    assert_eq!(filtered.groups.len(), 1);
    assert_eq!(filtered.groups[0].primary.username.as_deref(), Some("z7"));
    assert!(filtered.groups[0].raw.is_some());
    assert_eq!(missing_account.total_groups, 0);
}

struct ReviewFixture {
    _temp_dir: tempfile::TempDir,
    service: CameraConnectorService,
    project_id: String,
    burst_id: String,
    best_group_id: String,
    alternate_group_id: String,
}

fn recommended_burst_fixture() -> ReviewFixture {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Review Decision")
        .expect("project should create");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:decision-1", "DCIM/200/IMG_8201.JPG", 1000),
        )
        .expect("first burst transfer should record");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:decision-2", "DCIM/200/IMG_8202.JPG", 1100),
        )
        .expect("second burst transfer should record");
    service
        .drain_analysis_jobs(10)
        .expect("burst analysis should drain");

    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 25)
        .expect("page should load");
    let burst_members = page
        .groups
        .iter()
        .filter(|group| group.burst.is_some())
        .collect::<Vec<_>>();
    let burst_id = burst_members[0]
        .burst
        .as_ref()
        .expect("burst should exist")
        .burst_group_id
        .clone();

    for member in burst_members {
        service
            .score_asset_group_preview(
                member.group_id.as_deref().expect("group id"),
                checkerboard_sample(16, 16),
                "local-v1",
            )
            .expect("score should save");
    }
    let recommendation = service
        .recommend_burst_group(&burst_id, None)
        .expect("recommendation should save");
    let best_group_id = recommendation
        .best_asset_group_id
        .clone()
        .expect("recommendation should choose best group");
    let alternate_group_id = page
        .groups
        .iter()
        .filter(|group| group.burst.is_some())
        .filter_map(|group| group.group_id.as_ref())
        .find(|group_id| *group_id != &best_group_id)
        .cloned()
        .expect("fixture should have an alternate group");

    ReviewFixture {
        _temp_dir: temp_dir,
        service,
        project_id: project.project_id,
        burst_id,
        best_group_id,
        alternate_group_id,
    }
}

fn low_score_burst_fixture() -> ReviewFixture {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Low Score Review Decision")
        .expect("project should create");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:low-1", "DCIM/210/IMG_8211.JPG", 1000),
        )
        .expect("first burst transfer should record");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:low-2", "DCIM/210/IMG_8212.JPG", 1100),
        )
        .expect("second burst transfer should record");
    service
        .drain_analysis_jobs(10)
        .expect("burst analysis should drain");

    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 25)
        .expect("page should load");
    let burst_members = page
        .groups
        .iter()
        .filter(|group| group.burst.is_some())
        .collect::<Vec<_>>();
    let burst_id = burst_members[0]
        .burst
        .as_ref()
        .expect("burst should exist")
        .burst_group_id
        .clone();
    let first_group_id = burst_members[0]
        .group_id
        .clone()
        .expect("first group id should exist");
    let second_group_id = burst_members[1]
        .group_id
        .clone()
        .expect("second group id should exist");

    service
        .score_asset_group_preview(&first_group_id, flat_sample(16, 16, 128), "local-v1")
        .expect("low score should save");
    service
        .score_asset_group_preview(&second_group_id, checkerboard_sample(16, 16), "local-v1")
        .expect("high score should save");
    let recommendation = service
        .recommend_burst_group(&burst_id, None)
        .expect("recommendation should save");
    let best_group_id = recommendation
        .best_asset_group_id
        .clone()
        .expect("recommendation should choose best group");
    let alternate_group_id = if best_group_id == first_group_id {
        second_group_id
    } else {
        first_group_id
    };

    ReviewFixture {
        _temp_dir: temp_dir,
        service,
        project_id: project.project_id,
        burst_id,
        best_group_id,
        alternate_group_id,
    }
}

fn three_member_recommended_burst_fixture() -> ReviewFixture {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Three Member Review Decision")
        .expect("project should create");
    for (transfer_id, path, started_at_ms) in [
        ("ftp:three-1", "DCIM/220/IMG_8221.JPG", 1000),
        ("ftp:three-2", "DCIM/220/IMG_8222.JPG", 1100),
        ("ftp:three-3", "DCIM/220/IMG_8223.JPG", 1200),
    ] {
        service
            .record_project_transfer(
                &project.project_id,
                completed_transfer(transfer_id, path, started_at_ms),
            )
            .expect("burst transfer should record");
    }
    service
        .drain_analysis_jobs(10)
        .expect("burst analysis should drain");

    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 25)
        .expect("page should load");
    let burst_members = page
        .groups
        .iter()
        .filter(|group| group.burst.is_some())
        .collect::<Vec<_>>();
    let burst_id = burst_members[0]
        .burst
        .as_ref()
        .expect("burst should exist")
        .burst_group_id
        .clone();

    for member in burst_members {
        service
            .score_asset_group_preview(
                member.group_id.as_deref().expect("group id"),
                checkerboard_sample(16, 16),
                "local-v1",
            )
            .expect("score should save");
    }
    let recommendation = service
        .recommend_burst_group(&burst_id, None)
        .expect("recommendation should save");
    let best_group_id = recommendation
        .best_asset_group_id
        .clone()
        .expect("recommendation should choose best group");
    let alternate_group_id = page
        .groups
        .iter()
        .filter(|group| group.burst.is_some())
        .filter_map(|group| group.group_id.as_ref())
        .find(|group_id| *group_id != &best_group_id)
        .cloned()
        .expect("fixture should have an alternate group");

    ReviewFixture {
        _temp_dir: temp_dir,
        service,
        project_id: project.project_id,
        burst_id,
        best_group_id,
        alternate_group_id,
    }
}

fn flat_sample(width: usize, height: usize, value: u8) -> PreviewSample {
    PreviewSample {
        width,
        height,
        luma: vec![value; width * height],
        preview_source: Some("test".to_string()),
    }
}

fn checkerboard_sample(width: usize, height: usize) -> PreviewSample {
    let mut luma = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            luma.push(if (x + y) % 2 == 0 { 0 } else { 255 });
        }
    }
    PreviewSample {
        width,
        height,
        luma,
        preview_source: Some("test".to_string()),
    }
}

fn completed_transfer(
    transfer_id: &str,
    original_path: &str,
    started_at_ms: i64,
) -> TransferRecord {
    let final_filename = original_path.rsplit('/').next().unwrap().to_string();
    TransferRecord {
        transfer_id: transfer_id.to_string(),
        protocol: "ftp".to_string(),
        status: TransferStatus::Completed,
        original_path: original_path.to_string(),
        final_filename: final_filename.clone(),
        final_path: None,
        final_location: Some(StoredObjectLocation::local_path(final_filename)),
        size_bytes: 100,
        username: Some("z6".to_string()),
        remote_addr: Some("192.168.137.56".to_string()),
        source_name: Some("Nikon Z".to_string()),
        started_at_ms,
        completed_at_ms: Some(started_at_ms),
        error: None,
    }
}

fn completed_transfer_from(
    transfer_id: &str,
    original_path: &str,
    started_at_ms: i64,
    username: &str,
    source_name: &str,
) -> TransferRecord {
    let mut record = completed_transfer(transfer_id, original_path, started_at_ms);
    record.username = Some(username.to_string());
    record.source_name = Some(source_name.to_string());
    record
}
