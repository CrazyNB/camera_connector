use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CameraInfo {
    pub manufacturer: String,
    pub model: String,
    pub serial_number: Option<String>,
    pub firmware_version: Option<String>,
    pub supported_operations: Vec<u16>,
    pub supported_formats: Vec<u16>,
}

impl CameraInfo {
    pub fn new(manufacturer: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            manufacturer: manufacturer.into(),
            model: model.into(),
            serial_number: None,
            firmware_version: None,
            supported_operations: Vec::new(),
            supported_formats: Vec::new(),
        }
    }
}
