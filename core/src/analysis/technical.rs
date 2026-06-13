use serde::{Deserialize, Serialize};

const DEFAULT_ASSESSOR_VERSION: &str = "technical-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewSample {
    pub width: usize,
    pub height: usize,
    pub luma: Vec<u8>,
    #[serde(default)]
    pub red: Option<Vec<u8>>,
    #[serde(default)]
    pub green: Option<Vec<u8>>,
    #[serde(default)]
    pub blue: Option<Vec<u8>>,
    pub preview_source: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TechnicalAssessmentStatus {
    Pending,
    Analyzing,
    Ready,
    Failed,
    Unsupported,
}

impl TechnicalAssessmentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Analyzing => "analyzing",
            Self::Ready => "ready",
            Self::Failed => "failed",
            Self::Unsupported => "unsupported",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "analyzing" => Self::Analyzing,
            "ready" => Self::Ready,
            "failed" => Self::Failed,
            "unsupported" => Self::Unsupported,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TechnicalGateStatus {
    Pass,
    Warn,
    Reject,
    Inconclusive,
    Unsupported,
}

impl TechnicalGateStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Warn => "warn",
            Self::Reject => "reject",
            Self::Inconclusive => "inconclusive",
            Self::Unsupported => "unsupported",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "pass" => Self::Pass,
            "warn" => Self::Warn,
            "reject" => Self::Reject,
            "unsupported" => Self::Unsupported,
            _ => Self::Inconclusive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TechnicalDefectType {
    Blur,
    HighlightClip,
    ShadowClip,
    Noise,
    ColorCast,
    Unsupported,
}

impl TechnicalDefectType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Blur => "blur",
            Self::HighlightClip => "highlight_clip",
            Self::ShadowClip => "shadow_clip",
            Self::Noise => "noise",
            Self::ColorCast => "color_cast",
            Self::Unsupported => "unsupported",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "blur" => Self::Blur,
            "highlight_clip" => Self::HighlightClip,
            "shadow_clip" => Self::ShadowClip,
            "noise" => Self::Noise,
            "color_cast" => Self::ColorCast,
            _ => Self::Unsupported,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TechnicalDefectSeverity {
    Low,
    Medium,
    High,
    Severe,
}

