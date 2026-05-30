use serde::{Deserialize, Serialize};

const DEFAULT_SCORER_VERSION: &str = "local-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityAnalysisStatus {
    Pending,
    Analyzing,
    Ready,
    Stale,
    Failed,
    Unsupported,
}

impl QualityAnalysisStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Analyzing => "analyzing",
            Self::Ready => "ready",
            Self::Stale => "stale",
            Self::Failed => "failed",
            Self::Unsupported => "unsupported",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "analyzing" => Self::Analyzing,
            "ready" => Self::Ready,
            "stale" => Self::Stale,
            "failed" => Self::Failed,
            "unsupported" => Self::Unsupported,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SignalScore {
    pub value: f64,
    pub available: bool,
}

impl SignalScore {
    pub fn ready(value: f64) -> Self {
        Self {
            value,
            available: true,
        }
    }

    pub fn unavailable() -> Self {
        Self {
            value: 0.0,
            available: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityScore {
    pub asset_group_id: String,
    pub preview_source: Option<String>,
    pub scorer_version: String,
    pub analysis_status: QualityAnalysisStatus,
    pub exif_status: Option<String>,
    pub capture_time_ms: Option<i64>,
    pub sharpness: SignalScore,
    pub exposure: SignalScore,
    pub highlight_clipping_penalty: SignalScore,
    pub shadow_clipping_penalty: SignalScore,
    pub composition: SignalScore,
    pub composition_confidence: f64,
    pub similarity_cluster_id: Option<String>,
    pub overall: f64,
    pub reasons: Vec<String>,
    pub analyzed_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewSample {
    pub width: usize,
    pub height: usize,
    pub luma: Vec<u8>,
    pub preview_source: Option<String>,
}

pub fn score_preview_sample(
    asset_group_id: &str,
    sample: PreviewSample,
    scorer_version: &str,
    analyzed_at_ms: i64,
) -> QualityScore {
    let scorer_version = if scorer_version.trim().is_empty() {
        DEFAULT_SCORER_VERSION
    } else {
        scorer_version
    };
    if sample.width == 0
        || sample.height == 0
        || sample.luma.len() != sample.width.saturating_mul(sample.height)
    {
        return unsupported_score(
            asset_group_id,
            sample.preview_source,
            scorer_version,
            analyzed_at_ms,
        );
    }

    let sharpness = sharpness_score(&sample);
    let exposure = exposure_score(&sample);
    let highlight_clipping = clipping_fraction(&sample, 245, true);
    let shadow_clipping = clipping_fraction(&sample, 10, false);
    let (composition, composition_confidence, mut reasons) = composition_score(&sample);

    if sharpness < 0.25 {
        reasons.push("low sharpness".to_string());
    }
    if highlight_clipping > 0.10 {
        reasons.push("highlight clipping".to_string());
    }
    if shadow_clipping > 0.10 {
        reasons.push("shadow clipping".to_string());
    }
    if exposure < 0.35 {
        reasons.push("weak exposure".to_string());
    }
    if reasons.is_empty() {
        reasons.push("balanced technical score".to_string());
    }

    let overall = clamp01(
        sharpness * 0.42
            + exposure * 0.26
            + composition * 0.14
            + (1.0 - highlight_clipping) * 0.10
            + (1.0 - shadow_clipping) * 0.08,
    );

    QualityScore {
        asset_group_id: asset_group_id.to_string(),
        preview_source: sample.preview_source,
        scorer_version: scorer_version.to_string(),
        analysis_status: QualityAnalysisStatus::Ready,
        exif_status: None,
        capture_time_ms: None,
        sharpness: SignalScore::ready(sharpness),
        exposure: SignalScore::ready(exposure),
        highlight_clipping_penalty: SignalScore::ready(highlight_clipping),
        shadow_clipping_penalty: SignalScore::ready(shadow_clipping),
        composition: SignalScore::ready(composition),
        composition_confidence,
        similarity_cluster_id: None,
        overall,
        reasons,
        analyzed_at_ms,
    }
}

fn unsupported_score(
    asset_group_id: &str,
    preview_source: Option<String>,
    scorer_version: &str,
    analyzed_at_ms: i64,
) -> QualityScore {
    QualityScore {
        asset_group_id: asset_group_id.to_string(),
        preview_source,
        scorer_version: scorer_version.to_string(),
        analysis_status: QualityAnalysisStatus::Unsupported,
        exif_status: None,
        capture_time_ms: None,
        sharpness: SignalScore::unavailable(),
        exposure: SignalScore::unavailable(),
        highlight_clipping_penalty: SignalScore::unavailable(),
        shadow_clipping_penalty: SignalScore::unavailable(),
        composition: SignalScore::unavailable(),
        composition_confidence: 0.0,
        similarity_cluster_id: None,
        overall: 0.0,
        reasons: vec!["unsupported preview sample".to_string()],
        analyzed_at_ms,
    }
}

fn sharpness_score(sample: &PreviewSample) -> f64 {
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
    clamp01((total / count) / 96.0)
}

fn exposure_score(sample: &PreviewSample) -> f64 {
    let mean =
        sample.luma.iter().map(|value| *value as f64).sum::<f64>() / sample.luma.len() as f64;
    clamp01(1.0 - ((mean / 255.0) - 0.50).abs() * 2.0)
}

fn clipping_fraction(sample: &PreviewSample, threshold: u8, high: bool) -> f64 {
    let count = sample
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
    count as f64 / sample.luma.len() as f64
}

fn composition_score(sample: &PreviewSample) -> (f64, f64, Vec<String>) {
    let mean =
        sample.luma.iter().map(|value| *value as f64).sum::<f64>() / sample.luma.len() as f64;
    let mut weight_sum = 0.0;
    let mut weighted_x = 0.0;
    let mut weighted_y = 0.0;
    let mut edge_weight = 0.0;
    let margin_x = (sample.width as f64 * 0.15).max(1.0);
    let margin_y = (sample.height as f64 * 0.15).max(1.0);

    for y in 0..sample.height {
        for x in 0..sample.width {
            let value = sample.luma[y * sample.width + x] as f64;
            let weight = (value - mean).abs();
            if weight <= 2.0 {
                continue;
            }
            weight_sum += weight;
            weighted_x += x as f64 * weight;
            weighted_y += y as f64 * weight;
            if x as f64 <= margin_x
                || y as f64 <= margin_y
                || x as f64 >= sample.width as f64 - margin_x
                || y as f64 >= sample.height as f64 - margin_y
            {
                edge_weight += weight;
            }
        }
    }

    if weight_sum <= 0.0 {
        return (0.45, 0.25, vec!["low information area".to_string()]);
    }

    let centroid_x = weighted_x / weight_sum / (sample.width.saturating_sub(1).max(1) as f64);
    let centroid_y = weighted_y / weight_sum / (sample.height.saturating_sub(1).max(1) as f64);
    let center_distance = ((centroid_x - 0.5).abs() + (centroid_y - 0.5).abs()) / 1.0;
    let edge_fraction = edge_weight / weight_sum;
    let mut score = clamp01(1.0 - center_distance * 0.85 - edge_fraction * 0.45);
    let mut reasons = Vec::new();
    if edge_fraction > 0.35
        || centroid_x < 0.20
        || centroid_x > 0.80
        || centroid_y < 0.20
        || centroid_y > 0.80
    {
        reasons.push("edge weighted detail".to_string());
        score = score.min(0.55);
    }
    (
        score,
        clamp01(weight_sum / (sample.luma.len() as f64 * 64.0)),
        reasons,
    )
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}
