    use super::*;
    use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb};
    use std::io::{Read, Write};

    #[test]
    fn write_thumbnail_applies_exif_orientation() {
        let temp_dir = unique_temp_dir("desktop-thumbnail-orientation");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let source_path = temp_dir.join("portrait-with-orientation.jpg");
        let output_path = temp_dir.join("thumb.jpg");
        fs::write(&source_path, jpeg_with_exif_orientation(6)).expect("source should write");

        write_thumbnail_with_quality(&source_path, &output_path, 64, ThumbnailQuality::Fast)
            .expect("thumbnail should write");

        let thumbnail = image::open(&output_path).expect("thumbnail should decode");
        assert_eq!(thumbnail.dimensions(), (64, 43));
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn write_thumbnail_prefers_embedded_jpeg_preview() {
        let temp_dir = unique_temp_dir("desktop-thumbnail-embedded-preview");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let source_path = temp_dir.join("image-with-preview.jpg");
        let output_path = temp_dir.join("thumb.jpg");
        fs::write(&source_path, jpeg_with_embedded_preview()).expect("source should write");

        write_thumbnail_with_quality(&source_path, &output_path, 64, ThumbnailQuality::Fast)
            .expect("thumbnail should write");

        let thumbnail = image::open(&output_path).expect("thumbnail should decode");
        let center = thumbnail
            .to_rgb8()
            .get_pixel(thumbnail.width() / 2, thumbnail.height() / 2)
            .0;
        assert!(
            center[2] > center[0],
            "thumbnail should be generated from the blue embedded preview, got rgb {center:?}"
        );
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn write_full_thumbnail_ignores_embedded_jpeg_preview() {
        let temp_dir = unique_temp_dir("desktop-thumbnail-full-quality");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let source_path = temp_dir.join("image-with-preview.jpg");
        let output_path = temp_dir.join("thumb.jpg");
        fs::write(&source_path, jpeg_with_embedded_preview()).expect("source should write");

        write_thumbnail_with_quality(&source_path, &output_path, 64, ThumbnailQuality::Full)
            .expect("thumbnail should write");

        let thumbnail = image::open(&output_path).expect("thumbnail should decode");
        let center = thumbnail
            .to_rgb8()
            .get_pixel(thumbnail.width() / 2, thumbnail.height() / 2)
            .0;
        assert!(
            center[0] > center[2],
            "full thumbnail should be generated from the red source image, got rgb {center:?}"
        );
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn thumbnail_request_allows_1280_edge() {
        let temp_dir = unique_temp_dir("desktop-thumbnail-1280-edge");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let source_path = temp_dir.join("source.jpg");
        fs::write(&source_path, encode_solid_jpeg(1600, 900, [220, 12, 8]))
            .expect("source should write");

        let response = get_asset_thumbnail_blocking(
            temp_dir.clone(),
            ThumbnailRequest {
                source_path: source_path.to_string_lossy().into_owned(),
                max_edge: Some(1280),
                quality: Some(ThumbnailQuality::Full),
            },
        )
        .expect("thumbnail should write");

        let thumbnail = image::open(response.path).expect("thumbnail should decode");
        assert_eq!(thumbnail.dimensions(), (1280, 720));
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn original_preview_reuses_browser_decodable_source() {
        let temp_dir = unique_temp_dir("desktop-original-preview-source");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let source_path = temp_dir.join("source.jpg");
        fs::write(&source_path, encode_solid_jpeg(80, 60, [220, 12, 8]))
            .expect("source should write");

        let response = get_asset_original_preview_blocking(
            temp_dir.clone(),
            OriginalPreviewRequest {
                source_path: source_path.to_string_lossy().into_owned(),
            },
        )
        .expect("browser original should return source path");

        assert_eq!(PathBuf::from(response.path), source_path);
        assert!(response.direct_source);
        assert!(response.cached);
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn write_original_preview_image_does_not_apply_thumbnail_clamp() {
        let temp_dir = unique_temp_dir("desktop-original-preview-large");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let output_path = temp_dir.join("original.jpg");
        let mut image =
            DynamicImage::ImageRgb8(ImageBuffer::from_pixel(1600, 900, Rgb([10, 20, 30])));

        write_original_preview_image(&mut image, &output_path)
            .expect("original preview should write");

        let output = image::open(&output_path).expect("original preview should decode");
        assert_eq!(output.dimensions(), (1600, 900));
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn write_thumbnail_reads_embedded_preview_from_raw_tiff_container() {
        let temp_dir = unique_temp_dir("desktop-thumbnail-raw-preview");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let source_path = temp_dir.join("raw-only.nef");
        let output_path = temp_dir.join("thumb.jpg");
        fs::write(&source_path, raw_tiff_with_embedded_preview()).expect("source should write");

        write_thumbnail_with_quality(&source_path, &output_path, 64, ThumbnailQuality::Fast)
            .expect("thumbnail should write");

        let thumbnail = image::open(&output_path).expect("thumbnail should decode");
        let center = thumbnail
            .to_rgb8()
            .get_pixel(thumbnail.width() / 2, thumbnail.height() / 2)
            .0;
        assert!(
            center[1] > center[0] && center[1] > center[2],
            "raw thumbnail should be generated from the green embedded preview, got rgb {center:?}"
        );
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn write_thumbnail_applies_raw_tiff_orientation_to_embedded_preview() {
        let temp_dir = unique_temp_dir("desktop-thumbnail-raw-preview-orientation");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let source_path = temp_dir.join("raw-rotated.nef");
        let output_path = temp_dir.join("thumb.jpg");
        fs::write(
            &source_path,
            raw_tiff_with_embedded_preview_and_orientation(6),
        )
        .expect("source should write");

        write_thumbnail_with_quality(&source_path, &output_path, 64, ThumbnailQuality::Fast)
            .expect("thumbnail should write");

        let thumbnail = image::open(&output_path).expect("thumbnail should decode");
        assert!(
            thumbnail.height() > thumbnail.width(),
            "raw embedded preview orientation should rotate to portrait, got {:?}",
            thumbnail.dimensions()
        );
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn raw_sensor_thumbnail_uses_raw_pixels() {
        let mut image = rawloader::RawImage {
            make: "Test".to_string(),
            model: "Sensor".to_string(),
            clean_make: "Test".to_string(),
            clean_model: "Sensor".to_string(),
            width: 2,
            height: 2,
            cpp: 1,
            wb_coeffs: [1.0, 1.0, 1.0, 1.0],
            whitelevels: [1023, 1023, 1023, 1023],
            blacklevels: [0, 0, 0, 0],
            xyz_to_cam: [[0.0; 3]; 4],
            cfa: rawloader::CFA::new("RGGB"),
            crops: [0, 0, 0, 0],
            blackareas: Vec::new(),
            orientation: rawloader::Orientation::Normal,
            data: rawloader::RawImageData::Integer(vec![1023, 256, 512, 768]),
        };

        let thumbnail = raw_sensor_thumbnail_image(&mut image).expect("raw sensor should render");

        assert_eq!(thumbnail.dimensions(), (2, 2));
        let center = thumbnail.to_rgb8().get_pixel(0, 0).0;
        assert!(
            center[0] > center[1] && center[0] > center[2],
            "red pixel should stay red, got {center:?}"
        );
    }

    #[test]
    fn sync_project_snapshot_from_path_returns_compact_counts() {
        let temp_dir = unique_temp_dir("desktop-project-sync-snapshot");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let root = temp_dir.join("photos");
        fs::create_dir_all(&root).expect("photo root should create");
        fs::write(root.join("IMG_5100.JPG"), [1_u8, 2, 3, 4]).expect("jpeg should write");

        let service = CameraConnectorService::new(Some(temp_dir.join("config.json")));
        let project = service
            .create_project("Desktop Snapshot Sync")
            .expect("project should create");
        let scan = service
            .create_desktop_project_scan(&project.project_id, &root)
            .expect("scan should queue");
        service
            .run_desktop_project_scan(&scan.scan_id)
            .expect("scan should complete");

        let snapshot_path = temp_dir.join("snapshot.json");
        fs::write(
            &snapshot_path,
            r#"{
              "schema_version": 1,
              "source_device": {"device_id": "phone", "device_label": "Phone", "platform": "android"},
              "project": {"project_id": "phone-project", "name": "Phone Project", "exported_at_ms": 1781800000000},
              "assets": [{
                "asset_id": "remote-asset",
                "group_id": "remote-group",
                "original_filename": "IMG_5100.JPG",
                "final_filename": "IMG_5100.JPG",
                "normalized_stem": "IMG_5100",
                "original_path": "Android/DCIM/IMG_5100.JPG",
                "original_parent_path": "Android/DCIM",
                "format": "jpeg",
                "size_bytes": 4,
                "capture_at_ms": null,
                "received_at_ms": null,
                "source_identity": null
              }],
              "groups": [{
                "group_id": "remote-group",
                "display_key": "IMG_5100",
                "source_identity": null,
                "original_parent_path": "Android/DCIM",
                "member_asset_ids": ["remote-asset"],
                "primary_asset_id": "remote-asset",
                "preview_asset_id": "remote-asset",
                "has_raw": false,
                "has_jpeg": true,
                "has_video": false
              }],
              "model_evaluations": [],
              "selection_recommendations": [],
              "user_marks": [{"group_id": "remote-group", "favorite": true, "marked": null}]
            }"#,
        )
        .expect("snapshot should write");

        let response = sync_project_snapshot_from_path_blocking(
            &service,
            SyncProjectSnapshotRequest {
                project_id: project.project_id,
                snapshot_path,
            },
        )
        .expect("snapshot sync should return counts");

        assert_eq!(response.matched_assets, 1);
        assert_eq!(response.matched_groups, 1);
        assert_eq!(response.applied_user_marks, 1);
        assert_eq!(response.unresolved_records, 0);
        assert_eq!(response.ambiguous_records, 0);

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn sync_project_snapshot_from_url_fetches_snapshot_and_returns_compact_counts() {
        let temp_dir = unique_temp_dir("desktop-project-sync-snapshot-url");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let root = temp_dir.join("photos");
        fs::create_dir_all(&root).expect("photo root should create");
        fs::write(root.join("IMG_5200.JPG"), [1_u8, 2, 3, 4]).expect("jpeg should write");

        let service = CameraConnectorService::new(Some(temp_dir.join("config.json")));
        let project = service
            .create_project("Desktop Snapshot URL Sync")
            .expect("project should create");
        let scan = service
            .create_desktop_project_scan(&project.project_id, &root)
            .expect("scan should queue");
        service
            .run_desktop_project_scan(&scan.scan_id)
            .expect("scan should complete");

        let snapshot = r#"{
          "schema_version": 1,
          "source_device": {"device_id": "phone", "device_label": "Phone", "platform": "android"},
          "project": {"project_id": "phone-project", "name": "Phone Project", "exported_at_ms": 1781800000000},
          "assets": [{
            "asset_id": "remote-asset",
            "group_id": "remote-group",
            "original_filename": "IMG_5200.JPG",
            "final_filename": "IMG_5200.JPG",
            "normalized_stem": "IMG_5200",
            "original_path": "Android/DCIM/IMG_5200.JPG",
            "original_parent_path": "Android/DCIM",
            "format": "jpeg",
            "size_bytes": 4,
            "capture_at_ms": null,
            "received_at_ms": null,
            "source_identity": null
          }],
          "groups": [{
            "group_id": "remote-group",
            "display_key": "IMG_5200",
            "source_identity": null,
            "original_parent_path": "Android/DCIM",
            "member_asset_ids": ["remote-asset"],
            "primary_asset_id": "remote-asset",
            "preview_asset_id": "remote-asset",
            "has_raw": false,
            "has_jpeg": true,
            "has_video": false
          }],
          "model_evaluations": [],
          "selection_recommendations": [],
          "user_marks": [{"group_id": "remote-group", "favorite": true, "marked": null}]
        }"#;
        let url = serve_once(snapshot);

        let response = sync_project_snapshot_from_url_blocking(
            &service,
            SyncProjectSnapshotUrlRequest {
                project_id: project.project_id,
                snapshot_url: url,
            },
        )
        .expect("snapshot URL sync should return counts");

        assert_eq!(response.matched_assets, 1);
        assert_eq!(response.matched_groups, 1);
        assert_eq!(response.applied_user_marks, 1);
        assert_eq!(response.unresolved_records, 0);
        assert_eq!(response.ambiguous_records, 0);

        let _ = fs::remove_dir_all(temp_dir);
    }

    fn jpeg_with_exif_orientation(orientation: u16) -> Vec<u8> {
        let image = ImageBuffer::from_fn(2, 3, |x, y| {
            Rgb([
                (40 + x * 80) as u8,
                (50 + y * 50) as u8,
                (120 + x * 20 + y * 5) as u8,
            ])
        });
        let mut jpeg = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut jpeg),
                image::ImageFormat::Jpeg,
            )
            .expect("jpeg should encode");
        insert_exif_orientation(&mut jpeg, orientation);
        jpeg
    }

    fn jpeg_with_embedded_preview() -> Vec<u8> {
        let image = ImageBuffer::from_fn(48, 32, |_x, _y| Rgb([220u8, 12, 8]));
        let mut jpeg = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut jpeg),
                image::ImageFormat::Jpeg,
            )
            .expect("jpeg should encode");
        let preview = encode_solid_jpeg(16, 12, [8, 30, 220]);
        insert_exif_orientation_and_preview(&mut jpeg, 1, &preview);
        jpeg
    }

    fn encode_solid_jpeg(width: u32, height: u32, color: [u8; 3]) -> Vec<u8> {
        let image = ImageBuffer::from_pixel(width, height, Rgb(color));
        let mut jpeg = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut jpeg),
                image::ImageFormat::Jpeg,
            )
            .expect("jpeg should encode");
        jpeg
    }

    fn raw_tiff_with_embedded_preview() -> Vec<u8> {
        raw_tiff_with_embedded_preview_and_orientation(1)
    }

    fn raw_tiff_with_embedded_preview_and_orientation(orientation: u16) -> Vec<u8> {
        let preview = encode_solid_jpeg(18, 12, [10, 220, 30]);
        let ifd_offset = 8u32;
        let entry_count = 3u16;
        let ifd_size = 2u32 + u32::from(entry_count) * 12 + 4;
        let preview_offset = ifd_offset + ifd_size;

        let mut tiff = Vec::new();
        tiff.extend_from_slice(b"II");
        tiff.extend_from_slice(&42u16.to_le_bytes());
        tiff.extend_from_slice(&ifd_offset.to_le_bytes());
        tiff.extend_from_slice(&entry_count.to_le_bytes());
        tiff.extend_from_slice(&0x0112u16.to_le_bytes());
        tiff.extend_from_slice(&3u16.to_le_bytes());
        tiff.extend_from_slice(&1u32.to_le_bytes());
        tiff.extend_from_slice(&orientation.to_le_bytes());
        tiff.extend_from_slice(&0u16.to_le_bytes());
        tiff.extend_from_slice(&0x0201u16.to_le_bytes());
        tiff.extend_from_slice(&4u16.to_le_bytes());
        tiff.extend_from_slice(&1u32.to_le_bytes());
        tiff.extend_from_slice(&preview_offset.to_le_bytes());
        tiff.extend_from_slice(&0x0202u16.to_le_bytes());
        tiff.extend_from_slice(&4u16.to_le_bytes());
        tiff.extend_from_slice(&1u32.to_le_bytes());
        tiff.extend_from_slice(&(preview.len() as u32).to_le_bytes());
        tiff.extend_from_slice(&0u32.to_le_bytes());
        tiff.extend_from_slice(&preview);
        tiff
    }

    fn insert_exif_orientation(jpeg: &mut Vec<u8>, orientation: u16) {
        assert!(jpeg.starts_with(&[0xff, 0xd8]));
        let mut payload = Vec::new();
        payload.extend_from_slice(b"Exif\0\0");
        payload.extend_from_slice(b"II");
        payload.extend_from_slice(&42u16.to_le_bytes());
        payload.extend_from_slice(&8u32.to_le_bytes());
        payload.extend_from_slice(&1u16.to_le_bytes());
        payload.extend_from_slice(&0x0112u16.to_le_bytes());
        payload.extend_from_slice(&3u16.to_le_bytes());
        payload.extend_from_slice(&1u32.to_le_bytes());
        payload.extend_from_slice(&orientation.to_le_bytes());
        payload.extend_from_slice(&0u16.to_le_bytes());
        payload.extend_from_slice(&0u32.to_le_bytes());

        let length = (payload.len() + 2) as u16;
        let mut segment = vec![0xff, 0xe1];
        segment.extend_from_slice(&length.to_be_bytes());
        segment.extend_from_slice(&payload);
        jpeg.splice(2..2, segment);
    }

    fn insert_exif_orientation_and_preview(jpeg: &mut Vec<u8>, orientation: u16, preview: &[u8]) {
        assert!(jpeg.starts_with(&[0xff, 0xd8]));
        let ifd0_offset = 8u32;
        let ifd0_entry_count = 1u16;
        let ifd0_size = 2u32 + u32::from(ifd0_entry_count) * 12 + 4;
        let ifd1_offset = ifd0_offset + ifd0_size;
        let ifd1_entry_count = 2u16;
        let ifd1_size = 2u32 + u32::from(ifd1_entry_count) * 12 + 4;
        let preview_offset = ifd1_offset + ifd1_size;

        let mut tiff = Vec::new();
        tiff.extend_from_slice(b"II");
        tiff.extend_from_slice(&42u16.to_le_bytes());
        tiff.extend_from_slice(&ifd0_offset.to_le_bytes());
        tiff.extend_from_slice(&ifd0_entry_count.to_le_bytes());
        tiff.extend_from_slice(&0x0112u16.to_le_bytes());
        tiff.extend_from_slice(&3u16.to_le_bytes());
        tiff.extend_from_slice(&1u32.to_le_bytes());
        tiff.extend_from_slice(&orientation.to_le_bytes());
        tiff.extend_from_slice(&0u16.to_le_bytes());
        tiff.extend_from_slice(&ifd1_offset.to_le_bytes());
        tiff.extend_from_slice(&ifd1_entry_count.to_le_bytes());
        tiff.extend_from_slice(&0x0201u16.to_le_bytes());
        tiff.extend_from_slice(&4u16.to_le_bytes());
        tiff.extend_from_slice(&1u32.to_le_bytes());
        tiff.extend_from_slice(&preview_offset.to_le_bytes());
        tiff.extend_from_slice(&0x0202u16.to_le_bytes());
        tiff.extend_from_slice(&4u16.to_le_bytes());
        tiff.extend_from_slice(&1u32.to_le_bytes());
        tiff.extend_from_slice(&(preview.len() as u32).to_le_bytes());
        tiff.extend_from_slice(&0u32.to_le_bytes());
        tiff.extend_from_slice(preview);

        let mut payload = Vec::new();
        payload.extend_from_slice(b"Exif\0\0");
        payload.extend_from_slice(&tiff);
        let length = (payload.len() + 2) as u16;
        let mut segment = vec![0xff, 0xe1];
        segment.extend_from_slice(&length.to_be_bytes());
        segment.extend_from_slice(&payload);
        jpeg.splice(2..2, segment);
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("{name}-{}", current_time_ms()))
    }

    fn serve_once(body: &'static str) -> String {
        let listener =
            std::net::TcpListener::bind("127.0.0.1:0").expect("test HTTP listener should bind");
        let url = format!("http://{}/project-snapshot", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("test HTTP request should arrive");
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request);
            let response = format!(
                "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .expect("test HTTP response should write");
        });
        url
    }
