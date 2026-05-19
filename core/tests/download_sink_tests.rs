use nikon_importer_core::{DownloadState, LocalFileSink};

#[test]
fn writes_temp_file_then_publishes_final_file() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let sink = LocalFileSink::new(temp_dir.path());

    let progress = sink
        .write_complete(42, "DSC_1234.NEF", &[1, 2, 3, 4])
        .expect("file should write");

    let final_path = temp_dir.path().join("DSC_1234.NEF");
    let temp_path = temp_dir.path().join("DSC_1234.NEF.tmp");

    assert_eq!(progress.state, DownloadState::Completed);
    assert_eq!(progress.bytes_written, 4);
    assert!(final_path.exists());
    assert!(!temp_path.exists());
}

#[test]
fn sanitizes_windows_unsafe_filename_characters() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let sink = LocalFileSink::new(temp_dir.path());

    sink.write_complete(42, "DSC:1234?.NEF", &[1])
        .expect("file should write");

    assert!(temp_dir.path().join("DSC_1234_.NEF").exists());
}