impl TechnicalDefectSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Severe => "severe",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "medium" => Self::Medium,
            "high" => Self::High,
            "severe" => Self::Severe,
            _ => Self::Low,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TechnicalDefectFlag {
    pub defect_type: TechnicalDefectType,
    pub severity: TechnicalDefectSeverity,
    pub confidence: f64,
    pub metrics_json: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TechnicalAssessment {
    pub asset_group_id: String,
    pub assessor_version: String,
    pub status: TechnicalAssessmentStatus,
    pub gate_status: TechnicalGateStatus,
    pub defect_flags: Vec<TechnicalDefectFlag>,
    pub preview_source: Option<String>,
    pub visual_signature: Option<String>,
    pub analyzed_at_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TechnicalAssessmentPolicy {
    pub blur_severe_edge_threshold: f64,
    pub blur_severe_frequency_threshold: f64,
    pub blur_high_edge_threshold: f64,
    pub blur_high_frequency_threshold: f64,
    pub highlight_clip_threshold: u8,
    pub shadow_clip_threshold: u8,
    pub clipping_high_ratio: f64,
    pub clipping_high_connected_ratio: f64,
    pub clipping_severe_ratio: f64,
    pub clipping_severe_connected_ratio: f64,
    pub color_cast_high_threshold: f64,
    pub color_cast_severe_threshold: f64,
    #[serde(default = "default_face_eye_open_warn_threshold")]
    pub face_eye_open_warn_threshold: f64,
    #[serde(default = "default_face_exposure_warn_ratio")]
    pub face_exposure_warn_ratio: f64,
    #[serde(default = "default_face_color_cast_warn_threshold")]
    pub face_color_cast_warn_threshold: f64,
}

fn default_face_eye_open_warn_threshold() -> f64 {
    TechnicalAssessmentPolicy::standard().face_eye_open_warn_threshold
}

fn default_face_exposure_warn_ratio() -> f64 {
    TechnicalAssessmentPolicy::standard().face_exposure_warn_ratio
}

fn default_face_color_cast_warn_threshold() -> f64 {
    TechnicalAssessmentPolicy::standard().face_color_cast_warn_threshold
}

impl TechnicalAssessmentPolicy {
    pub fn loose() -> Self {
        Self {
            blur_severe_edge_threshold: 0.025,
            blur_severe_frequency_threshold: 0.025,
            blur_high_edge_threshold: 0.09,
            blur_high_frequency_threshold: 0.09,
            highlight_clip_threshold: 250,
            shadow_clip_threshold: 2,
            clipping_high_ratio: 0.18,
            clipping_high_connected_ratio: 0.25,
            clipping_severe_ratio: 0.65,
            clipping_severe_connected_ratio: 0.65,
            color_cast_high_threshold: 0.55,
            color_cast_severe_threshold: 0.85,
            face_eye_open_warn_threshold: 0.25,
            face_exposure_warn_ratio: 0.35,
            face_color_cast_warn_threshold: 0.55,
        }
    }

    pub fn standard() -> Self {
        Self {
            blur_severe_edge_threshold: 0.04,
            blur_severe_frequency_threshold: 0.04,
            blur_high_edge_threshold: 0.12,
            blur_high_frequency_threshold: 0.12,
            highlight_clip_threshold: 245,
            shadow_clip_threshold: 5,
            clipping_high_ratio: 0.12,
            clipping_high_connected_ratio: 0.18,
            clipping_severe_ratio: 0.50,
            clipping_severe_connected_ratio: 0.50,
            color_cast_high_threshold: 0.42,
            color_cast_severe_threshold: 0.70,
            face_eye_open_warn_threshold: 0.35,
            face_exposure_warn_ratio: 0.25,
            face_color_cast_warn_threshold: 0.42,
        }
    }

    pub fn strict() -> Self {
        Self {
            blur_severe_edge_threshold: 0.06,
            blur_severe_frequency_threshold: 0.06,
            blur_high_edge_threshold: 0.16,
            blur_high_frequency_threshold: 0.16,
            highlight_clip_threshold: 242,
            shadow_clip_threshold: 8,
            clipping_high_ratio: 0.08,
            clipping_high_connected_ratio: 0.12,
            clipping_severe_ratio: 0.40,
            clipping_severe_connected_ratio: 0.40,
            color_cast_high_threshold: 0.32,
            color_cast_severe_threshold: 0.55,
            face_eye_open_warn_threshold: 0.45,
            face_exposure_warn_ratio: 0.16,
            face_color_cast_warn_threshold: 0.32,
        }
    }
}

pub fn assess_preview_sample(
    asset_group_id: &str,
    sample: PreviewSample,
    assessor_version: &str,
    analyzed_at_ms: i64,
) -> TechnicalAssessment {
    assess_preview_sample_with_policy(
        asset_group_id,
        sample,
        assessor_version,
        analyzed_at_ms,
        TechnicalAssessmentPolicy::standard(),
    )
}

pub fn assess_preview_sample_with_policy(
    asset_group_id: &str,
    sample: PreviewSample,
    assessor_version: &str,
    analyzed_at_ms: i64,
    policy: TechnicalAssessmentPolicy,
) -> TechnicalAssessment {
    let assessor_version = if assessor_version.trim().is_empty() {
        DEFAULT_ASSESSOR_VERSION
    } else {
        assessor_version
    };
    if sample.width == 0
        || sample.height == 0
        || sample.luma.len() != sample.width.saturating_mul(sample.height)
    {
        return TechnicalAssessment {
            asset_group_id: asset_group_id.to_string(),
            assessor_version: assessor_version.to_string(),
            status: TechnicalAssessmentStatus::Unsupported,
            gate_status: TechnicalGateStatus::Unsupported,
            defect_flags: vec![TechnicalDefectFlag {
                defect_type: TechnicalDefectType::Unsupported,
                severity: TechnicalDefectSeverity::High,
                confidence: 1.0,
                metrics_json: None,
                reason: "unsupported preview sample".to_string(),
            }],
            preview_source: sample.preview_source,
            visual_signature: None,
            analyzed_at_ms,
        };
    }

    let visual_signature = average_hash_signature(&sample);
    let mut defect_flags = Vec::new();
    let edge_detail = edge_detail_score(&sample);
    let high_frequency = high_frequency_proxy(&sample);
    if edge_detail < policy.blur_severe_edge_threshold
        && high_frequency < policy.blur_severe_frequency_threshold
    {
        defect_flags.push(defect_flag(
            TechnicalDefectType::Blur,
            TechnicalDefectSeverity::Severe,
            0.92,
            Some(metrics_json(&[
                ("edge_detail", edge_detail),
                ("high_frequency", high_frequency),
            ])),
            "severe defocus or blur risk",
        ));
    } else if edge_detail < policy.blur_high_edge_threshold
        && high_frequency < policy.blur_high_frequency_threshold
    {
        defect_flags.push(defect_flag(
            TechnicalDefectType::Blur,
            TechnicalDefectSeverity::High,
            0.72,
            Some(metrics_json(&[
                ("edge_detail", edge_detail),
                ("high_frequency", high_frequency),
            ])),
            "soft detail risk",
        ));
    }

    push_clipping_defect(
        &mut defect_flags,
        &sample,
        TechnicalDefectType::HighlightClip,
        policy.highlight_clip_threshold,
        true,
        policy,
    );
    push_clipping_defect(
        &mut defect_flags,
        &sample,
        TechnicalDefectType::ShadowClip,
        policy.shadow_clip_threshold,
        false,
        policy,
    );
    push_color_cast_defect(&mut defect_flags, &sample, policy);

    let gate_status = gate_status_from_defects(&defect_flags);
    TechnicalAssessment {
        asset_group_id: asset_group_id.to_string(),
        assessor_version: assessor_version.to_string(),
        status: TechnicalAssessmentStatus::Ready,
        gate_status,
        defect_flags,
        preview_source: sample.preview_source,
        visual_signature,
        analyzed_at_ms,
    }
}

fn push_color_cast_defect(
    defect_flags: &mut Vec<TechnicalDefectFlag>,
    sample: &PreviewSample,
    policy: TechnicalAssessmentPolicy,
) {
    let Some((red, green, blue)) = rgb_channels(sample) else {
        return;
    };
    let pixel_count = red.len();
    if pixel_count == 0 {
        return;
    }

    let mean_red = red.iter().map(|value| *value as f64).sum::<f64>() / pixel_count as f64;
    let mean_green = green.iter().map(|value| *value as f64).sum::<f64>() / pixel_count as f64;
    let mean_blue = blue.iter().map(|value| *value as f64).sum::<f64>() / pixel_count as f64;
    let mean_luma = sample.luma.iter().map(|value| *value as f64).sum::<f64>() / pixel_count as f64;
    if mean_luma < 24.0 || mean_luma > 232.0 {
        return;
    }

    let normalized = [
        mean_red / mean_luma.max(1.0),
        mean_green / mean_luma.max(1.0),
        mean_blue / mean_luma.max(1.0),
    ];
    let max_channel = normalized.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let min_channel = normalized.iter().copied().fold(f64::MAX, f64::min);
    let cast_strength = max_channel - min_channel;
    let high_threshold = policy.color_cast_high_threshold.max(0.0);
    let severe_threshold = policy
        .color_cast_severe_threshold
        .max(high_threshold)
        .max(0.0);
    let (severity, confidence) = if cast_strength >= severe_threshold {
        (TechnicalDefectSeverity::Severe, 0.88)
    } else if cast_strength >= high_threshold {
        (TechnicalDefectSeverity::High, 0.72)
    } else {
        return;
    };

    defect_flags.push(defect_flag(
        TechnicalDefectType::ColorCast,
        severity,
        confidence,
        Some(metrics_json(&[
            ("mean_red", mean_red),
            ("mean_green", mean_green),
            ("mean_blue", mean_blue),
            ("cast_strength", cast_strength),
        ])),
        "strong color cast risk",
    ));
}

fn rgb_channels(sample: &PreviewSample) -> Option<(&[u8], &[u8], &[u8])> {
    let expected_len = sample.width.saturating_mul(sample.height);
    let red = sample.red.as_deref()?;
    let green = sample.green.as_deref()?;
    let blue = sample.blue.as_deref()?;
    if red.len() == expected_len && green.len() == expected_len && blue.len() == expected_len {
        Some((red, green, blue))
    } else {
        None
    }
}

fn average_hash_signature(sample: &PreviewSample) -> Option<String> {
    if sample.width == 0
        || sample.height == 0
        || sample.luma.len() != sample.width.saturating_mul(sample.height)
    {
        return None;
    }
    let mut cells = [0_u32; 64];
    let mut counts = [0_u32; 64];
    for y in 0..sample.height {
        let cell_y = y * 8 / sample.height;
        for x in 0..sample.width {
            let cell_x = x * 8 / sample.width;
            let index = cell_y * 8 + cell_x;
            cells[index] += sample.luma[y * sample.width + x] as u32;
            counts[index] += 1;
        }
    }
    let averages = cells
        .into_iter()
        .zip(counts)
        .map(|(sum, count)| {
            if count == 0 {
                0.0
            } else {
                sum as f64 / count as f64
            }
        })
        .collect::<Vec<_>>();
    let mean = averages.iter().sum::<f64>() / averages.len() as f64;
    let mut hash = 0_u64;
    for (index, value) in averages.into_iter().enumerate() {
        if value > mean {
            hash |= 1_u64 << index;
        }
    }
    Some(format!("ahash-v1:{hash:016x}"))
}

fn edge_detail_score(sample: &PreviewSample) -> f64 {
    if sample.width < 2 || sample.height < 2 {
        return 0.0;
    }
    let mut total = 0.0;
    let mut count = 0.0;
    for y in 0..sample.height {
        for x in 0..sample.width {
            let current = sample.luma[y * sample.width + x] as f64;
            if x + 1 < sample.width {
                total += (current - sample.luma[y * sample.width + x + 1] as f64).abs();
                count += 1.0;
            }
            if y + 1 < sample.height {
                total += (current - sample.luma[(y + 1) * sample.width + x] as f64).abs();
                count += 1.0;
            }
        }
    }
    ((total / count) / 96.0).clamp(0.0, 1.0)
}

fn high_frequency_proxy(sample: &PreviewSample) -> f64 {
    if sample.width < 3 || sample.height < 3 {
        return edge_detail_score(sample);
    }
    let mut total = 0.0;
    let mut count = 0.0;
    for y in 1..sample.height - 1 {
        for x in 1..sample.width - 1 {
            let center = sample.luma[y * sample.width + x] as f64 * 4.0;
            let neighbors = sample.luma[y * sample.width + x - 1] as f64
                + sample.luma[y * sample.width + x + 1] as f64
                + sample.luma[(y - 1) * sample.width + x] as f64
                + sample.luma[(y + 1) * sample.width + x] as f64;
            total += (center - neighbors).abs();
            count += 1.0;
        }
    }
    ((total / count) / 192.0).clamp(0.0, 1.0)
}

fn push_clipping_defect(
    defect_flags: &mut Vec<TechnicalDefectFlag>,
    sample: &PreviewSample,
    defect_type: TechnicalDefectType,
    threshold: u8,
    high: bool,
    policy: TechnicalAssessmentPolicy,
) {
    let clipped_count = sample
        .luma
        .iter()
        .filter(|value| {
            if high {
                **value >= threshold
            } else {
                **value <= threshold
            }
        })
        .count();
    let clipped_ratio = clipped_count as f64 / sample.luma.len() as f64;
    let connected_ratio = coarse_connected_clipping_ratio(sample, threshold, high);
    let (severity, confidence) = if clipped_ratio >= policy.clipping_severe_ratio
        || connected_ratio >= policy.clipping_severe_connected_ratio
    {
        (TechnicalDefectSeverity::Severe, 0.92)
    } else if clipped_ratio >= policy.clipping_high_ratio
        || connected_ratio >= policy.clipping_high_connected_ratio
    {
        (TechnicalDefectSeverity::High, 0.76)
    } else {
        return;
    };
    defect_flags.push(defect_flag(
        defect_type,
        severity,
        confidence,
        Some(metrics_json(&[
            ("clipped_ratio", clipped_ratio),
            ("connected_ratio", connected_ratio),
        ])),
        match defect_type {
            TechnicalDefectType::HighlightClip => "large highlight clipping risk",
            TechnicalDefectType::ShadowClip => "large shadow clipping risk",
            _ => "technical clipping risk",
        },
    ));
}

fn coarse_connected_clipping_ratio(sample: &PreviewSample, threshold: u8, high: bool) -> f64 {
    let cell_columns = 8.min(sample.width.max(1));
    let cell_rows = 8.min(sample.height.max(1));
    let mut clipped_cells = 0;
    for cell_y in 0..cell_rows {
        for cell_x in 0..cell_columns {
            let x_start = cell_x * sample.width / cell_columns;
            let x_end = ((cell_x + 1) * sample.width / cell_columns).max(x_start + 1);
            let y_start = cell_y * sample.height / cell_rows;
            let y_end = ((cell_y + 1) * sample.height / cell_rows).max(y_start + 1);
            let mut clipped = 0;
            let mut count = 0;
            for y in y_start..y_end.min(sample.height) {
                for x in x_start..x_end.min(sample.width) {
                    let value = sample.luma[y * sample.width + x];
                    if (high && value >= threshold) || (!high && value <= threshold) {
                        clipped += 1;
                    }
                    count += 1;
                }
            }
            if count > 0 && clipped as f64 / count as f64 >= 0.80 {
                clipped_cells += 1;
            }
        }
    }
    clipped_cells as f64 / (cell_columns * cell_rows) as f64
}

fn gate_status_from_defects(defect_flags: &[TechnicalDefectFlag]) -> TechnicalGateStatus {
    if defect_flags
        .iter()
        .any(|flag| flag.severity == TechnicalDefectSeverity::Severe)
    {
        TechnicalGateStatus::Reject
    } else if defect_flags
        .iter()
        .any(|flag| flag.severity == TechnicalDefectSeverity::High)
    {
        TechnicalGateStatus::Warn
    } else {
        TechnicalGateStatus::Pass
    }
}

fn defect_flag(
    defect_type: TechnicalDefectType,
    severity: TechnicalDefectSeverity,
    confidence: f64,
    metrics_json: Option<String>,
    reason: &str,
) -> TechnicalDefectFlag {
    TechnicalDefectFlag {
        defect_type,
        severity,
        confidence,
        metrics_json,
        reason: reason.to_string(),
    }
}

fn metrics_json(values: &[(&str, f64)]) -> String {
    let mut object = serde_json::Map::new();
    for (key, value) in values {
        object.insert((*key).to_string(), serde_json::json!(value));
    }
    serde_json::Value::Object(object).to_string()
}
