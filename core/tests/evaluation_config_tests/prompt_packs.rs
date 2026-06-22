use std::fs;

use super::*;
#[test]
fn evaluation_config_tests_service_creates_user_prompt_pack_from_shared_preference() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));

    let pack = service
        .create_global_prompt_pack(
            "Documentary Preference",
            vec!["documentary".to_string(), "portrait".to_string()],
            SceneProfile::General,
            "user",
            "Prefer quiet documentary emotion.",
            10_000,
        )
        .expect("prompt pack should create");

    assert_eq!(pack.name, "Documentary Preference");
    assert_eq!(pack.prompt_pack_id, "documentary-preference");
    assert!(!pack.built_in);
    assert_eq!(pack.version, "user-10000");
    assert!(pack.prompt_hash.starts_with("fnv1a64-"));
    assert!(pack.prompt_text.contains("shared_preference"));
    assert!(pack
        .prompt_text
        .contains("Prefer quiet documentary emotion."));
    assert!(service
        .prompt_text_for_pack(&pack.prompt_pack_id)
        .expect("prompt text should load")
        .expect("prompt text should exist")
        .contains("Prefer quiet documentary emotion."));
    assert_eq!(
        service
            .prompt_markdown_for_pack(&pack.prompt_pack_id)
            .expect("prompt markdown should load")
            .expect("prompt markdown should exist"),
        "Prefer quiet documentary emotion."
    );

    let prompt_pack_root = service
        .storage_state_dir()
        .expect("storage state dir should resolve")
        .join("prompt-packs");
    assert!(prompt_pack_root
        .join("user")
        .join("documentary-preference")
        .exists());
    let prompt_file = fs::read_dir(prompt_pack_root.join("user"))
        .expect("prompt pack root should exist")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path().join("PROMPT.md"))
        .find(|path| path.exists())
        .expect("prompt markdown file should exist");
    let prompt_markdown = fs::read_to_string(prompt_file).expect("prompt markdown should read");
    assert_eq!(prompt_markdown, "Prefer quiet documentary emotion.");
    assert!(!prompt_markdown.contains("shared_preference"));
}

#[test]
fn evaluation_config_tests_user_prompt_pack_uses_shareable_folder_names() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));

    let first = service
        .create_global_prompt_pack(
            "xx",
            vec!["test".to_string()],
            SceneProfile::General,
            "portrait-pack",
            "First prompt.",
            10_000,
        )
        .expect("first prompt pack should create");
    let second = service
        .create_global_prompt_pack(
            "xx",
            vec!["test".to_string()],
            SceneProfile::General,
            "portrait-pack",
            "Second prompt.",
            10_001,
        )
        .expect("duplicate prompt pack should create with short suffix");

    assert_eq!(first.prompt_pack_id, "xx");
    assert_eq!(first.distribution_folder, "portrait-pack");
    assert_eq!(second.prompt_pack_id, "xx-2");
    assert_eq!(second.distribution_folder, "portrait-pack");

    let prompt_pack_root = service
        .storage_state_dir()
        .expect("storage state dir should resolve")
        .join("prompt-packs");
    assert_eq!(
        fs::read_to_string(
            prompt_pack_root
                .join("portrait-pack")
                .join("xx")
                .join("PROMPT.md"),
        )
        .expect("first prompt should read"),
        "First prompt."
    );
    assert_eq!(
        fs::read_to_string(
            prompt_pack_root
                .join("portrait-pack")
                .join("xx-2")
                .join("PROMPT.md"),
        )
        .expect("second prompt should read"),
        "Second prompt."
    );
}

#[test]
fn evaluation_config_tests_user_prompt_pack_keeps_json_like_markdown_literal() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let json_like_markdown = r#"{"shared_preference":"Keep this literal markdown."}"#;

    let pack = service
        .create_global_prompt_pack(
            "Json Looking Prompt",
            vec!["literal".to_string()],
            SceneProfile::General,
            "user",
            json_like_markdown,
            12_000,
        )
        .expect("prompt pack should create");

    assert_eq!(
        service
            .prompt_markdown_for_pack(&pack.prompt_pack_id)
            .expect("prompt markdown should load")
            .expect("prompt markdown should exist"),
        json_like_markdown
    );

    let prompt_file = service
        .storage_state_dir()
        .expect("storage state dir should resolve")
        .join("prompt-packs")
        .join("user")
        .join(&pack.prompt_pack_id)
        .join("PROMPT.md");
    assert_eq!(
        fs::read_to_string(prompt_file).expect("prompt markdown should read"),
        json_like_markdown
    );
}

