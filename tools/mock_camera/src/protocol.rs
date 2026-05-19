use bytes::{Buf, BufMut, BytesMut};
use nikon_importer_core::ptp_ip::{
    PtpIpPacket, PACKET_TYPE_COMMAND_REQUEST, PACKET_TYPE_COMMAND_RESPONSE, PACKET_TYPE_DATA,
    PACKET_TYPE_INIT_COMMAND_ACK, PACKET_TYPE_INIT_COMMAND_REQUEST, PACKET_TYPE_INIT_EVENT_ACK,
    PACKET_TYPE_INIT_EVENT_REQUEST, PTP_IP_HEADER_LEN,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::fixtures;

const CONNECTION_ID: u32 = 1;

pub async fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    loop {
        let packet = match read_packet(&mut stream).await {
            Ok(packet) => packet,
            Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => return Ok(()),
            Err(error) => return Err(error),
        };

        match packet.packet_type {
            PACKET_TYPE_INIT_COMMAND_REQUEST => {
                let response = init_command_ack();
                write_packet(&mut stream, &response).await?;
            }
            PACKET_TYPE_INIT_EVENT_REQUEST => {
                let response = PtpIpPacket::new(PACKET_TYPE_INIT_EVENT_ACK, Vec::new());
                write_packet(&mut stream, &response).await?;
            }
            PACKET_TYPE_COMMAND_REQUEST => {
                let command = CommandRequest::parse(&packet.payload);
                log_operation(&command);
                if let Some(data) = command_data_response(&command) {
                    write_packet(&mut stream, &PtpIpPacket::new(PACKET_TYPE_DATA, data)).await?;
                }
                let response = command_response(&command);
                write_packet(&mut stream, &response).await?;
            }
            other => {
                eprintln!("mock camera ignored packet type {other:#010x}");
            }
        }
    }
}

fn init_command_ack() -> PtpIpPacket {
    let mut payload = BytesMut::new();
    payload.put_u32_le(CONNECTION_ID);
    payload.extend_from_slice(&[0; 16]);
    payload.put_u32_le(1);
    PtpIpPacket::new(PACKET_TYPE_INIT_COMMAND_ACK, payload.to_vec())
}

fn command_response(command: &CommandRequest) -> PtpIpPacket {
    let mut payload = BytesMut::new();
    payload.put_u16_le(command_response_code(command));
    payload.put_u32_le(command.transaction_id);
    PtpIpPacket::new(PACKET_TYPE_COMMAND_RESPONSE, payload.to_vec())
}

fn command_response_code(command: &CommandRequest) -> u16 {
    match command.operation_code {
        0x1001 | 0x1002 | 0x1003 | 0x1004 | 0x1005 | 0x1007 | 0x1008 | 0x1009 | 0x100A => {
            if command_data_response(command).is_none()
                && matches!(command.operation_code, 0x1008..=0x100A)
            {
                0x2009
            } else {
                0x2001
            }
        }
        _ => 0x2005,
    }
}

fn command_data_response(command: &CommandRequest) -> Option<Vec<u8>> {
    match command.operation_code {
        0x1001 => Some(fixtures::device_info()),
        0x1004 => Some(fixtures::storage_ids()),
        0x1005 => Some(Vec::new()),
        0x1007 => Some(fixtures::object_handles()),
        0x1008 => fixtures::object_info(command.params.first().copied().unwrap_or_default()),
        0x1009 => fixtures::object_bytes(command.params.first().copied().unwrap_or_default()),
        0x100A => fixtures::thumbnail(command.params.first().copied().unwrap_or_default()),
        _ => None,
    }
}

fn log_operation(command: &CommandRequest) {
    if !command.valid {
        eprintln!("mock camera received malformed command request");
        return;
    }

    println!(
        "operation {:#06x}, transaction {}, data phase {}",
        command.operation_code, command.transaction_id, command.data_phase
    );
}

#[derive(Debug, Clone)]
struct CommandRequest {
    valid: bool,
    data_phase: u32,
    operation_code: u16,
    transaction_id: u32,
    params: Vec<u32>,
}

impl CommandRequest {
    fn parse(payload: &[u8]) -> Self {
        if payload.len() < 10 {
            return Self {
                valid: false,
                data_phase: 0,
                operation_code: 0,
                transaction_id: 0,
                params: Vec::new(),
            };
        }

        let mut reader = payload;
        let data_phase = reader.get_u32_le();
        let operation_code = reader.get_u16_le();
        let transaction_id = reader.get_u32_le();
        let mut params = Vec::with_capacity(reader.remaining() / 4);
        while reader.remaining() >= 4 {
            params.push(reader.get_u32_le());
        }

        Self {
            valid: true,
            data_phase,
            operation_code,
            transaction_id,
            params,
        }
    }
}

async fn read_packet(stream: &mut TcpStream) -> std::io::Result<PtpIpPacket> {
    let mut header = [0_u8; PTP_IP_HEADER_LEN];
    stream.read_exact(&mut header).await?;

    let mut reader = header.as_slice();
    let packet_len = reader.get_u32_le() as usize;
    if packet_len < PTP_IP_HEADER_LEN {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid packet length",
        ));
    }

    let payload_len = packet_len - PTP_IP_HEADER_LEN;
    let mut full_packet = Vec::with_capacity(packet_len);
    full_packet.extend_from_slice(&header);
    if payload_len > 0 {
        let mut payload = vec![0_u8; payload_len];
        stream.read_exact(&mut payload).await?;
        full_packet.extend_from_slice(&payload);
    }

    PtpIpPacket::decode(&full_packet)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
}

async fn write_packet(stream: &mut TcpStream, packet: &PtpIpPacket) -> std::io::Result<()> {
    let encoded = packet
        .encode()
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    stream.write_all(&encoded).await
}
