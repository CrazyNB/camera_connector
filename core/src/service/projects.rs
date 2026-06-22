use std::fs;

use crate::{ProjectStatus, Result, StoredObjectLocation};

use super::CameraConnectorService;

impl CameraConnectorService {
    pub fn create_project(&self, name: impl AsRef<str>) -> Result<crate::Project> {
        self.storage_store()?.create_project(name)
    }

    pub fn rename_project(
        &self,
        project_id: &str,
        name: impl AsRef<str>,
    ) -> Result<crate::Project> {
        self.storage_store()?.rename_project(project_id, name)
    }

    pub fn archive_project(&self, project_id: &str) -> Result<crate::Project> {
        let archived = self.storage_store()?.archive_project(project_id)?;
        let mut config = self.load_config()?;
        if config.active_project_id.as_deref() == Some(project_id) {
            config.active_project_id = None;
            self.save_config(&config)?;
        }
        Ok(archived)
    }

    pub fn delete_project(&self, project_id: &str) -> Result<bool> {
        let store = self.storage_store()?;
        let deleted_assets = store.delete_project(project_id)?;
        let Some(deleted_assets) = deleted_assets else {
            return Ok(false);
        };
        for asset in &deleted_assets {
            if let Some(path) = asset
                .final_location
                .as_ref()
                .and_then(StoredObjectLocation::as_local_path)
            {
                match fs::remove_file(path) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => {
                        return Err(crate::ImporterError::internal(format!(
                            "delete asset file failed: {error}"
                        )));
                    }
                }
            }
        }

        let mut config = self.load_config()?;
        if config.active_project_id.as_deref() == Some(project_id) {
            config.active_project_id = None;
            self.save_config(&config)?;
        }
        Ok(true)
    }

    pub fn restore_project(&self, project_id: &str) -> Result<crate::Project> {
        self.storage_store()?.restore_project(project_id)
    }

    pub fn set_active_project(&self, project_id: &str) -> Result<()> {
        let project = self
            .storage_store()?
            .list_projects()?
            .into_iter()
            .find(|project| project.project_id == project_id)
            .ok_or_else(|| crate::ImporterError::internal("project not found"))?;
        if project.status != ProjectStatus::Active {
            return Err(crate::ImporterError::internal("project archived"));
        }
        let mut config = self.load_config()?;
        config.active_project_id = Some(project.project_id);
        self.save_config(&config)?;
        Ok(())
    }

    pub fn active_project(&self) -> Result<Option<crate::Project>> {
        let Some(project_id) = self.load_config()?.active_project_id else {
            return Ok(None);
        };
        Ok(self
            .storage_store()?
            .list_projects()?
            .into_iter()
            .find(|project| {
                project.project_id == project_id && project.status == ProjectStatus::Active
            }))
    }

    pub fn list_projects(&self) -> Result<Vec<crate::Project>> {
        self.storage_store()?.list_projects()
    }
}
