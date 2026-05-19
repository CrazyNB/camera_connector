use bytes::Buf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

use crate::{CameraEndpoint, ImporterError, Result};

use super::init::{build_init_command_request_payload, build_init_event_request_payload};
use super::packet::{
    PtpIpPacket, PACKET_TYPE_INIT_COMMAND_ACK, PACKET_TYPE_INIT_COMMAND_REQUEST,
    PACKET_TYPE_INIT_EVENT_ACK, PACKET_TYPE_INIT_EVENT_REQUEST, PTP_IP_HEADER_LEN,
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

pub struct PtpIpTransport {
    endpoint: CameraEndpoint,
    command_stream: TcpStream,
    event_stream: Option<TcpStream>,
    timeout: Duration,
    connection_id: Option<u32>,
}

impl PtpIpTransport {
    pub async fn connect(endpoint: CameraEndpoint) -> Result<Self> {
        let command_stream =
            timeout(DEFAULT_TIMEOUT, TcpStream::connect(endpoint.socket_addr())).await??;

        Ok(Self {
            endpoint,
            command_stream,
            event_stream: None,
            timeout: DEFAULT_TIMEOUT,
            connection_id: None,
        })
    }

    pub fn with_timeout(mut self, timeout_duration: Duration) -> Self {
        self.timeout = timeout_duration;
        self
    }

    pub async fn init_command(&mut self) -> Result<()> {
        let payload = build_init_command_request_payload("nikon-wireless-importer");
        let packet = PtpIpPacket::new(PACKET_TYPE_INIT_COMMAND_REQUEST, payload);
        self.write_packet_to_command(&packet).await?;

        let ack = self.read_packet_from_command().await?;
        if ack.packet_type != PACKET_TYPE_INIT_COMMAND_ACK {
            return Err(ImporterError::PtpInitFailed);
        }

        self.connection_id = parse_connection_id(&ack.payload);
        Ok(())
    }

    pub async fn init_event(&mut self) -> Result<()> {
        let connection_id = self.connection_id.ok_or(ImporterError::PtpInitFailed)?;
        let mut event_stream = timeout(
            self.timeout,
            TcpStream::connect(self.endpoint.socket_addr()),
        )
        .await??;
        let payload = build_init_event_request_payload(connection_id);
        let packet = PtpIpPacket::new(PACKET_TYPE_INIT_EVENT_REQUEST, payload);
        write_packet(&mut event_stream, self.timeout, &packet).await?;

        let ack = read_packet(&mut event_stream, self.timeout).await?;
        if ack.packet_type != PACKET_TYPE_INIT_EVENT_ACK {
            return Err(ImporterError::PtpInitFailed);
        }

        self.event_stream = Some(event_stream);
        Ok(())
    }

    pub async fn send_packet(&mut self, packet: PtpIpPacket) -> Result<()> {
        self.write_packet_to_command(&packet).await
    }

    pub async fn read_packet(&mut self) -> Result<PtpIpPacket> {
        self.read_packet_from_command().await
    }

    pub async fn close(&mut self) -> Result<()> {
        timeout(self.timeout, self.command_stream.shutdown()).await??;
        if let Some(event_stream) = &mut self.event_stream {
            timeout(self.timeout, event_stream.shutdown()).await??;
        }
        Ok(())
    }

    async fn write_packet_to_command(&mut self, packet: &PtpIpPacket) -> Result<()> {
        write_packet(&mut self.command_stream, self.timeout, packet).await
    }

    async fn read_packet_from_command(&mut self) -> Result<PtpIpPacket> {
        read_packet(&mut self.command_stream, self.timeout).await
    }
}

async fn write_packet(
    stream: &mut TcpStream,
    timeout_duration: Duration,
    packet: &PtpIpPacket,
) -> Result<()> {
    let encoded = packet.encode()?;
    timeout(timeout_duration, stream.write_all(&encoded)).await??;
    Ok(())
}

async fn read_packet(stream: &mut TcpStream, timeout_duration: Duration) -> Result<PtpIpPacket> {
    let mut header = [0_u8; PTP_IP_HEADER_LEN];
    timeout(timeout_duration, stream.read_exact(&mut header)).await??;

    let mut header_reader = header.as_slice();
    let packet_len = header_reader.get_u32_le() as usize;
    if packet_len < PTP_IP_HEADER_LEN {
        return Err(ImporterError::UnknownCameraResponse);
    }

    let payload_len = packet_len - PTP_IP_HEADER_LEN;
    let mut full_packet = Vec::with_capacity(packet_len);
    full_packet.extend_from_slice(&header);
    if payload_len > 0 {
        let mut payload = vec![0_u8; payload_len];
        timeout(timeout_duration, stream.read_exact(&mut payload)).await??;
        full_packet.extend_from_slice(&payload);
    }

    PtpIpPacket::decode(&full_packet)
}

fn parse_connection_id(payload: &[u8]) -> Option<u32> {
    if payload.len() < 4 {
        return None;
    }

    let mut reader = payload;
    Some(reader.get_u32_le())
}
