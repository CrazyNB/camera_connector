use camera_connector_core::{
    assess_preview_sample, assess_preview_sample_with_policy, PreviewSample,
    TechnicalAssessmentPolicy, TechnicalAssessmentStatus, TechnicalDefectType, TechnicalGateStatus,
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
            red: None,
            green: None,
            blue: None,
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

#[test]
fn strict_policy_flags_mild_clipping_that_standard_allows() {
    let standard = assess_preview_sample_with_policy(
        "group-standard-policy",
        scattered_highlight_sample(100, 100, 900),
        "technical-v1",
        2_000,
        TechnicalAssessmentPolicy::standard(),
    );
    let strict = assess_preview_sample_with_policy(
        "group-strict-policy",
        scattered_highlight_sample(100, 100, 900),
        "technical-v1",
        2_000,
        TechnicalAssessmentPolicy::strict(),
    );

    assert_eq!(standard.gate_status, TechnicalGateStatus::Pass);
    assert_eq!(strict.gate_status, TechnicalGateStatus::Warn);
    assert!(strict
        .defect_flags
        .iter()
        .any(|flag| flag.defect_type == TechnicalDefectType::HighlightClip));
}

#[test]
fn rgb_preview_flags_strong_color_cast() {
    let assessment = assess_preview_sample(
        "group-color-cast",
        color_sample(24, 24, 210, 96, 78),
        "technical-v1",
        2_000,
    );

    assert_eq!(assessment.gate_status, TechnicalGateStatus::Reject);
    assert!(assessment
        .defect_flags
        .iter()
        .any(|flag| flag.defect_type == TechnicalDefectType::ColorCast));
}

#[test]
fn grayscale_rgb_preview_does_not_flag_color_cast() {
    let assessment = assess_preview_sample(
        "group-neutral-color",
        color_sample(24, 24, 128, 128, 128),
        "technical-v1",
        2_000,
    );

    assert!(!assessment
        .defect_flags
        .iter()
        .any(|flag| flag.defect_type == TechnicalDefectType::ColorCast));
}

#[test]
fn custom_policy_controls_color_cast_thresholds() {
    let mut strict_color = TechnicalAssessmentPolicy::standard();
    strict_color.color_cast_high_threshold = 0.20;
    strict_color.color_cast_severe_threshold = 0.45;
    let mut loose_color = TechnicalAssessmentPolicy::standard();
    loose_color.color_cast_high_threshold = 0.80;
    loose_color.color_cast_severe_threshold = 0.95;

    let strict = assess_preview_sample_with_policy(
        "group-strict-color",
        color_sample(24, 24, 160, 124, 116),
        "technical-v1",
        2_000,
        strict_color,
    );
    let loose = assess_preview_sample_with_policy(
        "group-loose-color",
        color_sample(24, 24, 160, 124, 116),
        "technical-v1",
        2_000,
        loose_color,
    );

    assert!(strict
        .defect_flags
        .iter()
        .any(|flag| flag.defect_type == TechnicalDefectType::ColorCast));
    assert!(!loose
        .defect_flags
        .iter()
        .any(|flag| flag.defect_type == TechnicalDefectType::ColorCast));
}

#[test]
fn technical_assessment_policy_defaults_new_face_fields_when_missing() {
    let policy: TechnicalAssessmentPolicy = serde_json::from_str(
        r#"{
            "blur_severe_edge_threshold":0.04,
            "blur_severe_frequency_threshold":0.04,
            "blur_high_edge_threshold":0.12,
            "blur_high_frequency_threshold":0.12,
            "highlight_clip_threshold":245,
            "shadow_clip_threshold":10,
            "clipping_high_ratio":0.12,
            "clipping_high_connected_ratio":0.18,
            "clipping_severe_ratio":0.50,
            "clipping_severe_connected_ratio":0.50,
            "color_cast_high_threshold":0.42,
            "color_cast_severe_threshold":0.70
        }"#,
    )
    .expect("old policy JSON should remain readable");

    assert_eq!(policy.face_eye_open_warn_threshold, 0.35);
    assert_eq!(policy.face_exposure_warn_ratio, 0.25);
    assert_eq!(policy.face_color_cast_warn_threshold, 0.42);
}

fn flat_sample(width: usize, height: usize, value: u8) -> PreviewSample {
    PreviewSample {
        width,
        height,
        luma: vec![value; width * height],
        red: None,
        green: None,
        blue: None,
        preview_source: Some("test".to_string()),
    }
}

fn scattered_highlight_sample(width: usize, height: usize, clipped_count: usize) -> PreviewSample {
    let mut luma = vec![128; width * height];
    for i in 0..clipped_count.min(luma.len()) {
        let index = (i * 37) % luma.len();
        luma[index] = 248;
    }
    PreviewSample {
        width,
        height,
        luma,
        red: None,
        green: None,
        blue: None,
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
        red: None,
        green: None,
        blue: None,
        preview_source: Some("test".to_string()),
    }
}

fn color_sample(width: usize, height: usize, red: u8, green: u8, blue: u8) -> PreviewSample {
    let pixel_count = width * height;
    let mut luma = Vec::with_capacity(pixel_count);
    let mut red_values = Vec::with_capacity(pixel_count);
    let mut green_values = Vec::with_capacity(pixel_count);
    let mut blue_values = Vec::with_capacity(pixel_count);
    for y in 0..height {
        for x in 0..width {
            let multiplier = if (x + y) % 2 == 0 { 1.0 } else { 0.65 };
            let pixel_red = ((red as f64) * multiplier).round().clamp(0.0, 255.0) as u8;
            let pixel_green = ((green as f64) * multiplier).round().clamp(0.0, 255.0) as u8;
            let pixel_blue = ((blue as f64) * multiplier).round().clamp(0.0, 255.0) as u8;
            luma.push(
                (pixel_red as f64 * 0.2126
                    + pixel_green as f64 * 0.7152
                    + pixel_blue as f64 * 0.0722)
                    .round()
                    .clamp(0.0, 255.0) as u8,
            );
            red_values.push(pixel_red);
            green_values.push(pixel_green);
            blue_values.push(pixel_blue);
        }
    }
    PreviewSample {
        width,
        height,
        luma,
        red: Some(red_values),
        green: Some(green_values),
        blue: Some(blue_values),
        preview_source: Some("test".to_string()),
    }
}
