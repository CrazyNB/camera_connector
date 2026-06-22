use super::*;

#[test]
fn parses_project_create_command() {
    let cli = Cli::try_parse_from([
        "camera-connector",
        "project",
        "--config",
        "C:\\CameraConnector\\config.json",
        "create",
        "--name",
        "Verify Shoot",
    ])
    .expect("project create command should parse");

    assert!(matches!(
        cli.command,
        Some(Command::Project {
            config: Some(_),
            action: ProjectCommand::Create { name },
        }) if name == "Verify Shoot"
    ));
}

#[test]
fn project_line_marks_active_project() {
    let project = camera_connector_core::Project {
        project_id: "project-1".to_string(),
        name: "Verify Shoot".to_string(),
        slug: "verify-shoot".to_string(),
        status: camera_connector_core::ProjectStatus::Active,
        created_at_ms: 10,
        updated_at_ms: 20,
        archived_at_ms: None,
        default_output_target_id: None,
    };

    let line = project_line(&project, Some("project-1"));

    assert!(line.contains("project\tid=project-1"));
    assert!(line.contains("name=Verify Shoot"));
    assert!(line.contains("slug=verify-shoot"));
    assert!(line.contains("status=active"));
    assert!(line.contains("active=true"));
}

#[test]
fn parses_project_archive_and_restore_commands() {
    let archive = Cli::try_parse_from([
        "camera-connector",
        "project",
        "--config",
        "C:\\CameraConnector\\config.json",
        "archive",
        "--id",
        "project-1",
    ])
    .expect("project archive command should parse");
    let restore = Cli::try_parse_from([
        "camera-connector",
        "project",
        "--config",
        "C:\\CameraConnector\\config.json",
        "restore",
        "--id",
        "project-1",
    ])
    .expect("project restore command should parse");

    assert!(matches!(
        archive.command,
        Some(Command::Project {
            action: ProjectCommand::Archive { id },
            ..
        }) if id == "project-1"
    ));
    assert!(matches!(
        restore.command,
        Some(Command::Project {
            action: ProjectCommand::Restore { id },
            ..
        }) if id == "project-1"
    ));
}

#[test]
fn parses_project_rename_command() {
    let cli = Cli::try_parse_from([
        "camera-connector",
        "project",
        "rename",
        "--id",
        "project-1",
        "--name",
        "Client Shoot",
    ])
    .expect("project rename command should parse");

    assert!(matches!(
        cli.command,
        Some(Command::Project {
            action: ProjectCommand::Rename { id, name },
            ..
        }) if id == "project-1" && name == "Client Shoot"
    ));
}

#[test]
fn project_rename_command_updates_project_name() {
    let path = unique_temp_config_path("project-rename");
    handle_project_command(
        Some(&path),
        ProjectCommand::Create {
            name: "Untitled Shoot".to_string(),
        },
    )
    .expect("project should create");
    let service = CameraConnectorService::new(Some(path.clone()));
    let project = service
        .active_project()
        .expect("active project should load")
        .expect("active project should exist");

    handle_project_command(
        Some(&path),
        ProjectCommand::Rename {
            id: project.project_id.clone(),
            name: "Client Shoot".to_string(),
        },
    )
    .expect("project should rename");

    let active = service
        .active_project()
        .expect("active project should load")
        .expect("active project should exist");
    assert_eq!(active.project_id, project.project_id);
    assert_eq!(active.name, "Client Shoot");
    assert_eq!(active.slug, "client-shoot");

    let parent = path.parent().map(Path::to_path_buf);
    let _ = std::fs::remove_file(path);
    if let Some(parent) = parent {
        let _ = std::fs::remove_dir_all(parent);
    }
}

#[test]
fn project_archive_command_clears_active_project() {
    let path = unique_temp_config_path("project-archive");

    handle_project_command(
        Some(&path),
        ProjectCommand::Create {
            name: "Archive Me".to_string(),
        },
    )
    .expect("project should create");
    let service = CameraConnectorService::new(Some(path.clone()));
    let project = service
        .active_project()
        .expect("active project should load")
        .expect("active project should exist");

    handle_project_command(
        Some(&path),
        ProjectCommand::Archive {
            id: project.project_id.clone(),
        },
    )
    .expect("project should archive");

    assert!(service
        .active_project()
        .expect("active project should load")
        .is_none());
    assert!(service.set_active_project(&project.project_id).is_err());

    handle_project_command(
        Some(&path),
        ProjectCommand::Restore {
            id: project.project_id.clone(),
        },
    )
    .expect("project should restore");
    service
        .set_active_project(&project.project_id)
        .expect("restored project should be selectable");

    let _ = std::fs::remove_file(path);
}
