use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{ObjectFormat, ReceivedAsset};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReceivedAssetGroup {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    pub group_key: String,
    pub primary: ReceivedAsset,
    pub jpeg: Option<ReceivedAsset>,
    pub raw: Option<ReceivedAsset>,
    pub video: Option<ReceivedAsset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub burst: Option<ReceivedAssetBurstSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<ReceivedAssetQualitySummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub technical_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub technical_gate_status: Option<String>,
    #[serde(default)]
    pub technical_defects: Vec<ReceivedAssetTechnicalDefectSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_score: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_tier: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_evaluator_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_summary: Option<String>,
    #[serde(default)]
    pub is_model_select: bool,
    #[serde(default)]
    pub is_favorite: bool,
    #[serde(default)]
    pub is_flagged: bool,
    #[serde(default)]
    pub user_marks: AssetUserMarks,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetUserMarks {
    pub favorite: bool,
    pub marked: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReceivedAssetBurstSummary {
    pub burst_group_id: String,
    pub member_count: usize,
    pub recommendation_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best_asset_group_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best_score: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReceivedAssetQualitySummary {
    pub overall: f64,
    pub analysis_status: String,
    pub scorer_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sharpness: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exposure: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlight_clipping_penalty: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadow_clipping_penalty: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composition: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composition_confidence: Option<f64>,
    pub analyzed_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReceivedAssetTechnicalDefectSummary {
    pub defect_type: String,
    pub severity: String,
    pub confidence: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
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
                group_id: None,
                group_key,
                primary,
                jpeg,
                raw,
                video,
                burst: None,
                quality: None,
                technical_status: None,
                technical_gate_status: None,
                technical_defects: Vec::new(),
                model_status: None,
                model_score: None,
                model_tier: None,
                model_evaluator_kind: None,
                model_summary: None,
                is_model_select: false,
                is_favorite: false,
                is_flagged: false,
                user_marks: AssetUserMarks::default(),
            }
        })
        .collect();

    groups.sort_by(|left, right| {
        group_sort_time(right)
            .cmp(&group_sort_time(left))
            .then_with(|| left.group_key.cmp(&right.group_key))
    });

    groups
}

fn group_sort_time(group: &ReceivedAssetGroup) -> Option<i64> {
    group_members(group)
        .into_iter()
        .filter_map(|asset| asset.capture_time_ms.or(asset.received_time_ms))
        .max()
}

fn group_members(group: &ReceivedAssetGroup) -> Vec<&ReceivedAsset> {
    let mut members = Vec::new();
    push_unique_member(&mut members, &group.primary);
    if let Some(asset) = group.jpeg.as_ref() {
        push_unique_member(&mut members, asset);
    }
    if let Some(asset) = group.raw.as_ref() {
        push_unique_member(&mut members, asset);
    }
    if let Some(asset) = group.video.as_ref() {
        push_unique_member(&mut members, asset);
    }
    members
}

fn push_unique_member<'a>(members: &mut Vec<&'a ReceivedAsset>, asset: &'a ReceivedAsset) {
    if !members.iter().any(|member| member.id == asset.id) {
        members.push(asset);
    }
}

fn format_rank(format: ObjectFormat) -> u8 {
    match format {
        ObjectFormat::Jpeg => 0,
        ObjectFormat::Nef
        | ObjectFormat::Nrw
        | ObjectFormat::Cr2
        | ObjectFormat::Cr3
        | ObjectFormat::Arw
        | ObjectFormat::Raf
        | ObjectFormat::Rw2
        | ObjectFormat::Orf
        | ObjectFormat::Pef
        | ObjectFormat::Dng => 1,
        ObjectFormat::Mov | ObjectFormat::Mp4 => 2,
        ObjectFormat::Tiff => 3,
        ObjectFormat::Unknown => 4,
    }
}
