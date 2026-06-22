    use super::*;
    use camera_connector_core::{ObjectFormat, StoredObjectLocation};
    use image::{ImageBuffer, Rgb};
    use std::fs;

    #[test]
    fn preview_sample_from_image_preserves_luma_and_rgb_channels() {
        let mut image = DynamicImage::ImageRgb8(ImageBuffer::from_fn(2, 1, |x, _y| {
            if x == 0 {
                Rgb([255, 0, 0])
            } else {
                Rgb([0, 255, 0])
            }
        }));

        let sample = preview_sample_from_image(&mut image, Some("unit".to_string()), 16);

        assert_eq!(sample.width, 2);
        assert_eq!(sample.height, 1);
        assert_eq!(sample.luma, vec![54, 182]);
        assert_eq!(sample.red.as_deref(), Some([255, 0].as_slice()));
        assert_eq!(sample.green.as_deref(), Some([0, 255].as_slice()));
        assert_eq!(sample.blue.as_deref(), Some([0, 0].as_slice()));
        assert_eq!(sample.preview_source.as_deref(), Some("unit"));
    }

    #[test]
    fn best_asset_for_cv_uses_core_format_role_and_prefers_jpeg() {
        let temp_dir = unique_temp_dir("desktop-cv-photo-media-kind");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let raw_path = temp_dir.join("sample.nef");
        let jpeg_path = temp_dir.join("sample.jpg");
        fs::write(&raw_path, b"raw").expect("raw placeholder should write");
        fs::write(&jpeg_path, b"jpeg").expect("jpeg placeholder should write");

        let assets = vec![
            stored_cv_asset("raw", "raw", ObjectFormat::Nef, &raw_path),
            stored_cv_asset("jpeg", "jpeg", ObjectFormat::Jpeg, &jpeg_path),
        ];

        let selected = best_asset_for_cv(&assets).expect("photo asset should be selected");
        assert_eq!(selected.asset_id, "jpeg");
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn desktop_face_detector_loads_bundled_model() {
        desktop_face_detector().expect("bundled PICO model should load");
    }

    #[test]
    fn analyze_face_region_uses_project_clip_thresholds() {
        let image = ImageBuffer::from_fn(2, 1, |x, _y| {
            if x == 0 {
                Rgb([5, 5, 5])
            } else {
                Rgb([250, 250, 250])
            }
        });

        let standard =
            analyze_face_region(&image, 0, 0, 2, 1, TechnicalAssessmentPolicy::standard());
        let relaxed = analyze_face_region(
            &image,
            0,
            0,
            2,
            1,
            TechnicalAssessmentPolicy {
                shadow_clip_threshold: 0,
                highlight_clip_threshold: 255,
                ..TechnicalAssessmentPolicy::standard()
            },
        );

        assert_eq!(standard.shadow_ratio, 0.5);
        assert_eq!(standard.highlight_ratio, 0.5);
        assert_eq!(relaxed.shadow_ratio, 0.0);
        assert_eq!(relaxed.highlight_ratio, 0.0);
    }

    fn stored_cv_asset(
        asset_id: &str,
        group_role: &str,
        format: ObjectFormat,
        path: &Path,
    ) -> StoredAsset {
        StoredAsset {
            asset_id: asset_id.to_string(),
            project_id: "project".to_string(),
            group_id: Some("group".to_string()),
            transfer_id: asset_id.to_string(),
            group_role: group_role.to_string(),
            media_kind: "photo".to_string(),
            format,
            original_filename: path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(asset_id)
                .to_string(),
            final_filename: path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(asset_id)
                .to_string(),
            normalized_stem: "sample".to_string(),
            original_path: path.display().to_string(),
            original_parent_path: path.parent().map(|parent| parent.display().to_string()),
            final_location: Some(StoredObjectLocation::local_path(path)),
            size_bytes: 1,
            capture_at_ms: None,
            received_at_ms: None,
            published_at_ms: None,
            source_identity: None,
            username: None,
            remote_addr: None,
            source_status: "available".to_string(),
            source_modified_at_ms: None,
            last_seen_scan_id: None,
            duplicate_index: None,
            duplicate_count: None,
        }
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("{name}-{}", current_time_ms()))
    }
