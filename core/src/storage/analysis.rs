use super::{
    current_time_ms, ensure_project_exists, project_by_id, sqlite_data_error, SqliteStore,
};
use crate::{
    CvPolicy, EvaluationRun, EvaluationRunStatus, EvaluationRunTrigger, EvaluationRunType,
    ModelEvaluation, ModelEvaluationStatus, ModelEvaluationTier, ModelEvaluatorKind,
    ModelProviderKind, ProjectEvaluationSettings, ProjectRecommendationMode, ReceivedAssetGroup,
    ReceivedAssetTechnicalDefectSummary, Result, SceneProfile, SelectionRecommendation,
    SelectionRecommendationScope, SelectionRecommendationStatus, SelectionSource,
    TechnicalAssessment, TechnicalAssessmentStatus, TechnicalGateStatus,
};
use rusqlite::{params, Connection, OptionalExtension, Row};
#[path = "analysis_json.rs"]
mod analysis_json;
use analysis_json::{
    string_vec_from_json, string_vec_json, technical_assessment_policy_from_json,
    technical_assessment_policy_json, technical_defect_flags_from_json,
    technical_defect_flags_json,
};
impl SqliteStore {
    pub fn project_evaluation_settings(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectEvaluationSettings>> {
        self.with_connection(|connection| {
            project_evaluation_settings_for_project(connection, project_id)
        })
    }
    pub fn save_project_evaluation_settings(
        &self,
        settings: ProjectEvaluationSettings,
    ) -> Result<ProjectEvaluationSettings> {
        self.with_connection(|connection| {
            save_project_evaluation_settings_for_connection(connection, settings)
        })
    }
    pub fn save_evaluation_run(&self, run: EvaluationRun) -> Result<EvaluationRun> {
        self.with_connection(|connection| save_evaluation_run_for_connection(connection, run))
    }
    pub fn latest_evaluation_run(
        &self,
        project_id: &str,
        run_type: EvaluationRunType,
    ) -> Result<Option<EvaluationRun>> {
        self.with_read_connection(|connection| {
            latest_evaluation_run(connection, project_id, run_type)
        })
    }
    pub fn save_technical_assessment(
        &self,
        assessment: TechnicalAssessment,
    ) -> Result<TechnicalAssessment> {
        self.with_connection(|connection| {
            save_technical_assessment_for_connection(connection, assessment)
        })
    }
    pub fn technical_assessments_for_asset_groups(
        &self,
        asset_group_ids: &[String],
        assessor_version: &str,
    ) -> Result<Vec<TechnicalAssessment>> {
        self.with_read_connection(|connection| {
            technical_assessments_for_asset_group_ids(connection, asset_group_ids, assessor_version)
        })
    }
    pub fn save_model_evaluation(&self, evaluation: ModelEvaluation) -> Result<ModelEvaluation> {
        self.with_connection(|connection| {
            save_model_evaluation_for_connection(connection, evaluation)
        })
    }
    pub fn model_evaluations_for_asset_groups(
        &self,
        asset_group_ids: &[String],
        evaluator_version: &str,
    ) -> Result<Vec<ModelEvaluation>> {
        self.with_read_connection(|connection| {
            model_evaluations_for_asset_group_ids(connection, asset_group_ids, evaluator_version)
        })
    }
    pub fn save_selection_recommendation(
        &self,
        recommendation: SelectionRecommendation,
    ) -> Result<SelectionRecommendation> {
        self.with_connection(|connection| {
            save_selection_recommendation_for_connection(connection, recommendation)
        })
    }
    pub fn latest_selection_recommendation(
        &self,
        project_id: &str,
        scope: SelectionRecommendationScope,
        subject_id: &str,
    ) -> Result<Option<SelectionRecommendation>> {
        self.with_read_connection(|connection| {
            latest_selection_recommendation_for_connection(
                connection, project_id, scope, subject_id,
            )
        })
    }
}
pub(super) fn apply_technical_summary(
    connection: &Connection,
    asset_group_id: &str,
    group: &mut ReceivedAssetGroup,
) -> std::result::Result<(), rusqlite::Error> {
    let Some(assessment) = latest_technical_assessment_for_asset_group(connection, asset_group_id)?
    else {
        return Ok(());
    };
    group.technical_status = Some(assessment.status.as_str().to_string());
    group.technical_gate_status = Some(assessment.gate_status.as_str().to_string());
    group.technical_defects = assessment
        .defect_flags
        .into_iter()
        .map(|flag| ReceivedAssetTechnicalDefectSummary {
            defect_type: flag.defect_type.as_str().to_string(),
            severity: flag.severity.as_str().to_string(),
            confidence: flag.confidence,
            reason: (!flag.reason.is_empty()).then_some(flag.reason),
        })
        .collect();
    Ok(())
}
fn latest_technical_assessment_for_asset_group(
    connection: &Connection,
    asset_group_id: &str,
) -> std::result::Result<Option<TechnicalAssessment>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT asset_group_id, assessor_version, status, gate_status, defect_flags_json,
                    preview_source, visual_signature, analyzed_at_ms
             FROM technical_assessments
             WHERE asset_group_id = ?1
             ORDER BY analyzed_at_ms DESC, assessor_version DESC
             LIMIT 1",
            params![asset_group_id],
            technical_assessment_from_row,
        )
        .optional()
}
pub(super) fn apply_model_evaluation_summary(
    connection: &Connection,
    asset_group_id: &str,
    group: &mut ReceivedAssetGroup,
) -> std::result::Result<(), rusqlite::Error> {
    let Some(evaluation) = latest_any_model_evaluation_for_asset_group(connection, asset_group_id)?
    else {
        return Ok(());
    };
    group.model_status = Some(evaluation.status.as_str().to_string());
    group.model_score = Some(evaluation.score);
    group.model_tier = Some(evaluation.tier.as_str().to_string());
    group.model_evaluator_kind = Some(evaluation.evaluator_kind.as_str().to_string());
    group.model_summary = Some(evaluation.summary);
    Ok(())
}
fn latest_any_model_evaluation_for_asset_group(
    connection: &Connection,
    asset_group_id: &str,
) -> std::result::Result<Option<ModelEvaluation>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT evaluation_id, project_id, asset_group_id, evaluator_kind, evaluator_version,
                    run_id, status, score, tier, selectable, summary, strengths_json, weaknesses_json,
                    technical_warnings_json, prompt_pack_id, prompt_pack_version, prompt_hash,
                    created_at_ms, updated_at_ms
             FROM model_evaluations
             WHERE asset_group_id = ?1
             ORDER BY updated_at_ms DESC, evaluator_version DESC
             LIMIT 1",
            params![asset_group_id],
            model_evaluation_from_row,
        )
        .optional()
}
pub(super) fn is_model_selected_asset_group(
    connection: &Connection,
    project_id: &str,
    asset_group_id: &str,
    burst_group_id: Option<&str>,
) -> std::result::Result<bool, rusqlite::Error> {
    if selection_recommendation_selects(
        latest_selection_recommendation_for_connection(
            connection,
            project_id,
            SelectionRecommendationScope::Project,
            project_id,
        )?,
        asset_group_id,
    ) {
        return Ok(true);
    }
    if let Some(burst_group_id) = burst_group_id {
        return Ok(selection_recommendation_selects(
            latest_selection_recommendation_for_connection(
                connection,
                project_id,
                SelectionRecommendationScope::BurstGroup,
                burst_group_id,
            )?,
            asset_group_id,
        ));
    }
    Ok(false)
}
fn selection_recommendation_selects(
    recommendation: Option<SelectionRecommendation>,
    asset_group_id: &str,
) -> bool {
    recommendation
        .filter(|value| value.status == SelectionRecommendationStatus::Ready)
        .map(|value| {
            value
                .selected_asset_group_ids
                .iter()
                .any(|selected| selected == asset_group_id)
        })
        .unwrap_or(false)
}
fn project_evaluation_settings_for_project(
    connection: &Connection,
    project_id: &str,
) -> std::result::Result<Option<ProjectEvaluationSettings>, rusqlite::Error> {
    if project_by_id(connection, project_id)?.is_none() {
        return Ok(None);
    }
    if let Some(settings) = project_evaluation_settings_by_project_id(connection, project_id)? {
        return Ok(Some(settings));
    }
    let settings =
        ProjectEvaluationSettings::default_for_project(project_id.to_string(), current_time_ms());
    save_project_evaluation_settings_for_connection(connection, settings).map(Some)
}
fn project_evaluation_settings_by_project_id(
    connection: &Connection,
    project_id: &str,
) -> std::result::Result<Option<ProjectEvaluationSettings>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT project_id, auto_evaluate_on_upload,
                    auto_burst_recommendation_enabled, project_recommendation_mode,
                    prompt_pack_id, model_provider_settings_id, scene_profile, cv_policy,
                    cv_policy_overrides_json, allow_risky_model_selects, max_image_side,
                    batch_size, updated_at_ms
             FROM project_evaluation_settings
             WHERE project_id = ?1",
            params![project_id],
            project_evaluation_settings_from_row,
        )
        .optional()
}
pub(super) fn save_project_evaluation_settings_for_connection(
    connection: &Connection,
    settings: ProjectEvaluationSettings,
) -> std::result::Result<ProjectEvaluationSettings, rusqlite::Error> {
    ensure_project_exists(connection, &settings.project_id)?;
    validate_project_evaluation_settings(connection, &settings)?;
    connection.execute(
        "INSERT INTO project_evaluation_settings (
            project_id, auto_evaluate_on_upload,
            auto_burst_recommendation_enabled, project_recommendation_mode, prompt_pack_id,
            model_provider_settings_id, scene_profile, cv_policy, cv_policy_overrides_json,
            allow_risky_model_selects, max_image_side, batch_size, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
         ON CONFLICT(project_id) DO UPDATE SET
            auto_evaluate_on_upload = excluded.auto_evaluate_on_upload,
            auto_burst_recommendation_enabled = excluded.auto_burst_recommendation_enabled,
            project_recommendation_mode = excluded.project_recommendation_mode,
            prompt_pack_id = excluded.prompt_pack_id,
            model_provider_settings_id = excluded.model_provider_settings_id,
            scene_profile = excluded.scene_profile,
            cv_policy = excluded.cv_policy,
            cv_policy_overrides_json = excluded.cv_policy_overrides_json,
            allow_risky_model_selects = excluded.allow_risky_model_selects,
            max_image_side = excluded.max_image_side,
            batch_size = excluded.batch_size,
            updated_at_ms = excluded.updated_at_ms",
        params![
            &settings.project_id,
            settings.auto_evaluate_on_upload,
            settings.auto_burst_recommendation_enabled,
            settings.project_recommendation_mode.as_str(),
            settings.prompt_pack_id.as_deref(),
            settings.model_provider_settings_id.as_deref(),
            settings.scene_profile.as_str(),
            settings.cv_policy.as_str(),
            technical_assessment_policy_json(settings.cv_policy_overrides.as_ref())?,
            settings.allow_risky_model_selects,
            settings.max_image_side,
            settings.batch_size,
            settings.updated_at_ms,
        ],
    )?;
    project_evaluation_settings_by_project_id(connection, &settings.project_id)?.ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("project evaluation settings not found".to_string())
    })
}
fn validate_project_evaluation_settings(
    _connection: &Connection,
    settings: &ProjectEvaluationSettings,
) -> std::result::Result<(), rusqlite::Error> {
    if settings
        .prompt_pack_id
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err(sqlite_data_error("prompt pack id cannot be blank"));
    }
    Ok(())
}
fn project_evaluation_settings_from_row(
    row: &Row<'_>,
) -> std::result::Result<ProjectEvaluationSettings, rusqlite::Error> {
    let project_recommendation_mode: String = row.get(3)?;
    let scene_profile: String = row.get(6)?;
    let cv_policy: String = row.get(7)?;
    let cv_policy_overrides_json: Option<String> = row.get(8)?;
    Ok(ProjectEvaluationSettings {
        project_id: row.get(0)?,
        auto_evaluate_on_upload: row.get(1)?,
        auto_burst_recommendation_enabled: row.get(2)?,
        project_recommendation_mode: ProjectRecommendationMode::from_str(
            &project_recommendation_mode,
        ),
        prompt_pack_id: row.get(4)?,
        model_provider_settings_id: row.get(5)?,
        scene_profile: SceneProfile::from_str(&scene_profile),
        cv_policy: CvPolicy::from_str(&cv_policy),
        cv_policy_overrides: technical_assessment_policy_from_json(cv_policy_overrides_json)?,
        allow_risky_model_selects: row.get(9)?,
        max_image_side: row.get(10)?,
        batch_size: row.get(11)?,
        updated_at_ms: row.get(12)?,
    })
}
fn save_evaluation_run_for_connection(
    connection: &Connection,
    run: EvaluationRun,
) -> std::result::Result<EvaluationRun, rusqlite::Error> {
    ensure_project_exists(connection, &run.project_id)?;
    connection.execute(
        "INSERT INTO evaluation_runs (
            run_id, project_id, run_type, trigger, status, provider_kind, provider_model,
            prompt_pack_id, prompt_pack_version, prompt_hash, settings_snapshot_json,
            error_message, started_at_ms, completed_at_ms, created_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
         ON CONFLICT(run_id) DO UPDATE SET
            project_id = excluded.project_id,
            run_type = excluded.run_type,
            trigger = excluded.trigger,
            status = excluded.status,
            provider_kind = excluded.provider_kind,
            provider_model = excluded.provider_model,
            prompt_pack_id = excluded.prompt_pack_id,
            prompt_pack_version = excluded.prompt_pack_version,
            prompt_hash = excluded.prompt_hash,
            settings_snapshot_json = excluded.settings_snapshot_json,
            error_message = excluded.error_message,
            started_at_ms = excluded.started_at_ms,
            completed_at_ms = excluded.completed_at_ms,
            created_at_ms = excluded.created_at_ms",
        params![
            run.run_id,
            run.project_id,
            run.run_type.as_str(),
            run.trigger.as_str(),
            run.status.as_str(),
            run.provider_kind.as_str(),
            run.provider_model,
            run.prompt_pack_id,
            run.prompt_pack_version,
            run.prompt_hash,
            run.settings_snapshot_json,
            run.error_message,
            run.started_at_ms,
            run.completed_at_ms,
            run.created_at_ms,
        ],
    )?;
    evaluation_run_by_id(connection, &run.run_id)?.ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("evaluation run not found".to_string())
    })
}
fn evaluation_run_by_id(
    connection: &Connection,
    run_id: &str,
) -> std::result::Result<Option<EvaluationRun>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT run_id, project_id, run_type, trigger, status, provider_kind, provider_model,
                    prompt_pack_id, prompt_pack_version, prompt_hash, settings_snapshot_json,
                    error_message, started_at_ms, completed_at_ms, created_at_ms
             FROM evaluation_runs
             WHERE run_id = ?1",
            params![run_id],
            evaluation_run_from_row,
        )
        .optional()
}
fn latest_evaluation_run(
    connection: &Connection,
    project_id: &str,
    run_type: EvaluationRunType,
) -> std::result::Result<Option<EvaluationRun>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT run_id, project_id, run_type, trigger, status, provider_kind, provider_model,
                    prompt_pack_id, prompt_pack_version, prompt_hash, settings_snapshot_json,
                    error_message, started_at_ms, completed_at_ms, created_at_ms
             FROM evaluation_runs
             WHERE project_id = ?1 AND run_type = ?2
             ORDER BY created_at_ms DESC, run_id DESC
             LIMIT 1",
            params![project_id, run_type.as_str()],
            evaluation_run_from_row,
        )
        .optional()
}
fn evaluation_run_from_row(row: &Row<'_>) -> std::result::Result<EvaluationRun, rusqlite::Error> {
    let run_type: String = row.get(2)?;
    let trigger: String = row.get(3)?;
    let status: String = row.get(4)?;
    let provider_kind: String = row.get(5)?;
    Ok(EvaluationRun {
        run_id: row.get(0)?,
        project_id: row.get(1)?,
        run_type: EvaluationRunType::from_str(&run_type),
        trigger: EvaluationRunTrigger::from_str(&trigger),
        status: EvaluationRunStatus::from_str(&status),
        provider_kind: ModelProviderKind::from_str(&provider_kind),
        provider_model: row.get(6)?,
        prompt_pack_id: row.get(7)?,
        prompt_pack_version: row.get(8)?,
        prompt_hash: row.get(9)?,
        settings_snapshot_json: row.get(10)?,
        error_message: row.get(11)?,
        started_at_ms: row.get(12)?,
        completed_at_ms: row.get(13)?,
        created_at_ms: row.get(14)?,
    })
}
fn save_technical_assessment_for_connection(
    connection: &Connection,
    assessment: TechnicalAssessment,
) -> std::result::Result<TechnicalAssessment, rusqlite::Error> {
    connection.execute(
        "INSERT INTO technical_assessments (
            asset_group_id, assessor_version, status, gate_status, defect_flags_json,
            preview_source, visual_signature, analyzed_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(asset_group_id, assessor_version) DO UPDATE SET
            status = excluded.status,
            gate_status = excluded.gate_status,
            defect_flags_json = excluded.defect_flags_json,
            preview_source = excluded.preview_source,
            visual_signature = excluded.visual_signature,
            analyzed_at_ms = excluded.analyzed_at_ms",
        params![
            assessment.asset_group_id,
            assessment.assessor_version,
            assessment.status.as_str(),
            assessment.gate_status.as_str(),
            technical_defect_flags_json(&assessment.defect_flags)?,
            assessment.preview_source,
            assessment.visual_signature,
            assessment.analyzed_at_ms,
        ],
    )?;
    technical_assessment_by_key(
        connection,
        &assessment.asset_group_id,
        &assessment.assessor_version,
    )?
    .ok_or_else(|| sqlite_data_error("technical assessment not found"))
}
fn technical_assessment_by_key(
    connection: &Connection,
    asset_group_id: &str,
    assessor_version: &str,
) -> std::result::Result<Option<TechnicalAssessment>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT asset_group_id, assessor_version, status, gate_status, defect_flags_json,
                    preview_source, visual_signature, analyzed_at_ms
             FROM technical_assessments
             WHERE asset_group_id = ?1 AND assessor_version = ?2",
            params![asset_group_id, assessor_version],
            technical_assessment_from_row,
        )
        .optional()
}
pub(super) fn technical_assessments_for_asset_group_ids(
    connection: &Connection,
    asset_group_ids: &[String],
    assessor_version: &str,
) -> std::result::Result<Vec<TechnicalAssessment>, rusqlite::Error> {
    let mut assessments = Vec::new();
    for asset_group_id in asset_group_ids {
        if let Some(assessment) =
            technical_assessment_by_key(connection, asset_group_id, assessor_version)?
        {
            assessments.push(assessment);
        }
    }
    Ok(assessments)
}
fn technical_assessment_from_row(
    row: &Row<'_>,
) -> std::result::Result<TechnicalAssessment, rusqlite::Error> {
    let status: String = row.get(2)?;
    let gate_status: String = row.get(3)?;
    Ok(TechnicalAssessment {
        asset_group_id: row.get(0)?,
        assessor_version: row.get(1)?,
        status: TechnicalAssessmentStatus::from_str(&status),
        gate_status: TechnicalGateStatus::from_str(&gate_status),
        defect_flags: technical_defect_flags_from_json(row.get::<_, String>(4)?)?,
        preview_source: row.get(5)?,
        visual_signature: row.get(6)?,
        analyzed_at_ms: row.get(7)?,
    })
}
fn save_model_evaluation_for_connection(
    connection: &Connection,
    evaluation: ModelEvaluation,
) -> std::result::Result<ModelEvaluation, rusqlite::Error> {
    connection.execute(
        "INSERT INTO model_evaluations (
            evaluation_id, run_id, project_id, asset_group_id, evaluator_kind, evaluator_version,
            status, score, tier, selectable, summary, strengths_json, weaknesses_json,
            technical_warnings_json, prompt_pack_id, prompt_pack_version, prompt_hash,
            created_at_ms, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
         ON CONFLICT(evaluation_id) DO UPDATE SET
            run_id = excluded.run_id,
            project_id = excluded.project_id,
            asset_group_id = excluded.asset_group_id,
            evaluator_kind = excluded.evaluator_kind,
            evaluator_version = excluded.evaluator_version,
            status = excluded.status,
            score = excluded.score,
            tier = excluded.tier,
            selectable = excluded.selectable,
            summary = excluded.summary,
            strengths_json = excluded.strengths_json,
            weaknesses_json = excluded.weaknesses_json,
            technical_warnings_json = excluded.technical_warnings_json,
            prompt_pack_id = excluded.prompt_pack_id,
            prompt_pack_version = excluded.prompt_pack_version,
            prompt_hash = excluded.prompt_hash,
            updated_at_ms = excluded.updated_at_ms",
        params![
            evaluation.evaluation_id,
            evaluation.run_id,
            evaluation.project_id,
            evaluation.asset_group_id,
            evaluation.evaluator_kind.as_str(),
            evaluation.evaluator_version,
            evaluation.status.as_str(),
            evaluation.score,
            evaluation.tier.as_str(),
            evaluation.selectable,
            evaluation.summary,
            string_vec_json(&evaluation.strengths)?,
            string_vec_json(&evaluation.weaknesses)?,
            string_vec_json(&evaluation.technical_warnings)?,
            evaluation.prompt_pack_id,
            evaluation.prompt_pack_version,
            evaluation.prompt_hash,
            evaluation.created_at_ms,
            evaluation.updated_at_ms,
        ],
    )?;
    model_evaluation_by_id(connection, &evaluation.evaluation_id)?
        .ok_or_else(|| sqlite_data_error("model evaluation not found"))
}
fn model_evaluation_by_id(
    connection: &Connection,
    evaluation_id: &str,
) -> std::result::Result<Option<ModelEvaluation>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT evaluation_id, project_id, asset_group_id, evaluator_kind, evaluator_version,
                    run_id, status, score, tier, selectable, summary, strengths_json, weaknesses_json,
                    technical_warnings_json, prompt_pack_id, prompt_pack_version, prompt_hash,
                    created_at_ms, updated_at_ms
             FROM model_evaluations
             WHERE evaluation_id = ?1",
            params![evaluation_id],
            model_evaluation_from_row,
        )
        .optional()
}
fn latest_model_evaluation_for_asset_group(
    connection: &Connection,
    asset_group_id: &str,
    evaluator_version: &str,
) -> std::result::Result<Option<ModelEvaluation>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT evaluation_id, project_id, asset_group_id, evaluator_kind, evaluator_version,
                    run_id, status, score, tier, selectable, summary, strengths_json, weaknesses_json,
                    technical_warnings_json, prompt_pack_id, prompt_pack_version, prompt_hash,
                    created_at_ms, updated_at_ms
             FROM model_evaluations
             WHERE asset_group_id = ?1 AND evaluator_version = ?2
             ORDER BY updated_at_ms DESC, evaluation_id DESC
             LIMIT 1",
            params![asset_group_id, evaluator_version],
            model_evaluation_from_row,
        )
        .optional()
}
fn model_evaluations_for_asset_group_ids(
    connection: &Connection,
    asset_group_ids: &[String],
    evaluator_version: &str,
) -> std::result::Result<Vec<ModelEvaluation>, rusqlite::Error> {
    let mut evaluations = Vec::new();
    for asset_group_id in asset_group_ids {
        if let Some(evaluation) =
            latest_model_evaluation_for_asset_group(connection, asset_group_id, evaluator_version)?
        {
            evaluations.push(evaluation);
        }
    }
    Ok(evaluations)
}
fn model_evaluation_from_row(
    row: &Row<'_>,
) -> std::result::Result<ModelEvaluation, rusqlite::Error> {
    let evaluator_kind: String = row.get(3)?;
    let status: String = row.get(6)?;
    let tier: String = row.get(8)?;
    Ok(ModelEvaluation {
        evaluation_id: row.get(0)?,
        run_id: row.get(5)?,
        project_id: row.get(1)?,
        asset_group_id: row.get(2)?,
        evaluator_kind: ModelEvaluatorKind::from_str(&evaluator_kind),
        evaluator_version: row.get(4)?,
        status: ModelEvaluationStatus::from_str(&status),
        score: row.get(7)?,
        tier: ModelEvaluationTier::from_str(&tier),
        selectable: row.get::<_, bool>(9)?,
        summary: row.get(10)?,
        strengths: string_vec_from_json(row.get::<_, String>(11)?)?,
        weaknesses: string_vec_from_json(row.get::<_, String>(12)?)?,
        technical_warnings: string_vec_from_json(row.get::<_, String>(13)?)?,
        prompt_pack_id: row.get(14)?,
        prompt_pack_version: row.get(15)?,
        prompt_hash: row.get(16)?,
        created_at_ms: row.get(17)?,
        updated_at_ms: row.get(18)?,
    })
}
fn save_selection_recommendation_for_connection(
    connection: &Connection,
    recommendation: SelectionRecommendation,
) -> std::result::Result<SelectionRecommendation, rusqlite::Error> {
    connection.execute(
        "INSERT INTO selection_recommendations (
            recommendation_id, run_id, scope, project_id, subject_id, selected_asset_group_ids_json,
            candidate_asset_group_ids_json, rejected_asset_group_ids_json, source, status,
            confidence, reason, created_at_ms, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
         ON CONFLICT(recommendation_id) DO UPDATE SET
            run_id = excluded.run_id,
            scope = excluded.scope,
            project_id = excluded.project_id,
            subject_id = excluded.subject_id,
            selected_asset_group_ids_json = excluded.selected_asset_group_ids_json,
            candidate_asset_group_ids_json = excluded.candidate_asset_group_ids_json,
            rejected_asset_group_ids_json = excluded.rejected_asset_group_ids_json,
            source = excluded.source,
            status = excluded.status,
            confidence = excluded.confidence,
            reason = excluded.reason,
            updated_at_ms = excluded.updated_at_ms",
        params![
            recommendation.recommendation_id,
            recommendation.run_id,
            recommendation.scope.as_str(),
            recommendation.project_id,
            recommendation.subject_id,
            string_vec_json(&recommendation.selected_asset_group_ids)?,
            string_vec_json(&recommendation.candidate_asset_group_ids)?,
            string_vec_json(&recommendation.rejected_asset_group_ids)?,
            recommendation.source.as_str(),
            recommendation.status.as_str(),
            recommendation.confidence,
            recommendation.reason,
            recommendation.created_at_ms,
            recommendation.updated_at_ms,
        ],
    )?;
    selection_recommendation_by_id(connection, &recommendation.recommendation_id)?
        .ok_or_else(|| sqlite_data_error("selection recommendation not found"))
}
pub(super) fn latest_selection_recommendation_for_connection(
    connection: &Connection,
    project_id: &str,
    scope: SelectionRecommendationScope,
    subject_id: &str,
) -> std::result::Result<Option<SelectionRecommendation>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT recommendation_id, scope, project_id, subject_id, selected_asset_group_ids_json,
                    run_id, candidate_asset_group_ids_json, rejected_asset_group_ids_json, source,
                    status, confidence, reason, created_at_ms, updated_at_ms
             FROM selection_recommendations
             WHERE project_id = ?1 AND scope = ?2 AND subject_id = ?3
             ORDER BY updated_at_ms DESC, recommendation_id DESC
             LIMIT 1",
            params![project_id, scope.as_str(), subject_id],
            selection_recommendation_from_row,
        )
        .optional()
}
fn selection_recommendation_by_id(
    connection: &Connection,
    recommendation_id: &str,
) -> std::result::Result<Option<SelectionRecommendation>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT recommendation_id, scope, project_id, subject_id, selected_asset_group_ids_json,
                    run_id, candidate_asset_group_ids_json, rejected_asset_group_ids_json, source,
                    status, confidence, reason, created_at_ms, updated_at_ms
             FROM selection_recommendations
             WHERE recommendation_id = ?1",
            params![recommendation_id],
            selection_recommendation_from_row,
        )
        .optional()
}
fn selection_recommendation_from_row(
    row: &Row<'_>,
) -> std::result::Result<SelectionRecommendation, rusqlite::Error> {
    let scope: String = row.get(1)?;
    let source: String = row.get(8)?;
    let status: String = row.get(9)?;
    Ok(SelectionRecommendation {
        recommendation_id: row.get(0)?,
        run_id: row.get(5)?,
        scope: SelectionRecommendationScope::from_str(&scope),
        project_id: row.get(2)?,
        subject_id: row.get(3)?,
        selected_asset_group_ids: string_vec_from_json(row.get::<_, String>(4)?)?,
        candidate_asset_group_ids: string_vec_from_json(row.get::<_, String>(6)?)?,
        rejected_asset_group_ids: string_vec_from_json(row.get::<_, String>(7)?)?,
        source: SelectionSource::from_str(&source)
            .ok_or_else(|| sqlite_data_error(format!("unknown selection source: {source}")))?,
        status: SelectionRecommendationStatus::from_str(&status),
        confidence: row.get(10)?,
        reason: row.get(11)?,
        created_at_ms: row.get(12)?,
        updated_at_ms: row.get(13)?,
    })
}
