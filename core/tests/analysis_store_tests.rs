use camera_connector_core::{
    ModelEvaluation, ModelEvaluationStatus, ModelEvaluationTier, ModelEvaluatorKind,
    SelectionRecommendation, SelectionRecommendationScope, SelectionRecommendationStatus,
    SelectionSource, SqliteStore, TechnicalAssessment, TechnicalAssessmentStatus,
    TechnicalDefectFlag, TechnicalDefectSeverity, TechnicalDefectType, TechnicalGateStatus,
};

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
        visual_signature: Some("ahash-v1:1234567890abcdef".to_string()),
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
fn store_persists_model_project_recommendation() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let recommendation = SelectionRecommendation {
        recommendation_id: "recommendation-1".to_string(),
        run_id: Some("run-1".to_string()),
        scope: SelectionRecommendationScope::Project,
        project_id: "project-1".to_string(),
        subject_id: "project-1".to_string(),
        selected_asset_group_ids: vec!["group-good".to_string()],
        candidate_asset_group_ids: vec!["group-candidate".to_string()],
        rejected_asset_group_ids: vec!["group-reject".to_string()],
        source: SelectionSource::Imported,
        status: SelectionRecommendationStatus::Ready,
        confidence: 0.72,
        reason: "project model recommendation".to_string(),
        created_at_ms: 5000,
        updated_at_ms: 5000,
    };

    store
        .save_selection_recommendation(recommendation.clone())
        .expect("model recommendation should save");
    let loaded = store
        .latest_selection_recommendation(
            "project-1",
            SelectionRecommendationScope::Project,
            "project-1",
        )
        .expect("model recommendation should query")
        .expect("model recommendation should exist");

    assert_eq!(loaded, recommendation);
}
