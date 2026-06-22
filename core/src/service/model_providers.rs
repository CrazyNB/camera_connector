use crate::{
    CameraConnectorConfig, ModelProviderKind, ModelProviderSettings, ModelProviderSettingsConfig,
    ModelSendMode,
};

use super::current_time_ms;

#[derive(Debug, Clone)]
pub(super) struct RuntimeModelProvider {
    pub(super) settings: ModelProviderSettings,
    pub(super) api_key: Option<String>,
}

pub(super) fn model_provider_settings_from_config(
    config: ModelProviderSettingsConfig,
) -> ModelProviderSettings {
    let api_key_configured = config
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some()
        || config
            .key_alias
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_some();
    ModelProviderSettings {
        settings_id: normalized_model_provider_settings_id(&config.settings_id),
        provider_kind: ModelProviderKind::from_str(config.provider_kind.trim()),
        provider_label: config.provider_label,
        base_url: config.base_url,
        default_model: config.default_model,
        default_max_image_side: if config.default_max_image_side > 0 {
            config.default_max_image_side
        } else {
            1024
        },
        default_send_mode: ModelSendMode::from_str(config.default_send_mode.trim()),
        default_batch_size: config.default_batch_size.max(1),
        configured: config.configured,
        api_key_configured,
        key_alias: config.key_alias,
        updated_at_ms: config.updated_at_ms,
    }
}

pub(super) fn model_provider_settings_to_config(
    settings: ModelProviderSettings,
    api_key: Option<String>,
) -> ModelProviderSettingsConfig {
    let api_key = api_key.and_then(|value| {
        let value = value.trim().to_string();
        (!value.is_empty()).then_some(value)
    });
    ModelProviderSettingsConfig {
        settings_id: normalized_model_provider_settings_id(&settings.settings_id),
        provider_kind: settings.provider_kind.as_str().to_string(),
        provider_label: settings.provider_label,
        base_url: settings.base_url,
        default_model: settings.default_model,
        default_max_image_side: settings.default_max_image_side.max(1),
        default_send_mode: settings.default_send_mode.as_str().to_string(),
        default_batch_size: settings.default_batch_size.max(1),
        configured: settings.configured,
        api_key,
        key_alias: settings.key_alias,
        updated_at_ms: if settings.updated_at_ms == 0 {
            current_time_ms()
        } else {
            settings.updated_at_ms
        },
    }
}

pub(super) fn runtime_model_providers_from_config(
    config: CameraConnectorConfig,
) -> Vec<RuntimeModelProvider> {
    config
        .model_providers
        .into_iter()
        .filter(|provider| !model_provider_config_is_empty(provider))
        .map(|provider| RuntimeModelProvider {
            api_key: provider.api_key.clone(),
            settings: model_provider_settings_from_config(provider),
        })
        .collect()
}

pub(super) fn model_provider_config_by_id<'a>(
    config: &'a CameraConnectorConfig,
    settings_id: &str,
) -> Option<&'a ModelProviderSettingsConfig> {
    let settings_id = normalized_model_provider_settings_id(settings_id);
    config.model_providers.iter().find(|provider| {
        normalized_model_provider_settings_id(&provider.settings_id) == settings_id
    })
}

pub(super) fn upsert_model_provider_config(
    config: &mut CameraConnectorConfig,
    provider: ModelProviderSettingsConfig,
) {
    let settings_id = normalized_model_provider_settings_id(&provider.settings_id);
    if let Some(existing) = config.model_providers.iter_mut().find(|existing| {
        normalized_model_provider_settings_id(&existing.settings_id) == settings_id
    }) {
        *existing = provider;
    } else {
        config.model_providers.push(provider);
    }
}

fn model_provider_config_is_empty(provider: &ModelProviderSettingsConfig) -> bool {
    let provider_kind = provider.provider_kind.trim();
    (provider_kind.is_empty() || provider_kind == "none")
        && provider.default_model.trim().is_empty()
        && provider.base_url.trim().is_empty()
        && !provider.configured
}

pub(super) fn normalized_model_provider_settings_id(settings_id: &str) -> String {
    let settings_id = settings_id.trim();
    if settings_id.is_empty() {
        "global".to_string()
    } else {
        settings_id.to_string()
    }
}
