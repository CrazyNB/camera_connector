use std::fs;
use std::path::{Path, PathBuf};

use crate::{current_time_ms, source_protocol_label, DashboardArgs, ReceiveFileArgs};
use camera_connector_core::{
    append_transfer_record, AccountView, AssetFacetCount, AssetGroupPage, AssetGroupQuery,
    AssetGroupSummary, CameraConnectorDashboard, CameraConnectorService, LocalFileSink, Project,
    ReceivedAsset, ReceivedAssetGroup, ReceiverRuntimeStatus, Result, SqliteStore, StoredAsset,
    StoredObjectLocation, TransferQuery, TransferRecord, TransferRecordView, TransferStatus,
};

pub(crate) fn transfer_view_line(view: &TransferRecordView) -> String {
    let record = &view.record;
    format!(
        "{}\t{:?}\t{}\t{}\t{}\tusername={}\tremote={}\tsource={}\tdisplay={}\tlocation_kind={}\tlocation={}\terror={}",
        record.transfer_id,
        record.status,
        record.final_filename,
        record.original_path,
        record.size_bytes,
        record.username.as_deref().unwrap_or("-"),
        record.remote_addr.as_deref().unwrap_or("-"),
        view.display_source.as_deref().unwrap_or("-"),
        view.virtual_display_path,
        view.final_location_kind.as_deref().unwrap_or("-"),
        view.final_location_label.as_deref().unwrap_or("-"),
        record.error.as_deref().unwrap_or("-")
    )
}

pub(crate) fn handle_receive_file_command(args: ReceiveFileArgs) -> Result<TransferRecord> {
    let filename = args
        .input
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(camera_connector_core::ImporterError::InvalidUploadPath)?;
    let bytes = fs::read(&args.input)?;
    let started_at_ms = current_time_ms();
    let protocol_label = source_protocol_label(args.source);
    let progress = LocalFileSink::new(args.output).write_complete(
        format!("{protocol_label}:{started_at_ms}:{filename}"),
        filename,
        &bytes,
    )?;
    let asset = ReceivedAsset::new(
        progress.transfer_id,
        progress.filename,
        progress.bytes_written,
        args.source,
    );
    println!(
        "received {}\t{:?}\t{} bytes",
        asset.filename, asset.format, asset.size_bytes
    );
    let final_path = progress
        .output_location
        .as_ref()
        .and_then(StoredObjectLocation::as_local_path)
        .map(Path::to_path_buf)
        .or(progress.output_path.clone())
        .ok_or_else(|| camera_connector_core::ImporterError::internal("missing output location"))?;
    let log_dir = args.state.unwrap_or_else(|| {
        final_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    });
    let record = TransferRecord {
        transfer_id: asset.id,
        protocol: protocol_label.to_string(),
        status: TransferStatus::Completed,
        original_path: filename.to_string(),
        final_filename: asset.filename,
        final_location: progress.output_location,
        size_bytes: progress.bytes_written,
        username: args.username,
        remote_addr: None,
        source_name: args.source_name,
        started_at_ms,
        completed_at_ms: Some(current_time_ms()),
        error: None,
    };
    append_transfer_record(&log_dir, &record)?;
    record_transfer_in_project(&log_dir, &args.project_id, &record)?;
    Ok(record)
}

pub(crate) fn record_transfer_in_project(
    state_dir: &Path,
    project_id: &str,
    record: &TransferRecord,
) -> Result<()> {
    let store = SqliteStore::open_state_dir(state_dir)?;
    store.record_transfer(project_id, record.clone())
}

pub(crate) fn load_dashboard(args: DashboardArgs) -> Result<CameraConnectorDashboard> {
    let service = CameraConnectorService::new(args.config);
    service.project_dashboard(
        &args.project_id,
        args.query,
        args.offset,
        args.limit,
        args.online_devices,
    )
}

pub(crate) fn load_project_asset_page(
    config: Option<PathBuf>,
    project_id: &str,
    query: AssetGroupQuery,
    offset: usize,
    limit: usize,
) -> Result<AssetGroupPage> {
    CameraConnectorService::new(config)
        .project_asset_group_page_with_query(project_id, query, offset, limit)
}

pub(crate) fn load_project_group_assets(
    config: Option<PathBuf>,
    project_id: &str,
    group_id: &str,
) -> Result<Vec<StoredAsset>> {
    CameraConnectorService::new(config).project_group_assets(project_id, group_id)
}

pub(crate) fn load_transfers(
    config: Option<PathBuf>,
    state: Option<PathBuf>,
    project_id: Option<String>,
    query: TransferQuery,
) -> Result<Vec<TransferRecordView>> {
    let service = CameraConnectorService::new(config);
    match project_id {
        Some(project_id) => service.project_transfers(&project_id, query),
        None => {
            let state = state.ok_or(camera_connector_core::ImporterError::InvalidUploadPath)?;
            service.diagnostic_transfers(state, query)
        }
    }
}

