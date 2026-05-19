use bytes::{Buf, BufMut, BytesMut};

use crate::{ImporterError, Result};

pub const PTP_IP_HEADER_LEN: usize = 8;
pub const DEFAULT_MAX_PACKET_LEN: usize = 128 * 1024 * 1024;

pub const PACKET_TYPE_INIT_COMMAND_REQUEST: u32 = 0x0000_0001;
pub const PACKET_TYPE_INIT_COMMAND_ACK: u32 = 0x0000_0002;
pub const PACKET_TYPE_INIT_EVENT_REQUEST: u32 = 0x0000_0003;
pub const PACKET_TYPE_INIT_EVENT_ACK: u32 = 0x0000_0004;
pub const PACKET_TYPE_COMMAND_REQUEST: u32 = 0x0000_0006;
pub const PACKET_TYPE_COMMAND_RESPONSE: u32 = 0x0000_0007;
pub const PACKET_TYPE_EVENT: u32 = 0x0000_0008;
pub const PACKET_TYPE_DATA: u32 = 0x0000_0009;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtpIpPacket {
    pub packet_type: u32,
    pub payload: Vec<u8>,
}

impl PtpIpPacket {
    pub fn new(packet_type: u32, payload: impl Into<Vec<u8>>) -> Self {
        Self {
            packet_type,
            payload: payload.into(),
        }
    }

    pub fn len(&self) -> usize {
        PTP_IP_HEADER_LEN + self.payload.len()
    }

    pub fn is_empty(&self) -> bool {
        self.payload.is_empty()
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        let packet_len = self.len();
        let packet_len_u32 = u32::try_from(packet_len)
            .map_err(|_| ImporterError::internal("ptp/ip packet exceeds u32 length"))?;

        let mut out = BytesMut::with_capacity(packet_len);
        out.put_u32_le(packet_len_u32);
        out.put_u32_le(self.packet_type);
        out.extend_from_slice(&self.payload);
        Ok(out.to_vec())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        Self::decode_with_limit(bytes, DEFAULT_MAX_PACKET_LEN)
    }

    pub fn decode_with_limit(bytes: &[u8], max_packet_len: usize) -> Result<Self> {
        if bytes.len() < PTP_IP_HEADER_LEN {
            return Err(ImporterError::UnknownCameraResponse);
        }

        let mut reader = bytes;
        let declared_len = reader.get_u32_le() as usize;
        let packet_type = reader.get_u32_le();

        if declared_len < PTP_IP_HEADER_LEN {
            return Err(ImporterError::UnknownCameraResponse);
        }

        if declared_len > max_packet_len {
            return Err(ImporterError::internal(format!(
                "ptp/ip packet length {declared_len} exceeds limit {max_packet_len}"
            )));
        }

        if declared_len != bytes.len() {
            return Err(ImporterError::UnknownCameraResponse);
        }

        Ok(Self {
            packet_type,
            payload: reader.to_vec(),
        })
    }
}
