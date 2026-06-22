use camera_connector_core::{ImporterError, SavePromptPackRequest, SceneProfile};
use serde_json::json;

use super::json_support::{prompt_pack_json_value, prompt_pack_json_value_with_text};
use super::{MobileCore, MobileCoreResult};

impl MobileCore {
    pub fn prompt_packs_for_project_json(&self, project_id: String) -> MobileCoreResult<String> {
        let profiles = self.service.prompt_packs_for_project(&project_id)?;
        let values = profiles
            .iter()
            .map(prompt_pack_json_value)
            .collect::<Vec<_>>();
        Ok(serde_json::to_string(&values)?)
    }

    pub fn global_prompt_packs_json(&self) -> MobileCoreResult<String> {
        let profiles = self.service.global_prompt_packs()?;
        let mut values = Vec::with_capacity(profiles.len());
        for profile in &profiles {
            values.push(prompt_pack_json_value_with_text(
                profile,
                self.service
                    .prompt_markdown_for_pack(&profile.prompt_pack_id)?,
            ));
        }
        Ok(serde_json::to_string(&values)?)
    }

    pub fn fork_global_prompt_pack_json(
        &self,
        source_profile_id: String,
        name: String,
        distribution_folder: String,
    ) -> MobileCoreResult<String> {
        let profile = self.service.fork_global_prompt_pack(
            &source_profile_id,
            name,
            distribution_folder,
            self.next_action_time_ms(),
        )?;
        Ok(serde_json::to_string(&prompt_pack_json_value_with_text(
            &profile,
            self.service
                .prompt_markdown_for_pack(&profile.prompt_pack_id)?,
        ))?)
    }

    pub fn create_global_prompt_pack_json(
        &self,
        name: String,
        style_tags_json: String,
        scene_profile: String,
        distribution_folder: String,
        prompt_text: String,
    ) -> MobileCoreResult<String> {
        let style_tags = serde_json::from_str::<Vec<String>>(&style_tags_json)
            .map_err(|error| ImporterError::internal(format!("invalid style tags: {error}")))?;
        let profile = self.service.create_global_prompt_pack(
            name,
            style_tags,
            SceneProfile::from_str(&scene_profile),
            distribution_folder,
            prompt_text,
            self.next_action_time_ms(),
        )?;
        Ok(serde_json::to_string(&prompt_pack_json_value_with_text(
            &profile,
            self.service
                .prompt_markdown_for_pack(&profile.prompt_pack_id)?,
        ))?)
    }

    pub fn save_global_prompt_pack_json(
        &self,
        prompt_pack_id: String,
        name: String,
        style_tags_json: String,
        scene_profile: String,
        prompt_text: String,
    ) -> MobileCoreResult<String> {
        let style_tags = serde_json::from_str::<Vec<String>>(&style_tags_json)
            .map_err(|error| ImporterError::internal(format!("invalid style tags: {error}")))?;
        let profile = self.service.save_global_prompt_pack(
            &prompt_pack_id,
            name,
            style_tags,
            SceneProfile::from_str(&scene_profile),
            prompt_text,
            self.next_action_time_ms(),
        )?;
        Ok(serde_json::to_string(&prompt_pack_json_value_with_text(
            &profile,
            self.service
                .prompt_markdown_for_pack(&profile.prompt_pack_id)?,
        ))?)
    }

    pub fn delete_global_prompt_pack_json(
        &self,
        prompt_pack_id: String,
    ) -> MobileCoreResult<String> {
        let deleted = self.service.delete_global_prompt_pack(&prompt_pack_id)?;
        Ok(serde_json::to_string(&json!({
            "prompt_pack_id": prompt_pack_id,
            "deleted": deleted,
        }))?)
    }

    pub fn delete_global_prompt_package_json(
        &self,
        distribution_folder: String,
    ) -> MobileCoreResult<String> {
        let deleted = self
            .service
            .delete_global_prompt_package(&distribution_folder)?;
        Ok(serde_json::to_string(&json!({
            "distribution_folder": distribution_folder,
            "deleted": deleted,
        }))?)
    }

    pub fn fork_prompt_pack_json(
        &self,
        project_id: String,
        source_profile_id: String,
        name: String,
        distribution_folder: String,
    ) -> MobileCoreResult<String> {
        let profile = self.service.fork_prompt_pack_for_project(
            &project_id,
            &source_profile_id,
            name,
            distribution_folder,
            self.next_action_time_ms(),
        )?;
        Ok(serde_json::to_string(&prompt_pack_json_value(&profile))?)
    }

    pub fn save_prompt_pack_json(
        &self,
        project_id: String,
        prompt_pack_id: String,
        name: String,
        style_tags_json: String,
        scene_profile: String,
        prompt_text: String,
    ) -> MobileCoreResult<String> {
        let style_tags = serde_json::from_str::<Vec<String>>(&style_tags_json)
            .map_err(|error| ImporterError::internal(format!("invalid style tags: {error}")))?;
        let profile = self.service.save_prompt_pack(SavePromptPackRequest {
            project_id,
            prompt_pack_id,
            name,
            style_tags,
            scene_profile: SceneProfile::from_str(&scene_profile),
            prompt_text,
            now_ms: self.next_action_time_ms(),
        })?;
        Ok(serde_json::to_string(&prompt_pack_json_value_with_text(
            &profile,
            self.service
                .prompt_markdown_for_pack(&profile.prompt_pack_id)?,
        ))?)
    }
}
