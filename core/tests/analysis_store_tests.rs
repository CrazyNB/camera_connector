use camera_connector_core::{
    ModelEvaluation, ModelEvaluationStatus, ModelEvaluationTier, ModelEvaluatorKind,
    QualityAnalysisStatus, QualityScore, ScopedSelectionRecommendation, SelectionRecommendation,
    SelectionRecommendationScope, SelectionRecommendationStatus, SelectionSource, SignalScore,
    SqliteStore, StrategyProfile, TechnicalAssessment, TechnicalAssessmentStatus,
    TechnicalDefectFlag, TechnicalDefectSeverity, TechnicalDefectType, TechnicalGateStatus,
};
use rusqlite::Connection;

#[test]
fn store_seeds_builtin_strategy_profiles() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");

    let profiles = store.strategy_profiles().expect("profiles should query");
    let ids = profiles
        .iter()
        .map(|profile| profile.profile_id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        ids,
        vec!["general", "conservative", "portrait", "action", "landscape"]
    );
    assert!(profiles.iter().all(|profile| profile.built_in));
    assert!(profiles
        .iter()
        .all(|profile| profile.weights.composition <= 0.12));
}

#[test]
fn store_upserts_strategy_profile() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let mut profile = StrategyProfile::general();
    profile.profile_id = "custom-balanced".to_string();
    profile.name = "Custom Balanced".to_string();
    profile.weights.sharpness = 0.5;

    let saved = store
        .save_strategy_profile(profile.clone())
        .expect("profile should save");
    let profiles = store.strategy_profiles().expect("profiles should query");

    assert_eq!(saved.profile_id, "custom-balanced");
    assert!(profiles
        .iter()
        .any(|value| value.profile_id == "custom-balanced" && value.weights.sharpness == 0.5));
}

#[test]
fn store_persists_quality_score_with_versions() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let score = QualityScore {
        asset_group_id: "group-1".to_string(),
        preview_source: Some("jpeg".to_string()),
        scorer_version: "local-v1".to_string(),
        analysis_status: QualityAnalysisStatus::Ready,
        exif_status: Some("missing".to_string()),
        capture_time_ms: None,
        sharpness: SignalScore::ready(0.81),
        exposure: SignalScore::ready(0.72),
        highlight_clipping_penalty: SignalScore::ready(0.08),
        shadow_clipping_penalty: SignalScore::ready(0.04),
        composition: SignalScore::ready(0.66),
        composition_confidence: 0.7,
        similarity_cluster_id: Some("similarity-1".to_string()),
        overall: 0.77,
        reasons: vec!["清晰度高".to_string(), "曝光均衡".to_string()],
        analyzed_at_ms: 1000,
    };

    store
        .save_quality_score(score.clone())
        .expect("score should save");
    let loaded = store
        .quality_score("group-1", "local-v1")
        .expect("score should query")
        .expect("score should exist");

    assert_eq!(loaded, score);
}

#[test]
fn store_persists_technical_assessment_with_defect_flags() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let assessment = TechnicalAssessment {
        asset_group_id: "group-technical".to_string(),
        assessor_version: "technical-v1".to_string(),
        status: TechnicalAssessmentStatus::Ready,
        gate_status: TechnicalGateStatus::Warn,
        defect_flags: vec![TechnicalDefectFlag {
            defect_type: TechnicalDefectType::Blur,
            severity: TechnicalDefectSeverity::High,
            confidence: 0.82,
            metrics_json: Some("{\"laplacian\":0.12}".to_string()),
            reason: "severe softness risk".to_string(),
        }],
        preview_source: Some("jpeg".to_string()),
        analyzed_at_ms: 3000,
    };

    store
        .save_technical_assessment(assessment.clone())
        .expect("technical assessment should save");
    let loaded = store
        .technical_assessments_for_asset_groups(&["group-technical".to_string()], "technical-v1")
        .expect("technical assessment should query");

    assert_eq!(loaded, vec![assessment]);
}

#[test]
fn store_persists_model_evaluation_with_photographic_score() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let evaluation = ModelEvaluation {
        evaluation_id: "evaluation-1".to_string(),
        run_id: "run-1".to_string(),
        project_id: "project-1".to_string(),
        asset_group_id: "group-model".to_string(),
        evaluator_kind: ModelEvaluatorKind::LocalStub,
        evaluator_version: "model-stub-v1".to_string(),
        status: ModelEvaluationStatus::Ready,
        score: 74,
        tier: ModelEvaluationTier::Good,
        selectable: true,
        summary: "usable photographic candidate".to_string(),
        strengths: vec!["clear subject".to_string()],
        weaknesses: vec!["flat light".to_string()],
        technical_warnings: vec!["minor clipping risk".to_string()],
        prompt_profile_id: Some("general-default".to_string()),
        prompt_version_id: Some("general-default-v1".to_string()),
        prompt_hash: Some("hash-1".to_string()),
        created_at_ms: 4000,
        updated_at_ms: 4000,
    };

    store
        .save_model_evaluation(evaluation.clone())
        .expect("model evaluation should save");
    let loaded = store
        .model_evaluations_for_asset_groups(&["group-model".to_string()], "model-stub-v1")
        .expect("model evaluation should query");

    assert_eq!(loaded, vec![evaluation]);
}

