use crate::ptp_ip::{
    PtpIpPacket, PtpIpTransport, PACKET_TYPE_COMMAND_REQUEST, PACKET_TYPE_COMMAND_RESPONSE,
    PACKET_TYPE_DATA,
};
use crate::{ImporterError, Result};

use super::operation::{PtpOperation, PtpOperationCode, DATA_PHASE_NONE};
use super::response::PtpResponse;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtpOperationResult {
    pub response: PtpResponse,
    pub data: Vec<u8>,
}

pub struct PtpSession {
    transport: PtpIpTransport,
    session_id: u32,
    transaction_id: u32,
}

impl PtpSession {
    pub fn new(transport: PtpIpTransport) -> Self {
        Self {
            transport,
            session_id: 1,
            transaction_id: 0,
        }
    }

    pub async fn open(&mut self) -> Result<()> {
        let result = self
            .operation_with_phase(
                PtpOperationCode::OpenSession,
                vec![self.session_id],
                DATA_PHASE_NONE,
            )
            .await?;
        result.response.code.into_result()
    }

    pub async fn close(&mut self) -> Result<()> {
        let result = self
            .operation_with_phase(PtpOperationCode::CloseSession, Vec::new(), DATA_PHASE_NONE)
            .await?;
        result.response.code.into_result()?;
        self.transport.close().await
    }

    pub async fn operation(
        &mut self,
        code: PtpOperationCode,
        params: Vec<u32>,
    ) -> Result<PtpOperationResult> {
        self.operation_with_phase(code, params, super::operation::DATA_PHASE_IN)
            .await
    }

    pub async fn operation_with_phase(
        &mut self,
        code: PtpOperationCode,
        params: Vec<u32>,
        data_phase: u32,
    ) -> Result<PtpOperationResult> {
        let transaction_id = self.next_transaction_id();
        let operation = PtpOperation::new(code, transaction_id, params).with_data_phase(data_phase);
        let payload = operation.encode_request_payload()?;
        self.transport
            .send_packet(PtpIpPacket::new(PACKET_TYPE_COMMAND_REQUEST, payload))
            .await?;

        let mut data = Vec::new();
        loop {
            let packet = self.transport.read_packet().await?;
            match packet.packet_type {
                PACKET_TYPE_DATA => data.extend_from_slice(&packet.payload),
                PACKET_TYPE_COMMAND_RESPONSE => {
                    let response = PtpResponse::decode_payload(&packet.payload)?;
                    if response.transaction_id != transaction_id {
                        return Err(ImporterError::UnknownCameraResponse);
                    }
                    response.code.into_result()?;
                    return Ok(PtpOperationResult { response, data });
                }
                _ => return Err(ImporterError::UnknownCameraResponse),
            }
        }
    }

    fn next_transaction_id(&mut self) -> u32 {
        self.transaction_id = self.transaction_id.wrapping_add(1);
        if self.transaction_id == 0 {
            self.transaction_id = 1;
        }
        self.transaction_id
    }
}
