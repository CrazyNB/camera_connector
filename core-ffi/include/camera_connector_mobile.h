#ifndef CAMERA_CONNECTOR_MOBILE_H
#define CAMERA_CONNECTOR_MOBILE_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct CameraConnectorMobileCore CameraConnectorMobileCore;

/*
 * Camera Connector mobile FFI contract.
 *
 * All returned char* values are heap allocated by Rust and must be released
 * with camera_connector_mobile_core_free_string.
 *
 * String-returning calls use a JSON envelope:
 *   {"ok":true,"value":...,"error":null}
 *   {"ok":false,"value":null,"error":"message"}
 *
 * All input strings must be null-terminated UTF-8. Optional strings may be null.
 */

CameraConnectorMobileCore *camera_connector_mobile_core_create(const char *config_path);

void camera_connector_mobile_core_destroy(CameraConnectorMobileCore *core);

void camera_connector_mobile_core_free_string(char *value);

char *camera_connector_mobile_core_config_path(const CameraConnectorMobileCore *core);

char *camera_connector_mobile_core_default_state_dir(const CameraConnectorMobileCore *core);

char *camera_connector_mobile_core_create_project_json(
    const CameraConnectorMobileCore *core,
    const char *name
);

char *camera_connector_mobile_core_list_projects_json(const CameraConnectorMobileCore *core);

char *camera_connector_mobile_core_set_active_project_json(
    const CameraConnectorMobileCore *core,
    const char *project_id
);

char *camera_connector_mobile_core_rename_project_json(
    const CameraConnectorMobileCore *core,
    const char *project_id,
    const char *name
);

char *camera_connector_mobile_core_archive_project_json(
    const CameraConnectorMobileCore *core,
    const char *project_id
);

char *camera_connector_mobile_core_restore_project_json(
    const CameraConnectorMobileCore *core,
    const char *project_id
);

char *camera_connector_mobile_core_active_project_json(const CameraConnectorMobileCore *core);

char *camera_connector_mobile_core_project_dashboard_json(
    const CameraConnectorMobileCore *core,
    const char *project_id,
    uint32_t offset,
    uint32_t limit
);

char *camera_connector_mobile_core_project_asset_group_page_json(
    const CameraConnectorMobileCore *core,
    const char *project_id,
    const char *query_json,
    uint32_t offset,
    uint32_t limit
);

char *camera_connector_mobile_core_project_group_assets_json(
    const CameraConnectorMobileCore *core,
    const char *project_id,
    const char *group_id
);

char *camera_connector_mobile_core_move_project_group_json(
    const CameraConnectorMobileCore *core,
    const char *source_project_id,
    const char *group_id,
    const char *target_project_id
);

char *camera_connector_mobile_core_set_asset_group_user_marks_json(
    const CameraConnectorMobileCore *core,
    const char *project_id,
    const char *group_id,
    const char *patch_json
);

char *camera_connector_mobile_core_claim_next_publish_item_json(
    const CameraConnectorMobileCore *core
);

char *camera_connector_mobile_core_mark_publish_completed_json(
    const CameraConnectorMobileCore *core,
    const char *queue_id
);

char *camera_connector_mobile_core_complete_publish_json(
    const CameraConnectorMobileCore *core,
    const char *queue_id,
    const char *final_filename,
    const char *location_kind,
    const char *location
);

char *camera_connector_mobile_core_mark_publish_failed_json(
    const CameraConnectorMobileCore *core,
    const char *queue_id,
    const char *error
);

char *camera_connector_mobile_core_release_failed_publish_retries_json(
    const CameraConnectorMobileCore *core,
    const char *project_id
);

char *camera_connector_mobile_core_drain_analysis_jobs_json(
    const CameraConnectorMobileCore *core,
    uint32_t limit
);

char *camera_connector_mobile_core_drain_analysis_jobs_with_provider_configured_json(
    const CameraConnectorMobileCore *core,
    uint32_t limit,
    bool provider_configured
);

char *camera_connector_mobile_core_enqueue_model_evaluation_for_asset_groups_json(
    const CameraConnectorMobileCore *core,
    const char *request_json
);

char *camera_connector_mobile_core_evaluate_asset_groups_with_model_inputs_json(
    const CameraConnectorMobileCore *core,
    const char *request_json
);

char *camera_connector_mobile_core_recommend_burst_group_with_candidate_visuals_json(
    const CameraConnectorMobileCore *core,
    const char *request_json
);

char *camera_connector_mobile_core_assess_asset_group_preview_json(
    const CameraConnectorMobileCore *core,
    const char *asset_group_id,
    const char *sample_json,
    const char *assessor_version
);

