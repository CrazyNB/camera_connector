use std::collections::BTreeMap;
use std::path::Path;

use crate::{
    BurstGroupingProfile, ConnectedDevice, EvaluationRunType, LanShareSession,
    ProjectEvaluationSettings, ProjectStatus, ReceiverAccountConfig, Result, SceneProfile,
    SqliteStore, TransferRecord, TransferStatus,
};

use super::{AccountView, AssetGroupQuery, ConnectedDeviceView, TransferQuery, TransferSummary};

pub(super) fn receiver_account_configs_from_state_dir(
    state_dir: impl AsRef<Path>,
) -> Result<BTreeMap<String, ReceiverAccountConfig>> {
    SqliteStore::open_state_dir(state_dir)?
        .receiver_accounts()?
        .into_iter()
        .map(|account| {
            let account = account.into_account_config()?;
            Ok((account.username.clone(), account))
        })
        .collect()
}

pub(super) fn transfer_query_from_asset_query(query: &AssetGroupQuery) -> TransferQuery {
    TransferQuery {
        status: None,
        transfer_id: None,
        original_path: query.original_path.clone(),
        final_filename: None,
        username: query.username.clone(),
        source_name: query.source_name.clone(),
        remote_addr: query.remote_addr.clone(),
    }
}

pub(super) fn summarize_transfers(records: &[TransferRecord]) -> TransferSummary {
    TransferSummary {
        total_count: records.len(),
        completed_count: records
            .iter()
            .filter(|record| record.status == TransferStatus::Completed)
            .count(),
        failed_count: records
            .iter()
            .filter(|record| record.status == TransferStatus::Failed)
            .count(),
    }
}

pub(super) fn account_view(account: ReceiverAccountConfig) -> AccountView {
    let password_configured = account.password_configured();
    AccountView {
        username: account.username,
        device_name: account.device_name,
        password_configured,
        online: false,
        active_connections: 0,
        last_remote_addr: None,
        last_remote_port: None,
        last_seen_at_ms: None,
        last_disconnected_at_ms: None,
    }
}

pub(super) fn accounts_with_devices(
    accounts: Vec<AccountView>,
    devices: &[ConnectedDeviceView],
) -> Vec<AccountView> {
    accounts
        .into_iter()
        .map(|mut account| {
            account.online = false;
            account.active_connections = 0;
            account.last_remote_addr = None;
            account.last_remote_port = None;
            account.last_seen_at_ms = None;
            account.last_disconnected_at_ms = None;
            let matching_devices = devices
                .iter()
                .filter(|view| view.device.username.as_deref() == Some(account.username.as_str()));
            let mut active_connections = 0u32;
            let mut latest: Option<&ConnectedDevice> = None;
            for device in matching_devices.map(|view| &view.device) {
                active_connections = active_connections.saturating_add(device.active_connections);
                latest = Some(match latest {
                    Some(current)
                        if current.online && !device.online
                            || current.online == device.online
                                && current.last_seen_at_ms >= device.last_seen_at_ms =>
                    {
                        current
                    }
                    _ => device,
                });
            }
            if let Some(device) = latest {
                account.online = device.online;
                account.active_connections = active_connections;
                account.last_remote_addr = Some(device.remote_addr.clone());
                account.last_remote_port = device.last_remote_port;
                account.last_seen_at_ms = Some(device.last_seen_at_ms);
                account.last_disconnected_at_ms = device.last_disconnected_at_ms;
            }
            account
        })
        .collect()
}

pub(super) fn default_burst_grouping_profile(_store: &SqliteStore) -> Result<BurstGroupingProfile> {
    Ok(BurstGroupingProfile::default())
}

pub(super) fn recommend_job_dedupe_key(burst_group_id: &str) -> String {
    format!("recommend:{burst_group_id}")
}

pub(super) fn evaluation_run_id(
    project_id: &str,
    run_type: EvaluationRunType,
    subject_id: &str,
    now_ms: i64,
) -> String {
    let key = format!("{}:{project_id}:{subject_id}:{now_ms}", run_type.as_str());
    let mut hash = 1469598103934665603_u64;
    for byte in key.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("evaluation-run-{hash:016x}")
}

pub(super) fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

pub(super) fn mapped_project_sync_group_ids(
    snapshot_group_ids: &[String],
    matched_groups: &BTreeMap<String, String>,
) -> Option<Vec<String>> {
    let mut local_group_ids = Vec::with_capacity(snapshot_group_ids.len());
    for snapshot_group_id in snapshot_group_ids {
        local_group_ids.push(matched_groups.get(snapshot_group_id)?.clone());
    }
    Some(local_group_ids)
}

pub(super) fn project_sync_model_evaluation(
    project_id: &str,
    local_group_id: &str,
    evaluation: &crate::ProjectSyncSnapshotModelEvaluation,
) -> crate::ModelEvaluation {
    crate::ModelEvaluation {
        evaluation_id: format!(
            "project-sync-eval-{}",
            stable_project_sync_key(&format!(
                "{project_id}\t{}\t{local_group_id}",
                evaluation.evaluation_id
            ))
        ),
        run_id: format!(
            "project-sync-run-{}",
            stable_project_sync_key(&format!("{project_id}\t{}", evaluation.evaluation_id))
        ),
        project_id: project_id.to_string(),
        asset_group_id: local_group_id.to_string(),
        evaluator_kind: crate::ModelEvaluatorKind::Imported,
        evaluator_version: evaluation.evaluator_version.clone(),
        status: crate::ModelEvaluationStatus::from_str(&evaluation.status),
        score: evaluation.score,
        tier: crate::ModelEvaluationTier::from_str(&evaluation.tier),
        selectable: evaluation.selectable,
        summary: evaluation.summary.clone(),
        strengths: evaluation.strengths.clone(),
        weaknesses: evaluation.weaknesses.clone(),
        technical_warnings: evaluation.technical_warnings.clone(),
        prompt_pack_id: evaluation.prompt_pack_id.clone(),
        prompt_pack_version: evaluation.prompt_pack_version.clone(),
        prompt_hash: evaluation.prompt_hash.clone(),
        created_at_ms: evaluation.created_at_ms,
        updated_at_ms: evaluation.updated_at_ms,
    }
}

pub(super) fn stable_project_sync_key(value: &str) -> String {
    let mut hash = 1469598103934665603_u64;
    for byte in value.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{hash:016x}")
}

pub(super) fn ensure_service_project_is_active(
    store: &SqliteStore,
    project_id: &str,
) -> Result<()> {
    let project = store
        .list_projects()?
        .into_iter()
        .find(|project| project.project_id == project_id)
        .ok_or_else(|| crate::ImporterError::internal("project not found"))?;
    if project.status != ProjectStatus::Active {
        return Err(crate::ImporterError::internal("project archived"));
    }
    Ok(())
}

pub(super) fn active_lan_share_session(
    store: &SqliteStore,
    token: &str,
) -> Result<LanShareSession> {
    let session = store
        .lan_share_session_by_token(token)?
        .ok_or_else(|| crate::ImporterError::internal("lan share session not found"))?;
    if !session.active {
        return Err(crate::ImporterError::internal("lan share session stopped"));
    }
    Ok(session)
}

pub(super) fn should_schedule_subject_assessment_for_settings(
    settings: &ProjectEvaluationSettings,
) -> bool {
    settings.scene_profile == SceneProfile::Portrait
}

pub(super) fn stable_prompt_hash(output_schema_version: &str, prompt_text: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in output_schema_version
        .as_bytes()
        .iter()
        .copied()
        .chain([0])
        .chain(prompt_text.as_bytes().iter().copied())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64-{hash:016x}")
}