#[test]
fn store_migrates_dev_model_evaluation_columns() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let db_path = temp_dir.path().join("state.sqlite");
    {
        let connection = Connection::open(&db_path).expect("legacy db should open");
        connection
            .execute_batch(
                "
                CREATE TABLE model_evaluations (
                    evaluation_id TEXT PRIMARY KEY,
                    project_id TEXT NOT NULL,
                    asset_group_id TEXT NOT NULL,
                    evaluator_kind TEXT NOT NULL,
                    evaluator_version TEXT NOT NULL,
                    status TEXT NOT NULL,
                    score INTEGER NOT NULL,
                    tier TEXT NOT NULL,
                    selectable INTEGER NOT NULL,
                    summary TEXT NOT NULL,
                    strengths_json TEXT NOT NULL,
                    weaknesses_json TEXT NOT NULL,
                    technical_warnings_json TEXT NOT NULL,
                    created_at_ms INTEGER NOT NULL,
                    updated_at_ms INTEGER NOT NULL
                );
                ",
            )
            .expect("legacy table should create");
    }
    let store = SqliteStore::open(&db_path).expect("store should open");
    let evaluation = ModelEvaluation {
        evaluation_id: "evaluation-migrated".to_string(),
        run_id: "run-migrated".to_string(),
        project_id: "project-1".to_string(),
        asset_group_id: "group-migrated".to_string(),
        evaluator_kind: ModelEvaluatorKind::LocalStub,
        evaluator_version: "model-stub-v1".to_string(),
        status: ModelEvaluationStatus::Ready,
        score: 80,
        tier: ModelEvaluationTier::Good,
        selectable: true,
        summary: "migrated model evaluation".to_string(),
        strengths: Vec::new(),
        weaknesses: Vec::new(),
        technical_warnings: Vec::new(),
        prompt_profile_id: Some("general-default".to_string()),
        prompt_version_id: Some("general-default-v1".to_string()),
        prompt_hash: Some("hash-migrated".to_string()),
        created_at_ms: 4_500,
        updated_at_ms: 4_500,
    };

    store
        .save_model_evaluation(evaluation.clone())
        .expect("model evaluation should save after migration");
    let loaded = store
        .model_evaluations_for_asset_groups(&["group-migrated".to_string()], "model-stub-v1")
        .expect("model evaluation should query after migration");

    assert_eq!(loaded, vec![evaluation]);
}

#[test]
fn store_rejects_incompatible_dev_model_evaluation_column_type() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let db_path = temp_dir.path().join("state.sqlite");
    {
        let connection = Connection::open(&db_path).expect("legacy db should open");
        connection
            .execute_batch(
                "
                CREATE TABLE model_evaluations (
                    evaluation_id TEXT PRIMARY KEY,
                    run_id INTEGER NOT NULL DEFAULT 0,
                    project_id TEXT NOT NULL,
                    asset_group_id TEXT NOT NULL,
                    evaluator_kind TEXT NOT NULL,
                    evaluator_version TEXT NOT NULL,
                    status TEXT NOT NULL,
                    score INTEGER NOT NULL,
                    tier TEXT NOT NULL,
                    selectable INTEGER NOT NULL,
                    summary TEXT NOT NULL,
                    strengths_json TEXT NOT NULL,
                    weaknesses_json TEXT NOT NULL,
                    technical_warnings_json TEXT NOT NULL,
                    prompt_profile_id TEXT,
                    prompt_version_id TEXT,
                    prompt_hash TEXT,
                    created_at_ms INTEGER NOT NULL,
                    updated_at_ms INTEGER NOT NULL
                );
                ",
            )
            .expect("legacy table should create");
    }

    let error = SqliteStore::open(&db_path).expect_err("store should reject incompatible schema");

    assert!(error
        .to_string()
        .contains("model_evaluations.run_id must have TEXT affinity"));
}

#[test]
fn store_persists_scoped_project_recommendation() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let recommendation = ScopedSelectionRecommendation {
        recommendation_id: "scoped-recommendation-1".to_string(),
        run_id: Some("run-1".to_string()),
        scope: SelectionRecommendationScope::Project,
        project_id: "project-1".to_string(),
        subject_id: "project-1".to_string(),
        selected_asset_group_ids: vec!["group-good".to_string()],
        candidate_asset_group_ids: vec!["group-candidate".to_string()],
        rejected_asset_group_ids: vec!["group-reject".to_string()],
        source: SelectionSource::LocalRule,
        status: SelectionRecommendationStatus::Ready,
        confidence: 0.72,
        reason: "project model selects".to_string(),
        created_at_ms: 5000,
        updated_at_ms: 5000,
    };

    store
        .save_scoped_selection_recommendation(recommendation.clone())
        .expect("scoped recommendation should save");
    let loaded = store
        .latest_scoped_selection_recommendation(
            "project-1",
            SelectionRecommendationScope::Project,
            "project-1",
        )
        .expect("scoped recommendation should query")
        .expect("scoped recommendation should exist");

    assert_eq!(loaded, recommendation);
}

#[test]
fn store_persists_recommendation_with_versions() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let recommendation = SelectionRecommendation {
        recommendation_id: "recommendation-1".to_string(),
        burst_group_id: "burst-1".to_string(),
        strategy_profile_id: "general".to_string(),
        scorer_version: "local-v1".to_string(),
        strategy_version: "strategy-v1".to_string(),
        grouping_version: 3,
        best_asset_group_id: Some("group-best".to_string()),
        alternate_asset_group_ids: vec!["group-alt".to_string()],
        low_score_asset_group_ids: vec!["group-low".to_string()],
        near_duplicate_asset_group_ids: vec!["group-dup".to_string()],
        source: SelectionSource::LocalCv,
        status: SelectionRecommendationStatus::Ready,
        confidence: 0.86,
        reasons: vec!["本组最清晰".to_string()],
        llm_review_id: None,
        created_at_ms: 2000,
        updated_at_ms: 2000,
    };

    store
        .save_selection_recommendation(recommendation.clone())
        .expect("recommendation should save");
    let loaded = store
        .latest_selection_recommendation("burst-1", "general")
        .expect("recommendation should query")
        .expect("recommendation should exist");

    assert_eq!(loaded, recommendation);
}
