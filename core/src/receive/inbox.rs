use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use super::{devices::CONNECTED_DEVICES_FILENAME, transfer_log::TRANSFER_LOG_FILENAME};
use crate::runtime::RECEIVER_STATUS_FILENAME;
use crate::{group_received_assets, ImportSource, ReceivedAsset, ReceivedAssetGroup, Result};

pub fn scan_inbox(root: impl AsRef<Path>, source: ImportSource) -> Result<Vec<ReceivedAsset>> {
    let root = root.as_ref();
    let mut assets = Vec::new();
    collect_assets(root, root, source, &mut assets)?;
    assets.sort_by(|left, right| {
        right
            .received_time_ms
            .cmp(&left.received_time_ms)
            .then_with(|| left.filename.cmp(&right.filename))
    });
    Ok(assets)
}

pub fn scan_inbox_groups(
    root: impl AsRef<Path>,
    source: ImportSource,
) -> Result<Vec<ReceivedAssetGroup>> {
    scan_inbox(root, source).map(group_received_assets)
}

fn collect_assets(
    root: &Path,
    current: &Path,
    source: ImportSource,
    assets: &mut Vec<ReceivedAsset>,
) -> Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            collect_assets(root, &path, source, assets)?;
            continue;
        }

        if !metadata.is_file() || is_temporary_upload(&path) || is_receiver_metadata(&path) {
            continue;
        }

        let relative_path = relative_display_path(root, &path);
        let mut asset =
            ReceivedAsset::new(relative_path.clone(), relative_path, metadata.len(), source);
        asset.received_time_ms = metadata
            .modified()
            .ok()
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_millis() as i64);
        assets.push(asset);
    }

    Ok(())
}

fn is_temporary_upload(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_ascii_lowercase().ends_with(".tmp"))
        .unwrap_or(false)
}

fn is_receiver_metadata(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| {
            name.eq_ignore_ascii_case(TRANSFER_LOG_FILENAME)
                || name.eq_ignore_ascii_case(CONNECTED_DEVICES_FILENAME)
                || name.eq_ignore_ascii_case(RECEIVER_STATUS_FILENAME)
                || name.eq_ignore_ascii_case("sftp-host-key")
        })
        .unwrap_or(false)
}

fn relative_display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .filter_map(component_to_string)
        .collect::<Vec<_>>()
        .join("/")
}

fn component_to_string(component: std::path::Component<'_>) -> Option<String> {
    match component {
        std::path::Component::Normal(value) => value.to_str().map(ToOwned::to_owned),
        _ => None,
    }
}