pub(crate) fn project_line(project: &Project, active_project_id: Option<&str>) -> String {
    format!(
        "project\tid={}\tname={}\tslug={}\tstatus={}\tactive={}",
        project.project_id,
        project.name,
        project.slug,
        project.status.as_str(),
        active_project_id == Some(project.project_id.as_str())
    )
}

pub(crate) fn account_view_line(account: &AccountView) -> String {
    format!(
        "account\tusername={}\tdevice={}\tpassword_configured={}\tonline={}\tconnections={}\tremote={}\tport={}\tlast_seen_ms={}\tlast_disconnected_ms={}",
        account.username,
        account.device_name,
        account.password_configured,
        account.online,
        account.active_connections,
        account.last_remote_addr.as_deref().unwrap_or("-"),
        account
            .last_remote_port
            .map(|port| port.to_string())
            .unwrap_or_else(|| "-".to_string()),
        account
            .last_seen_at_ms
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string()),
        account
            .last_disconnected_at_ms
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string())
    )
}

pub(crate) fn print_asset_groups(groups: Vec<ReceivedAssetGroup>) {
    for group in groups {
        println!("{}", asset_group_line(&group));
    }
}

pub(crate) fn print_stored_assets(assets: Vec<StoredAsset>) {
    for asset in assets {
        println!("{}", stored_asset_line(&asset));
    }
}

