use bytes::{BufMut, BytesMut};

pub fn build_init_command_request_payload(client_name: &str) -> Vec<u8> {
    let mut out = BytesMut::new();
    out.extend_from_slice(&[0; 16]);
    put_utf16z(&mut out, client_name);
    out.put_u32_le(1);
    out.to_vec()
}

pub fn build_init_event_request_payload(connection_id: u32) -> Vec<u8> {
    let mut out = BytesMut::with_capacity(4);
    out.put_u32_le(connection_id);
    out.to_vec()
}

fn put_utf16z(out: &mut BytesMut, value: &str) {
    for unit in value.encode_utf16() {
        out.put_u16_le(unit);
    }
    out.put_u16_le(0);
}
