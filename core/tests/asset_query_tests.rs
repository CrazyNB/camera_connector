use camera_connector_core::{
    AssetGroupQuery, AssetGroupSort, BurstGroupingProfile, ModelEvaluation, ModelEvaluationStatus,
    ModelEvaluationTier, ModelEvaluatorKind, SelectionRecommendation, SelectionRecommendationScope,
    SelectionRecommendationStatus, SelectionSource, SqliteStore, StoredObjectLocation,
    TechnicalAssessment, TechnicalAssessmentStatus, TechnicalDefectFlag, TechnicalDefectSeverity,
    TechnicalDefectType, TechnicalGateStatus, TransferRecord, TransferStatus,
};

#[test]
fn asset_groups_sort_by_model_score() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Model Score Sort")
        .expect("project should create");
    record_jpeg(
        &store,
        &project.project_id,
        "ftp:low",
        "DCIM/100/IMG_3001.JPG",
        1000,
    );
    record_jpeg(
        &store,
        &project.project_id,
        "ftp:high",
        "DCIM/100/IMG_3002.JPG",
        1100,
    );
    record_jpeg(
        &store,
        &project.project_id,
        "ftp:unassessed",
        "DCIM/100/IMG_3003.JPG",
        1200,
    );
    save_model_score_for_display_key(&store, &project.project_id, "IMG_3001", 42);
    save_model_score_for_display_key(&store, &project.project_id, "IMG_3002", 91);

    let page = store
        .asset_group_page(
            &project.project_id,
            AssetGroupQuery {
                sort: AssetGroupSort::ModelScore,
                ..AssetGroupQuery::default()
            },
            0,
            25,
        )
        .expect("asset groups should query");

    assert_eq!(page.groups[0].group_key, "IMG_3002");
    assert_eq!(page.groups[1].group_key, "IMG_3001");
    assert_eq!(page.groups[2].group_key, "IMG_3003");
}

#[test]
fn asset_groups_filter_technical_risk_collection() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Risk Collection")
        .expect("project should create");
    record_jpeg(
        &store,
        &project.project_id,
        "ftp:risk",
        "DCIM/100/IMG_4001.JPG",
        1000,
    );
    record_jpeg(
        &store,
        &project.project_id,
        "ftp:ok",
        "DCIM/100/IMG_4002.JPG",
        1100,
    );
    save_technical_gate_for_display_key(
        &store,
        &project.project_id,
        "IMG_4001",
        TechnicalGateStatus::Warn,
    );
    save_technical_gate_for_display_key(
        &store,
        &project.project_id,
        "IMG_4002",
        TechnicalGateStatus::Pass,
    );

    let page = store
        .asset_group_page(
            &project.project_id,
            AssetGroupQuery {
                collection: Some("technical_risk".to_string()),
                ..AssetGroupQuery::default()
            },
            0,
            25,
        )
        .expect("asset groups should query");

    assert_eq!(page.groups.len(), 1);
    assert_eq!(page.groups[0].group_key, "IMG_4001");
    assert_eq!(
        page.groups[0].technical_gate_status.as_deref(),
        Some("warn")
    );
}

#[test]
fn burst_summary_uses_best_model_score() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Burst Model Score")
        .expect("project should create");
    record_jpeg(
        &store,
        &project.project_id,
        "ftp:best",
        "DCIM/100/IMG_5001.JPG",
        1000,
    );
    record_jpeg(
        &store,
        &project.project_id,
        "ftp:soft",
        "DCIM/100/IMG_5002.JPG",
        1100,
    );
    let first_group = store
        .stored_asset_groups(&project.project_id)
        .expect("groups should query")
        .into_iter()
        .find(|group| group.display_key == "IMG_5001")
        .expect("first group should exist")
        .group_id;
    store
        .detect_bursts_for_asset_group(
            &project.project_id,
            &first_group,
            &BurstGroupingProfile::default(),
        )
        .expect("burst should detect");
    save_model_score_for_display_key(&store, &project.project_id, "IMG_5001", 92);
    save_model_score_for_display_key(&store, &project.project_id, "IMG_5002", 55);
    let selected_group = group_id_for_display_key(&store, &project.project_id, "IMG_5002");
    let burst_id = store
        .burst_group_for_asset_group(&selected_group)
        .expect("burst lookup")
        .expect("burst should exist")
        .burst_group_id;
    store
        .save_selection_recommendation(SelectionRecommendation {
            recommendation_id: "selected-low-score".to_string(),
            run_id: None,
            scope: SelectionRecommendationScope::BurstGroup,
            project_id: project.project_id.clone(),
            subject_id: burst_id,
            selected_asset_group_ids: vec![selected_group],
            candidate_asset_group_ids: Vec::new(),
            rejected_asset_group_ids: Vec::new(),
            source: SelectionSource::Llm,
            status: SelectionRecommendationStatus::Ready,
            confidence: 0.8,
            reason: "selected for stronger moment despite lower score".to_string(),
            created_at_ms: 12_000,
            updated_at_ms: 12_000,
        })
        .expect("recommendation should save");

    let page = store
        .asset_group_page(
            &project.project_id,
            AssetGroupQuery {
                sort: AssetGroupSort::ModelScore,
                ..AssetGroupQuery::default()
            },
            0,
            25,
        )
        .expect("asset groups should query");

    assert_eq!(
        vec!["IMG_5001", "IMG_5002"],
        page.groups
            .iter()
            .map(|group| group.group_key.as_str())
            .collect::<Vec<_>>()
    );
    for group in page.groups {
        assert_eq!(
            Some(55.0),
            group.burst.as_ref().and_then(|burst| burst.best_score)
        );
    }
}

