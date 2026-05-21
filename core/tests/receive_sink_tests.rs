use camera_connector_core::{
    LocalFileSink, ReceiveState, ReceiveStorage, ReceiveUpload, StoredObjectLocation,
};

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
    assert_eq!(
        progress.output_location,
        Some(StoredObjectLocation::local_path(final_path.clone()))
    );
    assert!(final_path.exists());
    assert!(!temp_path.exists());
}

#[test]
fn streams_to_temp_file_before_publishing_final_file() {
    use std::io::Write;

    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let sink = LocalFileSink::new(temp_dir.path());

    let mut upload = sink
        .begin_write("sftp-42", "DCIM/100CAM/IMG_0001.NEF")
        .expect("upload should begin");
    upload.write_all(&[1, 2]).expect("first chunk should write");
    upload
        .write_all(&[3, 4])
        .expect("second chunk should write");

    let final_path = temp_dir.path().join("IMG_0001.NEF");
    let temp_path = temp_dir.path().join("IMG_0001.NEF.tmp");
    assert!(!final_path.exists());
    assert!(temp_path.exists());

    let progress = upload.finish().expect("upload should publish");

    assert_eq!(progress.state, ReceiveState::Completed);
    assert_eq!(progress.bytes_written, 4);
    assert_eq!(std::fs::read(final_path).unwrap(), vec![1, 2, 3, 4]);
    assert!(!temp_path.exists());
}

#[test]
fn sanitizes_windows_unsafe_filename_characters() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let sink = LocalFileSink::new(temp_dir.path());

    sink.write_complete("ftp-42", "NIKON/../DSC:1234?.NEF", &[1])
        .expect("file should write");

    assert!(temp_dir.path().join("DSC_1234_.NEF").exists());
    assert!(!temp_dir.path().join("NIKON").exists());
}

#[test]
fn ignores_remote_directory_creation_for_flat_inbox() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let sink = LocalFileSink::new(temp_dir.path());

    let directory = sink
        .create_dir_all("NIKON/../CARD:1")
        .expect("directory command should be accepted");

    assert!(directory.is_dir());
    assert_eq!(directory, temp_dir.path());
    assert!(!temp_dir.path().join("NIKON").exists());
    assert!(!directory.join(".keep").exists());
}

#[test]
fn remote_upload_paths_are_flattened_to_filename_only() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let sink = LocalFileSink::new(temp_dir.path());

    let progress = sink
        .write_complete("ftp-42", "/DCIM/100CANON/IMG_1001.CR3", &[1])
        .expect("file should write");

    assert_eq!(progress.filename, "IMG_1001.CR3");
    assert!(temp_dir.path().join("IMG_1001.CR3").exists());
    assert!(!temp_dir.path().join("DCIM").exists());
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

#[test]
fn local_file_sink_implements_storage_backend_contract() {
    fn receive_with_backend<S: ReceiveStorage>(
        storage: &S,
        transfer_id: &str,
        relative_path: &str,
        bytes: &[u8],
    ) -> camera_connector_core::Result<camera_connector_core::ReceiveProgress> {
        let mut upload = storage.begin_write(transfer_id, relative_path)?;
        std::io::Write::write_all(&mut upload, bytes)?;
        upload.finish()
    }

    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let sink = LocalFileSink::new(temp_dir.path());

    let progress = receive_with_backend(&sink, "ftp-3", "DCIM/IMG_3000.RAF", &[9, 8, 7])
        .expect("generic storage backend should receive file");

    assert_eq!(progress.filename, "IMG_3000.RAF");
    assert_eq!(progress.bytes_written, 3);
    assert_eq!(
        progress.output_location,
        Some(StoredObjectLocation::local_path(
            temp_dir.path().join("IMG_3000.RAF")
        ))
    );
}
