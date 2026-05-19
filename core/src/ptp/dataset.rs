use bytes::Buf;

use crate::ImporterError;
use crate::{CameraInfo, CameraObject, ObjectFormat, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfoDataset {
    pub manufacturer: String,
    pub model: String,
    pub serial_number: Option<String>,
    pub firmware_version: Option<String>,
    pub supported_operations: Vec<u16>,
    pub supported_formats: Vec<u16>,
}

impl DeviceInfoDataset {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let mut reader = DatasetReader::new(bytes);

        reader.read_u16()?;
        reader.read_u32()?;
        reader.read_u16()?;
        reader.read_string()?;
        reader.read_u16()?;
        let supported_operations = reader.read_u16_array()?;
        reader.read_u16_array()?;
        reader.read_u16_array()?;
        reader.read_u16_array()?;
        let supported_formats = reader.read_u16_array()?;
        let manufacturer = reader.read_string()?;
        let model = reader.read_string()?;
        let firmware_version = none_if_empty(reader.read_string()?);
        let serial_number = none_if_empty(reader.read_string()?);

        Ok(Self {
            manufacturer,
            model,
            serial_number,
            firmware_version,
            supported_operations,
            supported_formats,
        })
    }
}

impl From<DeviceInfoDataset> for CameraInfo {
    fn from(value: DeviceInfoDataset) -> Self {
        Self {
            manufacturer: value.manufacturer,
            model: value.model,
            serial_number: value.serial_number,
            firmware_version: value.firmware_version,
            supported_operations: value.supported_operations,
            supported_formats: value.supported_formats,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageInfoDataset {
    pub storage_type: u16,
    pub filesystem_type: u16,
    pub access_capability: u16,
    pub max_capacity_bytes: u64,
    pub free_space_bytes: u64,
    pub free_space_images: u32,
    pub storage_description: String,
    pub volume_label: String,
}

impl StorageInfoDataset {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let mut reader = DatasetReader::new(bytes);

        Ok(Self {
            storage_type: reader.read_u16()?,
            filesystem_type: reader.read_u16()?,
            access_capability: reader.read_u16()?,
            max_capacity_bytes: reader.read_u64()?,
            free_space_bytes: reader.read_u64()?,
            free_space_images: reader.read_u32()?,
            storage_description: reader.read_string()?,
            volume_label: reader.read_string()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectInfoDataset {
    pub storage_id: u32,
    pub filename: String,
    pub size_bytes: u64,
    pub format: ObjectFormat,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub thumb_available: bool,
}

impl ObjectInfoDataset {
    pub fn parse(handle: u32, bytes: &[u8]) -> Result<CameraObject> {
        let mut reader = DatasetReader::new(bytes);

        let storage_id = reader.read_u32()?;
        let object_format_code = reader.read_u16()?;
        reader.read_u16()?;
        let size_bytes = reader.read_u32()? as u64;
        reader.read_u16()?;
        let thumb_size = reader.read_u32()?;
        reader.read_u32()?;
        reader.read_u32()?;
        let width = zero_as_none(reader.read_u32()?);
        let height = zero_as_none(reader.read_u32()?);
        reader.read_u32()?;
        reader.read_u32()?;
        reader.read_u16()?;
        reader.read_u32()?;
        reader.read_u32()?;
        let filename = reader.read_string()?;
        reader.read_string()?;
        reader.read_string()?;
        let _ = reader.read_string();

        let mut object = CameraObject::new(handle, storage_id, filename, size_bytes);
        object.format = object_format_from_code(object_format_code, &object.filename);
        object.width = width;
        object.height = height;
        object.thumb_available = thumb_size > 0;
        Ok(object)
    }
}

fn object_format_from_code(code: u16, filename: &str) -> ObjectFormat {
    match code {
        0x3801 => ObjectFormat::Jpeg,
        0x300A => ObjectFormat::Tiff,
        0x300D => ObjectFormat::Mov,
        _ => ObjectFormat::from_filename(filename),
    }
}

fn none_if_empty(value: String) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn zero_as_none(value: u32) -> Option<u32> {
    if value == 0 {
        None
    } else {
        Some(value)
    }
}

struct DatasetReader<'a> {
    bytes: &'a [u8],
}

impl<'a> DatasetReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    fn read_u16(&mut self) -> Result<u16> {
        if self.bytes.remaining() < 2 {
            return Err(ImporterError::UnknownCameraResponse);
        }
        Ok(self.bytes.get_u16_le())
    }

    fn read_u32(&mut self) -> Result<u32> {
        if self.bytes.remaining() < 4 {
            return Err(ImporterError::UnknownCameraResponse);
        }
        Ok(self.bytes.get_u32_le())
    }

    fn read_u64(&mut self) -> Result<u64> {
        if self.bytes.remaining() < 8 {
            return Err(ImporterError::UnknownCameraResponse);
        }
        Ok(self.bytes.get_u64_le())
    }

    fn read_string(&mut self) -> Result<String> {
        if self.bytes.remaining() < 1 {
            return Err(ImporterError::UnknownCameraResponse);
        }

        let char_count = self.bytes.get_u8() as usize;
        if char_count == 0 {
            return Ok(String::new());
        }

        let byte_count = char_count
            .checked_mul(2)
            .ok_or_else(|| ImporterError::internal("ptp string length overflow"))?;
        if self.bytes.remaining() < byte_count {
            return Err(ImporterError::UnknownCameraResponse);
        }

        let mut chars = Vec::with_capacity(char_count);
        for _ in 0..char_count {
            chars.push(self.bytes.get_u16_le());
        }

        while chars.last() == Some(&0) {
            chars.pop();
        }

        String::from_utf16(&chars).map_err(|_| ImporterError::UnknownCameraResponse)
    }

    fn read_u16_array(&mut self) -> Result<Vec<u16>> {
        let count = self.read_u32()? as usize;
        let byte_count = count
            .checked_mul(2)
            .ok_or_else(|| ImporterError::internal("ptp array length overflow"))?;
        if self.bytes.remaining() < byte_count {
            return Err(ImporterError::UnknownCameraResponse);
        }

        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push(self.bytes.get_u16_le());
        }
        Ok(values)
    }
}