fn save_model_score_for_display_key(
    store: &SqliteStore,
    project_id: &str,
    display_key: &str,
    score: i64,
) {
    let group_id = group_id_for_display_key(store, project_id, display_key);
    store
        .save_model_evaluation(ModelEvaluation {
            evaluation_id: format!("model-evaluation-{group_id}"),
            run_id: "run-model".to_string(),
            project_id: project_id.to_string(),
            asset_group_id: group_id,
            evaluator_kind: ModelEvaluatorKind::LocalStub,
            evaluator_version: "model-stub-v1".to_string(),
            status: ModelEvaluationStatus::Ready,
            score,
            tier: ModelEvaluationTier::from_score(score),
            selectable: score >= 40,
            summary: "model score for query ordering".to_string(),
            strengths: Vec::new(),
            weaknesses: Vec::new(),
            technical_warnings: Vec::new(),
            prompt_profile_id: Some("general-default".to_string()),
            prompt_version_id: Some("general-default-v1".to_string()),
            prompt_hash: Some("prompt-hash".to_string()),
            created_at_ms: 11_000 + score,
            updated_at_ms: 11_000 + score,
        })
        .expect("model evaluation should save");
}

fn save_technical_gate_for_display_key(
    store: &SqliteStore,
    project_id: &str,
    display_key: &str,
    gate_status: TechnicalGateStatus,
) {
    let group_id = group_id_for_display_key(store, project_id, display_key);
    store
        .save_technical_assessment(TechnicalAssessment {
            asset_group_id: group_id,
            assessor_version: "technical-v1".to_string(),
            status: TechnicalAssessmentStatus::Ready,
            gate_status,
            defect_flags: if gate_status == TechnicalGateStatus::Pass {
                Vec::new()
            } else {
                vec![TechnicalDefectFlag {
                    defect_type: TechnicalDefectType::Blur,
                    severity: TechnicalDefectSeverity::High,
                    confidence: 0.8,
                    metrics_json: None,
                    reason: "technical risk".to_string(),
                }]
            },
            preview_source: Some("jpeg".to_string()),
            visual_signature: None,
            analyzed_at_ms: 10_000,
        })
        .expect("technical assessment should save");
}

fn group_id_for_display_key(store: &SqliteStore, project_id: &str, display_key: &str) -> String {
    store
        .stored_asset_groups(project_id)
        .expect("groups should query")
        .into_iter()
        .find(|group| group.display_key == display_key)
        .expect("group should exist")
        .group_id
}

fn record_jpeg(
    store: &SqliteStore,
    project_id: &str,
    transfer_id: &str,
    original_path: &str,
    started_at_ms: i64,
) {
    store
        .record_transfer(
            project_id,
            completed_transfer(transfer_id, original_path, started_at_ms),
        )
        .expect("transfer should record");
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
        final_location: Some(StoredObjectLocation::local_path(final_filename)),
        size_bytes: 100,
        username: Some("z5".to_string()),
        remote_addr: Some("192.168.137.56".to_string()),
        source_name: Some("Nikon Z".to_string()),
        started_at_ms,
        completed_at_ms: Some(started_at_ms),
        error: None,
    }
}
