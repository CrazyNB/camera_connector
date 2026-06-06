use camera_connector_core::{
    assess_preview_sample, PreviewSample, TechnicalAssessmentStatus, TechnicalDefectType,
    TechnicalGateStatus,
};

#[test]
fn flat_preview_is_rejected_as_blur_risk() {
    let assessment = assess_preview_sample(
        "group-soft",
        flat_sample(24, 24, 128),
        "technical-v1",
        2_000,
    );

    assert_eq!(assessment.status, TechnicalAssessmentStatus::Ready);
    assert_eq!(assessment.gate_status, TechnicalGateStatus::Reject);
    assert!(assessment
        .defect_flags
        .iter()
        .any(|flag| flag.defect_type == TechnicalDefectType::Blur));
}

#[test]
fn highlight_clipping_is_rejected() {
    let assessment = assess_preview_sample(
        "group-highlight",
        flat_sample(24, 24, 252),
        "technical-v1",
        2_000,
    );

    assert_eq!(assessment.gate_status, TechnicalGateStatus::Reject);
    assert!(assessment
        .defect_flags
        .iter()
        .any(|flag| flag.defect_type == TechnicalDefectType::HighlightClip));
}

#[test]
fn shadow_clipping_is_rejected() {
    let assessment = assess_preview_sample(
        "group-shadow",
        flat_sample(24, 24, 3),
        "technical-v1",
        2_000,
    );

    assert_eq!(assessment.gate_status, TechnicalGateStatus::Reject);
    assert!(assessment
        .defect_flags
        .iter()
        .any(|flag| flag.defect_type == TechnicalDefectType::ShadowClip));
}

#[test]
fn unsupported_preview_records_unsupported_gate() {
    let assessment = assess_preview_sample(
        "group-empty",
        PreviewSample {
            width: 0,
            height: 0,
            luma: Vec::new(),
            preview_source: Some("missing".to_string()),
        },
        "technical-v1",
        2_000,
    );

    assert_eq!(assessment.status, TechnicalAssessmentStatus::Unsupported);
    assert_eq!(assessment.gate_status, TechnicalGateStatus::Unsupported);
    assert!(assessment.visual_signature.is_none());
    assert!(assessment
        .defect_flags
        .iter()
        .any(|flag| flag.defect_type == TechnicalDefectType::Unsupported));
}

#[test]
fn valid_preview_records_visual_signature() {
    let assessment = assess_preview_sample(
        "group-signature",
        bright_square_sample(24, 24, 9, 9, 6),
        "technical-v1",
        2_000,
    );

    assert!(assessment
        .visual_signature
        .as_deref()
        .is_some_and(|value| value.starts_with("ahash-v1:")));
}

fn flat_sample(width: usize, height: usize, value: u8) -> PreviewSample {
    PreviewSample {
        width,
        height,
        luma: vec![value; width * height],
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
