use camera_connector_core::{
    QualityAnalysisStatus, QualityScore, SelectionRecommendation, SelectionRecommendationStatus,
    SelectionSource, SignalScore, SqliteStore, StrategyProfile,
};

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
