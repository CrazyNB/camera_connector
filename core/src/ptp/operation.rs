use bytes::{BufMut, BytesMut};

use crate::{ImporterError, Result};

pub const DATA_PHASE_NONE: u32 = 0x0000_0000;
pub const DATA_PHASE_IN: u32 = 0x0000_0001;
pub const DATA_PHASE_OUT: u32 = 0x0000_0002;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum PtpOperationCode {
    GetDeviceInfo = 0x1001,
    OpenSession = 0x1002,
    CloseSession = 0x1003,
    GetStorageIds = 0x1004,
    GetStorageInfo = 0x1005,
    GetObjectHandles = 0x1007,
    GetObjectInfo = 0x1008,
    GetObject = 0x1009,
    GetThumb = 0x100A,
}

impl PtpOperationCode {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x1001 => Some(Self::GetDeviceInfo),
            0x1002 => Some(Self::OpenSession),
            0x1003 => Some(Self::CloseSession),
            0x1004 => Some(Self::GetStorageIds),
            0x1005 => Some(Self::GetStorageInfo),
            0x1007 => Some(Self::GetObjectHandles),
            0x1008 => Some(Self::GetObjectInfo),
            0x1009 => Some(Self::GetObject),
            0x100A => Some(Self::GetThumb),
            _ => None,
        }
    }

    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtpOperation {
    pub code: PtpOperationCode,
    pub transaction_id: u32,
    pub params: Vec<u32>,
    pub data_phase: u32,
}

impl PtpOperation {
    pub fn new(code: PtpOperationCode, transaction_id: u32, params: Vec<u32>) -> Self {
        Self {
            code,
            transaction_id,
            params,
            data_phase: DATA_PHASE_IN,
        }
    }

    pub fn with_data_phase(mut self, data_phase: u32) -> Self {
        self.data_phase = data_phase;
        self
    }

    pub fn encode_request_payload(&self) -> Result<Vec<u8>> {
        if self.params.len() > 5 {
            return Err(ImporterError::internal(
                "ptp operation supports at most five parameters",
            ));
        }

        let mut out = BytesMut::with_capacity(10 + self.params.len() * 4);
        out.put_u32_le(self.data_phase);
        out.put_u16_le(self.code.as_u16());
        out.put_u32_le(self.transaction_id);
        for param in &self.params {
            out.put_u32_le(*param);
        }
        Ok(out.to_vec())
    }
}
