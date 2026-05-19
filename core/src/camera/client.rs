use bytes::Buf;

use crate::ptp::{DeviceInfoDataset, ObjectInfoDataset, PtpOperationCode, PtpSession};
use crate::ptp_ip::PtpIpTransport;
use crate::{CameraEndpoint, CameraInfo, CameraObject, ImporterError, Result};

use super::CameraCapability;

pub struct NikonCameraClient {
    session: PtpSession,
    camera_info: Option<CameraInfo>,
}

impl NikonCameraClient {
    pub async fn connect(endpoint: CameraEndpoint) -> Result<Self> {
        let mut transport = PtpIpTransport::connect(endpoint).await?;
        transport.init_command().await?;
        transport.init_event().await?;

        let mut session = PtpSession::new(transport);
        session.open().await?;

        Ok(Self {
            session,
            camera_info: None,
        })
    }

    pub async fn get_camera_info(&mut self) -> Result<CameraInfo> {
        let result = self
            .session
            .operation(PtpOperationCode::GetDeviceInfo, Vec::new())
            .await?;
        let info: CameraInfo = DeviceInfoDataset::parse(&result.data)?.into();
        self.camera_info = Some(info.clone());
        Ok(info)
    }

    pub async fn probe_capabilities(&mut self) -> Result<CameraCapability> {
        let info = match &self.camera_info {
            Some(info) => info.clone(),
            None => self.get_camera_info().await?,
        };
        Ok(CameraCapability::from_camera_info(&info))
    }

    pub async fn list_objects(&mut self) -> Result<Vec<CameraObject>> {
        let storage_ids = self.get_storage_ids().await?;
        let mut objects = Vec::new();

        for storage_id in storage_ids {
            let handles = self.get_object_handles(storage_id).await?;
            for handle in handles {
                let object = self.get_object_info(handle).await?;
                objects.push(object);
            }
        }

        objects.sort_by(|left, right| {
            right
                .capture_time_ms
                .cmp(&left.capture_time_ms)
                .then_with(|| left.filename.cmp(&right.filename))
        });
        Ok(objects)
    }

    pub async fn get_thumbnail(&mut self, handle: u32) -> Result<Vec<u8>> {
        let result = self
            .session
            .operation(PtpOperationCode::GetThumb, vec![handle])
            .await?;
        if result.data.is_empty() {
            return Err(ImporterError::ThumbnailUnavailable);
        }
        Ok(result.data)
    }

    pub async fn get_object(&mut self, handle: u32) -> Result<Vec<u8>> {
        let result = self
            .session
            .operation(PtpOperationCode::GetObject, vec![handle])
            .await?;
        Ok(result.data)
    }

    pub async fn close(&mut self) -> Result<()> {
        self.session.close().await
    }

    async fn get_storage_ids(&mut self) -> Result<Vec<u32>> {
        let result = self
            .session
            .operation(PtpOperationCode::GetStorageIds, Vec::new())
            .await?;
        parse_u32_array(&result.data)
    }

    async fn get_object_handles(&mut self, storage_id: u32) -> Result<Vec<u32>> {
        let result = self
            .session
            .operation(PtpOperationCode::GetObjectHandles, vec![storage_id, 0, 0])
            .await?;
        parse_u32_array(&result.data)
    }

    async fn get_object_info(&mut self, handle: u32) -> Result<CameraObject> {
        let result = self
            .session
            .operation(PtpOperationCode::GetObjectInfo, vec![handle])
            .await?;
        ObjectInfoDataset::parse(handle, &result.data)
    }
}

fn parse_u32_array(bytes: &[u8]) -> Result<Vec<u32>> {
    if bytes.len() < 4 {
        return Err(ImporterError::UnknownCameraResponse);
    }

    let mut reader = bytes;
    let count = reader.get_u32_le() as usize;
    if reader.remaining() < count * 4 {
        return Err(ImporterError::UnknownCameraResponse);
    }

    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(reader.get_u32_le());
    }
    Ok(values)
}
