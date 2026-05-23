#ifndef CAMERA_CONNECTOR_MOBILE_H
#define CAMERA_CONNECTOR_MOBILE_H

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

char *camera_connector_mobile_core_dashboard_json(
    const CameraConnectorMobileCore *core,
    const char *state_dir,
    uint32_t offset,
    uint32_t limit
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
