use std::io::Write;

use camera_connector_core::{LocalFolderObjectStore, LocalStagingStore, StoredObjectLocation};

#[test]
fn local_staging_store_writes_and_finishes_staged_object() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let staging = LocalStagingStore::new(temp_dir.path().join("staging"));

    let mut upload = staging
        .begin_write("ftp:1", "DCIM/100/IMG_0001.NEF")
        .expect("staged upload should begin");
    upload.write_all(&[1, 2]).expect("bytes should write");
    upload
        .write_at(4, &[5, 6])
        .expect("random bytes should write");
    let staged = upload.finish().expect("staged upload should finish");

    assert_eq!(staged.transfer_id, "ftp:1");
    assert_eq!(staged.final_filename, "IMG_0001.NEF");
    assert_eq!(staged.bytes_written, 6);
    assert!(staged.staged_path.exists());
    assert_eq!(
        std::fs::read(&staged.staged_path).expect("staged bytes should read"),
        vec![1, 2, 0, 0, 5, 6]
    );
}

#[test]
fn local_folder_object_store_publishes_staged_object_and_removes_staged_file() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let output_dir = temp_dir.path().join("output");
    let staging = LocalStagingStore::new(temp_dir.path().join("staging"));
    let object_store = LocalFolderObjectStore::new(&output_dir);
    std::fs::create_dir_all(&output_dir).expect("output dir should create");
    std::fs::write(output_dir.join("IMG_0001.JPG"), [9]).expect("duplicate seed should write");

    let mut upload = staging
        .begin_write("ftp:2", "DCIM/100/IMG_0001.JPG")
        .expect("staged upload should begin");
    upload.write_all(&[1, 2, 3]).expect("bytes should write");
    let staged = upload.finish().expect("staged upload should finish");
    let staged_path = staged.staged_path.clone();

    let progress = object_store
        .publish(staged)
        .expect("staged object should publish");

    assert_eq!(progress.filename, "IMG_0001 (1).JPG");
    assert_eq!(progress.bytes_written, 3);
    assert!(!staged_path.exists());
    assert_eq!(
        std::fs::read(output_dir.join("IMG_0001 (1).JPG")).expect("published bytes should read"),
        vec![1, 2, 3]
    );
    assert_eq!(
        progress
            .output_location
            .as_ref()
            .map(StoredObjectLocation::kind),
        Some("local_path")
    );
}

#[test]
fn local_folder_object_store_keeps_staged_file_when_publish_fails() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let output_file = temp_dir.path().join("not-a-directory");
    std::fs::write(&output_file, [1]).expect("blocking file should write");
    let staging = LocalStagingStore::new(temp_dir.path().join("staging"));
    let object_store = LocalFolderObjectStore::new(&output_file);

    let mut upload = staging
        .begin_write("ftp:3", "IMG_0002.JPG")
        .expect("staged upload should begin");
    upload.write_all(&[7, 8]).expect("bytes should write");
    let staged = upload.finish().expect("staged upload should finish");
    let staged_path = staged.staged_path.clone();

    let result = object_store.publish(staged);

    assert!(result.is_err());
    assert!(staged_path.exists());
}
