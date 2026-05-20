use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{ObjectFormat, ReceivedAsset};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceivedAssetGroup {
    pub group_key: String,
    pub primary: ReceivedAsset,
    pub jpeg: Option<ReceivedAsset>,
    pub raw: Option<ReceivedAsset>,
    pub video: Option<ReceivedAsset>,
}

pub fn group_received_assets(assets: Vec<ReceivedAsset>) -> Vec<ReceivedAssetGroup> {
    let mut grouped: BTreeMap<String, Vec<ReceivedAsset>> = BTreeMap::new();

    for asset in assets {
        let group_key = asset
            .group_key
            .clone()
            .unwrap_or_else(|| asset.id.to_ascii_uppercase());
        grouped.entry(group_key).or_default().push(asset);
    }

    let mut groups: Vec<_> = grouped
        .into_iter()
        .map(|(group_key, mut members)| {
            members.sort_by_key(|asset| format_rank(asset.format));

            let jpeg = members
                .iter()
                .find(|asset| asset.format == ObjectFormat::Jpeg)
                .cloned();
            let raw = members.iter().find(|asset| asset.format.is_raw()).cloned();
            let video = members
                .iter()
                .find(|asset| asset.format.is_video())
                .cloned();
            let primary = jpeg
                .clone()
                .or_else(|| raw.clone())
                .or_else(|| video.clone())
                .unwrap_or_else(|| members[0].clone());

            ReceivedAssetGroup {
                group_key,
                primary,
                jpeg,
                raw,
                video,
            }
        })
        .collect();

    groups.sort_by(|left, right| {
        right
            .primary
            .capture_time_ms
            .cmp(&left.primary.capture_time_ms)
            .then_with(|| left.group_key.cmp(&right.group_key))
    });

    groups
}

fn format_rank(format: ObjectFormat) -> u8 {
    match format {
        ObjectFormat::Jpeg => 0,
        ObjectFormat::Nef => 1,
        ObjectFormat::Mov | ObjectFormat::Mp4 => 2,
        ObjectFormat::Tiff => 3,
        ObjectFormat::Unknown => 4,
    }
}