char *camera_connector_mobile_core_assess_asset_group_preview_with_provider_configured_json(
    const CameraConnectorMobileCore *core,
    const char *asset_group_id,
    const char *sample_json,
    const char *assessor_version,
    bool provider_configured
);

char *camera_connector_mobile_core_split_burst_member_json(
    const CameraConnectorMobileCore *core,
    const char *burst_group_id,
    const char *member_group_id
);

char *camera_connector_mobile_core_merge_burst_member_json(
    const CameraConnectorMobileCore *core,
    const char *target_burst_group_id,
    const char *member_group_id
);

char *camera_connector_mobile_core_model_provider_settings_json(
    const CameraConnectorMobileCore *core
);

char *camera_connector_mobile_core_model_provider_settings_list_json(
    const CameraConnectorMobileCore *core
);

char *camera_connector_mobile_core_save_model_provider_settings_json(
    const CameraConnectorMobileCore *core,
    const char *settings_json
);

char *camera_connector_mobile_core_delete_model_provider_settings_json(
    const CameraConnectorMobileCore *core,
    const char *settings_id
);

char *camera_connector_mobile_core_project_evaluation_settings_json(
    const CameraConnectorMobileCore *core,
    const char *project_id
);

char *camera_connector_mobile_core_save_project_evaluation_settings_json(
    const CameraConnectorMobileCore *core,
    const char *project_id,
    const char *settings_json
);

char *camera_connector_mobile_core_prompt_packs_for_project_json(
    const CameraConnectorMobileCore *core,
    const char *project_id
);

char *camera_connector_mobile_core_global_prompt_packs_json(
    const CameraConnectorMobileCore *core
);

char *camera_connector_mobile_core_create_global_prompt_pack_json(
    const CameraConnectorMobileCore *core,
    const char *name,
    const char *style_tags_json,
    const char *scene_profile,
    const char *distribution_folder,
    const char *prompt_text
);

char *camera_connector_mobile_core_save_global_prompt_pack_json(
    const CameraConnectorMobileCore *core,
    const char *prompt_pack_id,
    const char *name,
    const char *style_tags_json,
    const char *scene_profile,
    const char *prompt_text
);

char *camera_connector_mobile_core_delete_global_prompt_pack_json(
    const CameraConnectorMobileCore *core,
    const char *prompt_pack_id
);

char *camera_connector_mobile_core_delete_global_prompt_package_json(
    const CameraConnectorMobileCore *core,
    const char *distribution_folder
);

char *camera_connector_mobile_core_fork_prompt_pack_json(
    const CameraConnectorMobileCore *core,
    const char *project_id,
    const char *source_pack_id,
    const char *name,
    const char *distribution_folder
);

char *camera_connector_mobile_core_save_prompt_pack_json(
    const CameraConnectorMobileCore *core,
    const char *project_id,
    const char *prompt_pack_id,
    const char *name,
    const char *style_tags_json,
    const char *scene_profile,
    const char *prompt_text
);

char *camera_connector_mobile_core_generate_project_recommendation_json(
    const CameraConnectorMobileCore *core,
    const char *project_id
);

char *camera_connector_mobile_core_generate_project_recommendation_with_candidate_visuals_json(
    const CameraConnectorMobileCore *core,
    const char *request_json
);

char *camera_connector_mobile_core_latest_project_recommendation_run_status_json(
    const CameraConnectorMobileCore *core,
    const char *project_id
);

char *camera_connector_mobile_core_should_schedule_subject_assessment_json(
    const CameraConnectorMobileCore *core,
    const char *project_id
);

char *camera_connector_mobile_core_save_subject_assessment_json(
    const CameraConnectorMobileCore *core,
    const char *assessment_json
);

char *camera_connector_mobile_core_subject_assessments_for_asset_groups_json(
    const CameraConnectorMobileCore *core,
    const char *project_id,
    const char *group_ids_json
);

char *camera_connector_mobile_core_save_receiver_settings_json(
    const CameraConnectorMobileCore *core,
    const char *patch_json
);

char *camera_connector_mobile_core_save_device_account_json(
    const CameraConnectorMobileCore *core,
    const char *username,
    const char *password,
    const char *device_name
);

char *camera_connector_mobile_core_remove_device_account_json(
    const CameraConnectorMobileCore *core,
    const char *username
);

char *camera_connector_mobile_core_start_receiver_json(const CameraConnectorMobileCore *core);

char *camera_connector_mobile_core_stop_receiver_json(const CameraConnectorMobileCore *core);

#ifdef __cplusplus
}
#endif

#endif
