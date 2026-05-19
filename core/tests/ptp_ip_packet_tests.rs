use nikon_importer_core::ptp_ip::{
    PtpIpPacket, PACKET_TYPE_INIT_COMMAND_REQUEST, PACKET_TYPE_INIT_EVENT_ACK,
};

#[test]
fn encodes_and_decodes_packet_header() {
    let packet = PtpIpPacket::new(PACKET_TYPE_INIT_COMMAND_REQUEST, vec![1, 2, 3, 4]);

    let encoded = packet.encode().expect("packet should encode");
    let decoded = PtpIpPacket::decode(&encoded).expect("packet should decode");

    assert_eq!(decoded.packet_type, PACKET_TYPE_INIT_COMMAND_REQUEST);
    assert_eq!(decoded.payload, vec![1, 2, 3, 4]);
}

#[test]
fn rejects_short_packets() {
    let result = PtpIpPacket::decode(&[1, 2, 3]);

    assert!(result.is_err());
}

#[test]
fn rejects_packet_length_mismatch() {
    let mut encoded = PtpIpPacket::new(PACKET_TYPE_INIT_EVENT_ACK, vec![9, 8, 7])
        .encode()
        .expect("packet should encode");
    encoded[0] = 99;

    let result = PtpIpPacket::decode(&encoded);

    assert!(result.is_err());
}

#[test]
fn rejects_packets_over_limit() {
    let encoded = PtpIpPacket::new(PACKET_TYPE_INIT_EVENT_ACK, vec![9, 8, 7])
        .encode()
        .expect("packet should encode");

    let result = PtpIpPacket::decode_with_limit(&encoded, 4);

    assert!(result.is_err());
}
