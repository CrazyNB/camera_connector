use camera_connector_core::{score_preview_sample, PreviewSample, QualityAnalysisStatus};

#[test]
fn sharp_preview_scores_above_flat_preview() {
    let sharp = score_preview_sample(
        "group-sharp",
        checkerboard_sample(16, 16),
        "local-v1",
        1_000,
    );
    let flat = score_preview_sample("group-flat", flat_sample(16, 16, 128), "local-v1", 1_000);

    assert_eq!(sharp.analysis_status, QualityAnalysisStatus::Ready);
    assert!(sharp.sharpness.value > flat.sharpness.value);
    assert!(sharp.overall > flat.overall);
}

#[test]
fn overexposed_preview_records_highlight_penalty() {
    let score = score_preview_sample("group-over", flat_sample(16, 16, 252), "local-v1", 1_000);

    assert!(score.highlight_clipping_penalty.value > 0.90);
    assert!(score
        .reasons
        .iter()
        .any(|reason| reason.contains("highlight")));
}

#[test]
fn underexposed_preview_records_shadow_penalty() {
    let score = score_preview_sample("group-under", flat_sample(16, 16, 3), "local-v1", 1_000);

    assert!(score.shadow_clipping_penalty.value > 0.90);
    assert!(score.reasons.iter().any(|reason| reason.contains("shadow")));
}

#[test]
fn composition_flags_edge_weighted_detail() {
    let centered = score_preview_sample(
        "group-centered",
        bright_square_sample(24, 24, 9, 9, 6),
        "local-v1",
        1_000,
    );
    let edge = score_preview_sample(
        "group-edge",
        bright_square_sample(24, 24, 0, 9, 6),
        "local-v1",
        1_000,
    );

    assert!(centered.composition.value > edge.composition.value);
    assert!(edge.reasons.iter().any(|reason| reason.contains("edge")));
}

#[test]
fn empty_preview_records_unsupported_status() {
    let score = score_preview_sample(
        "group-empty",
        PreviewSample {
            width: 0,
            height: 0,
            luma: Vec::new(),
            preview_source: Some("missing".to_string()),
        },
        "local-v1",
        1_000,
    );

    assert_eq!(score.analysis_status, QualityAnalysisStatus::Unsupported);
    assert!(score
        .reasons
        .iter()
        .any(|reason| reason.contains("unsupported")));
}

fn flat_sample(width: usize, height: usize, value: u8) -> PreviewSample {
    PreviewSample {
        width,
        height,
        luma: vec![value; width * height],
        preview_source: Some("test".to_string()),
    }
}

fn checkerboard_sample(width: usize, height: usize) -> PreviewSample {
    let mut luma = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            luma.push(if (x + y) % 2 == 0 { 0 } else { 255 });
        }
    }
    PreviewSample {
        width,
        height,
        luma,
        preview_source: Some("test".to_string()),
    }
}

fn bright_square_sample(
    width: usize,
    height: usize,
    left: usize,
    top: usize,
    size: usize,
) -> PreviewSample {
    let mut luma = vec![20; width * height];
    for y in top..top.saturating_add(size).min(height) {
        for x in left..left.saturating_add(size).min(width) {
            luma[y * width + x] = 230;
        }
    }
    PreviewSample {
        width,
        height,
        luma,
        preview_source: Some("test".to_string()),
    }
}