#[test]
fn evaluation_config_tests_delete_user_prompt_pack_removes_files_and_project_references() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Prompt Delete Project")
        .expect("project should create");
    let pack = service
        .create_global_prompt_pack(
            "Delete Me",
            vec!["custom".to_string()],
            SceneProfile::General,
            "my-pack",
            "Temporary preference.",
            13_000,
        )
        .expect("prompt pack should create");
    let prompt_pack_dir = service
        .storage_state_dir()
        .expect("storage state dir should resolve")
        .join("prompt-packs")
        .join("my-pack")
        .join(&pack.prompt_pack_id);

    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .expect("settings should load")
        .expect("settings should exist");
    settings.prompt_pack_id = Some(pack.prompt_pack_id.clone());
    service
        .save_project_evaluation_settings(settings)
        .expect("settings should save selected prompt pack");

    let deleted = service
        .delete_global_prompt_pack(&pack.prompt_pack_id)
        .expect("prompt pack should delete");

    assert!(deleted);
    assert!(!prompt_pack_dir.exists());
    assert!(service
        .prompt_pack_by_id(&pack.prompt_pack_id)
        .expect("prompt pack lookup should succeed")
        .is_none());
    assert_eq!(
        service
            .project_evaluation_settings(&project.project_id)
            .expect("settings should reload")
            .expect("settings should exist")
            .prompt_pack_id,
        None
    );
}

#[test]
fn evaluation_config_tests_delete_built_in_prompt_pack_is_rejected() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));

    let error = service
        .delete_global_prompt_pack("general-default")
        .expect_err("built-in prompt pack should not delete")
        .to_string();

    assert!(error.contains("built-in prompt packs cannot be deleted"));
    assert!(service
        .prompt_pack_by_id("general-default")
        .expect("built-in prompt pack should still load")
        .is_some());
}

#[test]
fn evaluation_config_tests_delete_user_prompt_package_removes_all_packs_in_package() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Prompt Package Delete Project")
        .expect("project should create");
    let first = service
        .create_global_prompt_pack(
            "Package One",
            vec!["custom".to_string()],
            SceneProfile::General,
            "shareable-pack",
            "First preference.",
            14_000,
        )
        .expect("first prompt pack should create");
    let second = service
        .create_global_prompt_pack(
            "Package Two",
            vec!["custom".to_string()],
            SceneProfile::Portrait,
            "shareable-pack",
            "Second preference.",
            14_001,
        )
        .expect("second prompt pack should create");
    let other = service
        .create_global_prompt_pack(
            "Other Package",
            vec!["custom".to_string()],
            SceneProfile::General,
            "other-pack",
            "Other preference.",
            14_002,
        )
        .expect("other prompt pack should create");

    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .expect("settings should load")
        .expect("settings should exist");
    settings.prompt_pack_id = Some(second.prompt_pack_id.clone());
    service
        .save_project_evaluation_settings(settings)
        .expect("settings should save selected prompt pack");

    let deleted = service
        .delete_global_prompt_package("shareable-pack")
        .expect("prompt package should delete");

    assert!(deleted);
    assert!(service
        .prompt_pack_by_id(&first.prompt_pack_id)
        .expect("first lookup should succeed")
        .is_none());
    assert!(service
        .prompt_pack_by_id(&second.prompt_pack_id)
        .expect("second lookup should succeed")
        .is_none());
    assert!(service
        .prompt_pack_by_id(&other.prompt_pack_id)
        .expect("other lookup should succeed")
        .is_some());
    assert_eq!(
        service
            .project_evaluation_settings(&project.project_id)
            .expect("settings should reload")
            .expect("settings should exist")
            .prompt_pack_id,
        None
    );
    assert!(!service
        .storage_state_dir()
        .expect("storage state dir should resolve")
        .join("prompt-packs")
        .join("shareable-pack")
        .exists());
}

#[test]
fn evaluation_config_tests_delete_built_in_prompt_package_is_rejected() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));

    let error = service
        .delete_global_prompt_package("builtin")
        .expect_err("built-in prompt package should not delete")
        .to_string();

    assert!(error.contains("built-in prompt package cannot be deleted"));
}

#[test]
fn evaluation_config_tests_corrupt_user_prompt_pack_reports_load_error() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    service
        .create_global_prompt_pack(
            "Broken Pack",
            vec!["broken".to_string()],
            SceneProfile::General,
            "user",
            "This prompt will be corrupted.",
            11_000,
        )
        .expect("prompt pack should create");

    let manifest_file = fs::read_dir(
        service
            .storage_state_dir()
            .expect("storage state dir should resolve")
            .join("prompt-packs")
            .join("user"),
    )
    .expect("prompt pack root should exist")
    .filter_map(|entry| entry.ok())
    .map(|entry| entry.path().join("manifest.json"))
    .find(|path| path.exists())
    .expect("manifest should exist");
    fs::write(manifest_file, "{not valid json").expect("manifest should corrupt");

    let error = service
        .global_prompt_packs()
        .expect_err("corrupt user prompt pack should fail loudly")
        .to_string();
    assert!(!error.contains("prompt pack not found"));
}
