use camera_connector_core::{
    evaluate_asset_group_with_stub, ModelEvaluationStatus, ModelEvaluationTier, ModelEvaluatorKind,
    TechnicalAssessment, TechnicalAssessmentStatus, TechnicalGateStatus,
};

#[test]
fn stub_evaluation_scores_passed_asset_group_as_good_candidate() {
    let evaluation = evaluate_asset_group_with_stub(
        "project-1",
        "group-pass",
        &assessment("group-pass", TechnicalGateStatus::Pass),
        5_000,
    );

    assert_eq!(evaluation.asset_group_id, "group-pass");
    assert!(evaluation.run_id.starts_with("evaluation-run-"));
    assert_eq!(evaluation.evaluator_kind, ModelEvaluatorKind::LocalStub);
    assert_eq!(evaluation.status, ModelEvaluationStatus::Ready);
    assert_eq!(evaluation.score, 72);
    assert_eq!(evaluation.tier, ModelEvaluationTier::Good);
    assert!(evaluation.selectable);
    assert_eq!(evaluation.prompt_pack_id, None);
    assert_eq!(evaluation.prompt_pack_version, None);
    assert_eq!(evaluation.prompt_hash, None);
}

#[test]
fn stub_evaluation_rejects_technical_gate_rejects() {
    let evaluation = evaluate_asset_group_with_stub(
        "project-1",
        "group-reject",
        &assessment("group-reject", TechnicalGateStatus::Reject),
        5_000,
    );

    assert_eq!(evaluation.status, ModelEvaluationStatus::Ready);
    assert_eq!(evaluation.score, 20);
    assert_eq!(evaluation.tier, ModelEvaluationTier::Reject);
    assert!(!evaluation.selectable);
}

#[test]
fn stub_evaluation_skips_unsupported_assessments() {
    let evaluation = evaluate_asset_group_with_stub(
        "project-1",
        "group-unsupported",
        &assessment("group-unsupported", TechnicalGateStatus::Unsupported),
        5_000,
    );

    assert_eq!(evaluation.status, ModelEvaluationStatus::Skipped);
    assert_eq!(evaluation.score, 0);
    assert!(!evaluation.selectable);
}

fn assessment(asset_group_id: &str, gate_status: TechnicalGateStatus) -> TechnicalAssessment {
    TechnicalAssessment {
        asset_group_id: asset_group_id.to_string(),
        assessor_version: "technical-v1".to_string(),
        status: TechnicalAssessmentStatus::Ready,
        gate_status,
        defect_flags: Vec::new(),
        preview_source: Some("test".to_string()),
        visual_signature: None,
        analyzed_at_ms: 4_000,
    }
}
