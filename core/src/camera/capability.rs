use crate::CameraInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraCapability {
    pub supports_get_thumb: bool,
    pub supports_get_object: bool,
    pub supports_storage_info: bool,
    pub supports_raw_download: bool,
    pub supports_video_download: bool,
}

impl CameraCapability {
    pub fn from_camera_info(info: &CameraInfo) -> Self {
        let has_operation = |code| info.supported_operations.contains(&code);
        let has_format = |code| info.supported_formats.contains(&code);

        Self {
            supports_get_thumb: has_operation(0x100A),
            supports_get_object: has_operation(0x1009),
            supports_storage_info: has_operation(0x1005),
            supports_raw_download: has_operation(0x1009) && has_format(0xB103),
            supports_video_download: has_operation(0x1009),
        }
    }
}
