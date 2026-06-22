use super::*;

#[test]
fn guest_mark_accepts_only_share_selection_values() {
    assert_eq!(
        GuestMark::from_wire("favorite").unwrap(),
        GuestMark::Favorite
    );
    assert_eq!(GuestMark::from_wire("marked").unwrap(), GuestMark::Marked);
    assert_eq!(GuestMark::from_wire("reject").unwrap(), GuestMark::Reject);
    assert!(GuestMark::from_wire("delete").is_none());
    assert!(GuestMark::from_wire("").is_none());
}

#[test]
fn service_lan_share_asset_page_uses_saved_query() {
    let (service, config_path, state_dir) = service_with_state_dir("service-lan-share-query");
    let project = service
        .create_project("LAN Query")
        .expect("project should create");
    let keep = record_service_jpeg_group(&service, &project.project_id, "KEEP_0001.JPG", 10);
    let skip = record_service_jpeg_group(&service, &project.project_id, "SKIP_0001.JPG", 20);
    service
        .set_asset_group_user_marks(&project.project_id, &keep, Some(true), None)
        .expect("favorite should save");

    let session = service
        .create_lan_share_session(
            &project.project_id,
            AssetGroupQuery {
                favorite: Some(true),
                ..AssetGroupQuery::default()
            },
            Some("Favorites".to_string()),
        )
        .expect("share session should create");

    let page = service
        .lan_share_asset_group_page(&session.token, 0, 25)
        .expect("share page should load");

    assert_eq!(page.total_groups, 1);
    assert_eq!(page.groups[0].group_id.as_deref(), Some(keep.as_str()));
    assert_ne!(page.groups[0].group_id.as_deref(), Some(skip.as_str()));

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(state_dir);
}

#[test]
fn service_lan_share_user_mark_any_matches_favorite_or_marked() {
    let (service, config_path, state_dir) = service_with_state_dir("service-lan-share-mark-any");
    let project = service
        .create_project("LAN Mark Any")
        .expect("project should create");
    let favorite =
        record_service_jpeg_group(&service, &project.project_id, "FAVORITE_0001.JPG", 10);
    let marked = record_service_jpeg_group(&service, &project.project_id, "MARKED_0001.JPG", 20);
    let neither = record_service_jpeg_group(&service, &project.project_id, "NEITHER_0001.JPG", 30);
    service
        .set_asset_group_user_marks(&project.project_id, &favorite, Some(true), None)
        .expect("favorite should save");
    service
        .set_asset_group_user_marks(&project.project_id, &marked, None, Some(true))
        .expect("marked should save");

    let session = service
        .create_lan_share_session(
            &project.project_id,
            AssetGroupQuery {
                user_mark_any: vec!["favorite".to_string(), "marked".to_string()],
                ..AssetGroupQuery::default()
            },
            Some("Favorite or marked".to_string()),
        )
        .expect("share session should create");

    let page = service
        .lan_share_asset_group_page(&session.token, 0, 25)
        .expect("share page should load");
    let group_ids = page
        .groups
        .iter()
        .filter_map(|group| group.group_id.as_deref())
        .collect::<Vec<_>>();

    assert_eq!(page.total_groups, 2);
    assert!(group_ids.contains(&favorite.as_str()));
    assert!(group_ids.contains(&marked.as_str()));
    assert!(!group_ids.contains(&neither.as_str()));

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(state_dir);
}

#[test]
fn service_guest_reject_mark_does_not_delete_or_mutate_user_marks() {
    let (service, config_path, state_dir) = service_with_state_dir("service-lan-share-reject");
    let project = service
        .create_project("LAN Reject")
        .expect("project should create");
    let group_id = record_service_jpeg_group(&service, &project.project_id, "IMG_4040.JPG", 10);
    let session = service
        .create_lan_share_session(&project.project_id, AssetGroupQuery::default(), None)
        .expect("share session should create");

    service
        .set_lan_share_guest_mark(&session.token, &group_id, Some(GuestMark::Reject))
        .expect("guest mark should save");

    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("page should load");
    assert_eq!(page.total_groups, 1);
    assert_eq!(page.groups[0].guest_mark, Some(GuestMark::Reject));
    assert!(!page.groups[0].user_marks.favorite);
    assert!(!page.groups[0].user_marks.marked);

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(state_dir);
}

#[test]
fn service_asset_group_query_filters_guest_marks() {
    let (service, config_path, state_dir) = service_with_state_dir("service-guest-mark-query");
    let project = service
        .create_project("Guest Mark Query")
        .expect("project should create");
    let rejected = record_service_jpeg_group(&service, &project.project_id, "REJECT_0001.JPG", 10);
    let unmarked =
        record_service_jpeg_group(&service, &project.project_id, "UNMARKED_0001.JPG", 20);
    let session = service
        .create_lan_share_session(&project.project_id, AssetGroupQuery::default(), None)
        .expect("share session should create");

    service
        .set_lan_share_guest_mark(&session.token, &rejected, Some(GuestMark::Reject))
        .expect("guest mark should save");

    let reject_page = service
        .project_asset_group_page_with_query(
            &project.project_id,
            AssetGroupQuery {
                guest_mark: Some("reject".to_string()),
                ..AssetGroupQuery::default()
            },
            0,
            25,
        )
        .expect("reject page should load");
    assert_eq!(reject_page.total_groups, 1);
    assert_eq!(
        reject_page.groups[0].group_id.as_deref(),
        Some(rejected.as_str())
    );

    let unmarked_page = service
        .project_asset_group_page_with_query(
            &project.project_id,
            AssetGroupQuery {
                guest_mark: Some("none".to_string()),
                ..AssetGroupQuery::default()
            },
            0,
            25,
        )
        .expect("unmarked page should load");
    assert_eq!(unmarked_page.total_groups, 1);
    assert_eq!(
        unmarked_page.groups[0].group_id.as_deref(),
        Some(unmarked.as_str())
    );

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(state_dir);
}
