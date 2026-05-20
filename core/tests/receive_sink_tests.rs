use nikon_importer_core::{LocalFileSink, ReceiveState};

#[test]
fn writes_temp_file_then_publishes_final_file() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let sink = LocalFileSink::new(temp_dir.path());

    let progress = sink
        .write_complete("ftp-42", "DSC_1234.NEF", &[1, 2, 3, 4])
        .expect("file should write");

    let final_path = temp_dir.path().join("DSC_1234.NEF");
    let temp_path = temp_dir.path().join("DSC_1234.NEF.tmp");

    assert_eq!(progress.state, ReceiveState::Completed);
    assert_eq!(progress.bytes_written, 4);
    assert!(final_path.exists());
    assert!(!temp_path.exists());
}

#[test]
fn sanitizes_windows_unsafe_filename_characters() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let sink = LocalFileSink::new(temp_dir.path());

    sink.write_complete("ftp-42", "NIKON/../DSC:1234?.NEF", &[1])
        .expect("file should write");

    assert!(temp_dir.path().join("NIKON").join("DSC_1234_.NEF").exists());
}

#[test]
fn creates_sanitized_upload_directories_without_marker_files() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let sink = LocalFileSink::new(temp_dir.path());

    sink.create_dir_all("NIKON/../CARD:1")
        .expect("directory should be created");

    let directory = temp_dir.path().join("NIKON").join("CARD_1");
    assert!(directory.is_dir());
    assert!(!directory.join(".keep").exists());
}

#[test]
fn duplicate_uploads_are_published_with_numbered_filenames() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let sink = LocalFileSink::new(temp_dir.path());

    sink.write_complete("ftp-1", "DSC_2467.NEF", &[1])
        .expect("first file should write");
    let duplicate = sink
        .write_complete("ftp-2", "DSC_2467.NEF", &[2, 3])
        .expect("duplicate file should write");

    assert_eq!(
        std::fs::read(temp_dir.path().join("DSC_2467.NEF")).unwrap(),
        vec![1]
    );
    assert_eq!(
        std::fs::read(temp_dir.path().join("DSC_2467 (1).NEF")).unwrap(),
        vec![2, 3]
    );
    assert_eq!(duplicate.filename, "DSC_2467 (1).NEF");
}