pub(crate) fn print_dashboard(dashboard: CameraConnectorDashboard) {
    match dashboard.receiver_status {
        Some(status) => println!("status\t{}", receiver_status_tab_fields(&status)),
        None => println!("status\tphase=Unknown\tmessage=receiver status file not found"),
    }
    println!(
        "paths\tconfig={}\tstate={}\toutput={}",
        dashboard.paths.config_path.display(),
        dashboard.paths.state_dir.display(),
        dashboard
            .paths
            .output_dir
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    for account in dashboard.accounts {
        println!("{}", account_view_line(&account));
    }
    for view in dashboard.devices {
        let device = view.device;
        println!(
            "device\t{}\tonline={}\tconnections={}\tport={}\tusername={}\tsource={}\tdisplay={}\tlast_seen_ms={}",
            device.remote_addr,
            device.online,
            device.active_connections,
            device
                .last_remote_port
                .map(|port| port.to_string())
                .unwrap_or_else(|| "-".to_string()),
            device.username.as_deref().unwrap_or("-"),
            device.source_name.as_deref().unwrap_or("-"),
            view.display_source,
            device.last_seen_at_ms
        );
    }
    println!(
        "transfers\ttotal={}\tcompleted={}\tfailed={}",
        dashboard.transfers.total_count,
        dashboard.transfers.completed_count,
        dashboard.transfers.failed_count
    );
    println!(
        "publish_queue\ttotal={}\tpending={}\tstaged={}\tpublishing={}\tcompleted={}\tfailed={}",
        dashboard.publish_queue.total_count,
        dashboard.publish_queue.pending_count,
        dashboard.publish_queue.staged_count,
        dashboard.publish_queue.publishing_count,
        dashboard.publish_queue.completed_count,
        dashboard.publish_queue.failed_count
    );
    for view in dashboard.recent_failures {
        println!("failure\t{}", transfer_view_line(&view));
    }
    println!(
        "summary\t{}",
        asset_group_page_summary_line(&dashboard.assets).trim_start_matches("summary\t")
    );
    for group in dashboard.assets.groups {
        println!("asset\t{}", asset_group_line(&group));
    }
}

pub(crate) fn print_dashboard_json(dashboard: &CameraConnectorDashboard) -> Result<()> {
    println!("{}", dashboard_json(dashboard)?);
    Ok(())
}

pub(crate) fn dashboard_json(dashboard: &CameraConnectorDashboard) -> Result<String> {
    serde_json::to_string_pretty(dashboard)
        .map_err(|error| camera_connector_core::ImporterError::internal(error.to_string()))
}

pub(crate) fn print_receiver_status_lines(status: &ReceiverRuntimeStatus) {
    println!("phase: {:?}", status.phase);
    println!(
        "protocol: {}",
        status
            .protocol
            .map(|protocol| protocol.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!("auth_mode: {:?}", status.auth_mode);
    println!(
        "local_addr: {}",
        status
            .local_addr
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "output: {}",
        status
            .output_dir
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "state: {}",
        status
            .state_dir
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!("accounts: {}", status.account_count);
    println!("message: {}", status.message.as_deref().unwrap_or("-"));
}

pub(crate) fn receiver_status_tab_fields(status: &ReceiverRuntimeStatus) -> String {
    format!(
        "phase={:?}\tprotocol={}\tauth_mode={:?}\tlocal_addr={}\toutput={}\tstate={}\taccounts={}\tmessage={}",
        status.phase,
        status
            .protocol
            .map(|protocol| protocol.to_string())
            .unwrap_or_else(|| "-".to_string()),
        status.auth_mode,
        status
            .local_addr
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "-".to_string()),
        status
            .output_dir
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "-".to_string()),
        status
            .state_dir
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "-".to_string()),
        status.account_count,
        status.message.as_deref().unwrap_or("-")
    )
}

pub(crate) fn asset_group_summary_line(summary: &AssetGroupSummary) -> String {
    format!(
        "summary\tgroups={}\tassets={}\tjpeg_groups={}\traw_groups={}\tvideo_groups={}\tsources={}\tremotes={}",
        summary.group_count,
        summary.asset_count,
        summary.groups_with_jpeg,
        summary.groups_with_raw,
        summary.groups_with_video,
        facet_counts_label(&summary.source_counts),
        facet_counts_label(&summary.remote_addr_counts)
    )
}

pub(crate) fn asset_group_page_summary_line(page: &AssetGroupPage) -> String {
    format!(
        "{}\toffset={}\tlimit={}\ttotal_groups={}\thas_more={}",
        asset_group_summary_line(&page.summary),
        page.offset,
        page.limit,
        page.total_groups,
        page.has_more
    )
}

pub(crate) fn facet_counts_label(counts: &[AssetFacetCount]) -> String {
    if counts.is_empty() {
        return "-".to_string();
    }
    counts
        .iter()
        .map(|count| format!("{}:{}", count.value, count.group_count))
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn asset_group_line(group: &ReceivedAssetGroup) -> String {
    let jpeg = group
        .jpeg
        .as_ref()
        .map(|asset| asset.filename.as_str())
        .unwrap_or("-");
    let raw = group
        .raw
        .as_ref()
        .map(|asset| asset.filename.as_str())
        .unwrap_or("-");
    let video = group
        .video
        .as_ref()
        .map(|asset| asset.filename.as_str())
        .unwrap_or("-");
    let total_bytes = group
        .jpeg
        .iter()
        .chain(group.raw.iter())
        .chain(group.video.iter())
        .map(|asset| asset.size_bytes)
        .sum::<u64>();
    let primary_location = group.primary.storage_location.as_ref();

    format!(
        "{}\tgroup_id={}\tprimary={}\tjpeg={}\traw={}\tvideo={}\t{} bytes\tusername={}\tsource={}\tremote={}\toriginal={}\tdisplay={}\tduplicate={}\tprimary_location_kind={}\tprimary_location={}",
        group.group_key,
        group.group_id.as_deref().unwrap_or("-"),
        group.primary.filename,
        jpeg,
        raw,
        video,
        total_bytes,
        group.primary.username.as_deref().unwrap_or("-"),
        group.primary.display_source.as_deref().unwrap_or("-"),
        group.primary.remote_addr.as_deref().unwrap_or("-"),
        group.primary.original_path.as_deref().unwrap_or("-"),
        group.primary.virtual_display_path.as_deref().unwrap_or("-"),
        duplicate_label(&group.primary),
        primary_location
            .map(StoredObjectLocation::kind)
            .unwrap_or("-"),
        primary_location
            .map(StoredObjectLocation::display_label)
            .unwrap_or_else(|| "-".to_string())
    )
}

pub(crate) fn stored_asset_line(asset: &StoredAsset) -> String {
    let final_location = asset.final_location.as_ref();
    format!(
        "asset\tid={}\tproject={}\tgroup_id={}\trole={}\tfilename={}\tformat={:?}\t{} bytes\tusername={}\tsource={}\tremote={}\toriginal={}\tlocation_kind={}\tlocation={}",
        asset.asset_id,
        asset.project_id,
        asset.group_id.as_deref().unwrap_or("-"),
        asset.group_role,
        asset.final_filename,
        asset.format,
        asset.size_bytes,
        asset.username.as_deref().unwrap_or("-"),
        asset.source_identity.as_deref().unwrap_or("-"),
        asset.remote_addr.as_deref().unwrap_or("-"),
        asset.original_path,
        final_location
            .map(StoredObjectLocation::kind)
            .unwrap_or("-"),
        final_location
            .map(StoredObjectLocation::display_label)
            .unwrap_or_else(|| "-".to_string())
    )
}

fn duplicate_label(asset: &ReceivedAsset) -> String {
    match (asset.duplicate_index, asset.duplicate_count) {
        (Some(index), Some(count)) => format!("{index}/{count}"),
        _ => "-".to_string(),
    }
}
