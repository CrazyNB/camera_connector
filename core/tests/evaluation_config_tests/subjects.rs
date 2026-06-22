use super::*;

#[test]
fn evaluation_config_tests_subject_assessment_save_query_round_trips_portrait_face_assessment() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Subject Project")
        .expect("project should create");
    let group_id = create_subject_asset_group(
        &store,
        &project.project_id,
        "ftp:subject-roundtrip",
        "DCIM/100/SUBJECT_0001.JPG",
    );
    let assessment = SubjectAssessment {
        assessment_id: "assessment-face-1".to_string(),
        project_id: project.project_id.clone(),
        asset_group_id: group_id.clone(),
        subject_type: "face".to_string(),
        detector_kind: "android_mlkit".to_string(),
        detector_version: "mlkit-face-v1".to_string(),
        status: EvaluationRunStatus::Ready,
        gate_status: "warn".to_string(),
        regions_json: "[{\"x\":10,\"y\":20,\"w\":80,\"h\":90}]".to_string(),
        signals_json: "{\"eyes\":\"open\",\"sharpness\":0.72}".to_string(),
        summary: "Face is usable with mild softness.".to_string(),
        created_at_ms: 4000,
        updated_at_ms: 4100,
    };

    store
        .save_subject_assessment(assessment.clone())
        .expect("assessment should save");
    let loaded = store
        .subject_assessments_for_asset_groups(&project.project_id, &[group_id])
        .expect("assessments should query");

    assert_eq!(loaded, vec![assessment]);
}

#[test]
fn evaluation_config_tests_subject_assessment_rejects_missing_or_wrong_project_asset_group() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let first_project = store
        .create_project("Subject First Project")
        .expect("first project should create");
    let second_project = store
        .create_project("Subject Second Project")
        .expect("second project should create");
    let first_group_id = create_subject_asset_group(
        &store,
        &first_project.project_id,
        "ftp:subject-wrong-project",
        "DCIM/100/SUBJECT_1001.JPG",
    );

    let missing_group = subject_assessment_for(
        &first_project.project_id,
        "missing-group",
        "assessment-missing-group",
    );
    let wrong_project = subject_assessment_for(
        &second_project.project_id,
        &first_group_id,
        "assessment-wrong-project",
    );

    assert!(store.save_subject_assessment(missing_group).is_err());
    assert!(store.save_subject_assessment(wrong_project).is_err());
}

#[test]
fn evaluation_config_tests_subject_assessment_id_cannot_move_between_asset_groups() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Subject Conflict Project")
        .expect("project should create");
    let first_group_id = create_subject_asset_group(
        &store,
        &project.project_id,
        "ftp:subject-conflict-a",
        "DCIM/100/SUBJECT_2001.JPG",
    );
    let second_group_id = create_subject_asset_group(
        &store,
        &project.project_id,
        "ftp:subject-conflict-b",
        "DCIM/101/SUBJECT_2002.JPG",
    );
    let first =
        subject_assessment_for(&project.project_id, &first_group_id, "assessment-stable-id");
    let moved = subject_assessment_for(
        &project.project_id,
        &second_group_id,
        "assessment-stable-id",
    );

    store
        .save_subject_assessment(first)
        .expect("first assessment should save");

    assert!(store.save_subject_assessment(moved).is_err());
}

#[test]
fn evaluation_config_tests_subject_assessment_requires_regions_array_and_signals_object() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Subject Json Project")
        .expect("project should create");
    let group_id = create_subject_asset_group(
        &store,
        &project.project_id,
        "ftp:subject-json",
        "DCIM/100/SUBJECT_3001.JPG",
    );

    let mut invalid_regions =
        subject_assessment_for(&project.project_id, &group_id, "assessment-invalid-regions");
    invalid_regions.regions_json = "{\"x\":10}".to_string();
    let mut invalid_signals =
        subject_assessment_for(&project.project_id, &group_id, "assessment-invalid-signals");
    invalid_signals.signals_json = "[\"closed_eyes\"]".to_string();
    let mut malformed_regions = subject_assessment_for(
        &project.project_id,
        &group_id,
        "assessment-malformed-regions",
    );
    malformed_regions.regions_json = "not-json".to_string();

    assert!(store.save_subject_assessment(invalid_regions).is_err());
    assert!(store.save_subject_assessment(invalid_signals).is_err());
    assert!(store.save_subject_assessment(malformed_regions).is_err());
}

#[test]
fn evaluation_config_tests_general_projects_do_not_schedule_portrait_subject_assessment() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("General Subject Scheduling")
        .expect("project should create");

    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:subject-general", "DCIM/100/GENERAL_0001.JPG", 1000),
        )
        .expect("transfer should record");
    let jobs = store
        .claim_analysis_jobs(i64::MAX, 10)
        .expect("jobs should claim");

    assert!(jobs
        .iter()
        .all(|job| job.job_type != AnalysisJobType::AssessPortraitSubject));
}

#[test]
fn evaluation_config_tests_portrait_projects_schedule_portrait_subject_assessment() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Portrait Subject Scheduling")
        .expect("project should create");
    let mut settings = base_project_settings(&project.project_id);
    settings.scene_profile = SceneProfile::Portrait;
    store
        .save_project_evaluation_settings(settings)
        .expect("settings should save");

    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:subject-portrait", "DCIM/100/PORTRAIT_0001.JPG", 1000),
        )
        .expect("transfer should record");
    let jobs = store
        .claim_analysis_jobs(i64::MAX, 10)
        .expect("jobs should claim");

    assert!(jobs
        .iter()
        .any(|job| job.job_type == AnalysisJobType::AssessPortraitSubject));
}

#[test]
fn evaluation_config_tests_service_reports_portrait_subject_assessment_schedule_condition() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Service Subject Scheduling")
        .expect("project should create");

    assert!(!service
        .should_schedule_subject_assessment(&project.project_id)
        .expect("schedule condition should load"));

    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .expect("settings should query")
        .expect("settings should exist");
    settings.scene_profile = SceneProfile::Portrait;
    service
        .save_project_evaluation_settings(settings)
        .expect("settings should save");

    assert!(service
        .should_schedule_subject_assessment(&project.project_id)
        .expect("schedule condition should load"));
}
