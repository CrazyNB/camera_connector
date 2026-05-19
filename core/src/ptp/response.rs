use bytes::Buf;

use crate::{ImporterError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum PtpResponseCode {
    Ok = 0x2001,
    GeneralError = 0x2002,
    SessionNotOpen = 0x2003,
    InvalidTransactionId = 0x2004,
    OperationNotSupported = 0x2005,
    ParameterNotSupported = 0x2006,
    IncompleteTransfer = 0x2007,
    InvalidStorageId = 0x2008,
    InvalidObjectHandle = 0x2009,
    DeviceBusy = 0x2019,
    Unknown(u16),
}

impl PtpResponseCode {
    pub fn from_u16(value: u16) -> Self {
        match value {
            0x2001 => Self::Ok,
            0x2002 => Self::GeneralError,
            0x2003 => Self::SessionNotOpen,
            0x2004 => Self::InvalidTransactionId,
            0x2005 => Self::OperationNotSupported,
            0x2006 => Self::ParameterNotSupported,
            0x2007 => Self::IncompleteTransfer,
            0x2008 => Self::InvalidStorageId,
            0x2009 => Self::InvalidObjectHandle,
            0x2019 => Self::DeviceBusy,
            other => Self::Unknown(other),
        }
    }

    pub fn as_u16(self) -> u16 {
        match self {
            Self::Ok => 0x2001,
            Self::GeneralError => 0x2002,
            Self::SessionNotOpen => 0x2003,
            Self::InvalidTransactionId => 0x2004,
            Self::OperationNotSupported => 0x2005,
            Self::ParameterNotSupported => 0x2006,
            Self::IncompleteTransfer => 0x2007,
            Self::InvalidStorageId => 0x2008,
            Self::InvalidObjectHandle => 0x2009,
            Self::DeviceBusy => 0x2019,
            Self::Unknown(value) => value,
        }
    }

    pub fn into_result(self) -> Result<()> {
        match self {
            Self::Ok => Ok(()),
            Self::OperationNotSupported | Self::ParameterNotSupported => {
                Err(ImporterError::UnsupportedOperation)
            }
            Self::InvalidObjectHandle => Err(ImporterError::ObjectNotFound),
            Self::IncompleteTransfer => Err(ImporterError::DownloadInterrupted),
            _ => Err(ImporterError::UnknownCameraResponse),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtpResponse {
    pub code: PtpResponseCode,
    pub transaction_id: u32,
    pub params: Vec<u32>,
}

impl PtpResponse {
    pub fn decode_payload(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 6 || !(bytes.len() - 6).is_multiple_of(4) {
            return Err(ImporterError::UnknownCameraResponse);
        }

        let mut reader = bytes;
        let code = PtpResponseCode::from_u16(reader.get_u16_le());
        let transaction_id = reader.get_u32_le();
        let mut params = Vec::with_capacity(reader.remaining() / 4);

        while reader.has_remaining() {
            params.push(reader.get_u32_le());
        }

        Ok(Self {
            code,
            transaction_id,
            params,
        })
    }
}
