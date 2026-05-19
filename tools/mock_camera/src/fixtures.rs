use bytes::{BufMut, BytesMut};

pub const STORAGE_ID: u32 = 1;
pub const JPEG_HANDLE: u32 = 101;
pub const RAW_HANDLE: u32 = 102;

pub fn device_info() -> Vec<u8> {
    let mut bytes = BytesMut::new();
    bytes.put_u16_le(100);
    bytes.put_u32_le(0);
    bytes.put_u16_le(100);
    put_ptp_string(&mut bytes, "");
    bytes.put_u16_le(0);
    put_u16_array(
        &mut bytes,
        &[
            0x1001, 0x1002, 0x1003, 0x1004, 0x1005, 0x1007, 0x1008, 0x1009, 0x100A,
        ],
    );
    put_u16_array(&mut bytes, &[]);
    put_u16_array(&mut bytes, &[]);
    put_u16_array(&mut bytes, &[]);
    put_u16_array(&mut bytes, &[0x3801, 0xB103]);
    put_ptp_string(&mut bytes, "Nikon");
    put_ptp_string(&mut bytes, "Mock Zf");
    put_ptp_string(&mut bytes, "1.00");
    put_ptp_string(&mut bytes, "MOCK123456");
    bytes.to_vec()
}

pub fn storage_ids() -> Vec<u8> {
    let mut bytes = BytesMut::new();
    bytes.put_u32_le(1);
    bytes.put_u32_le(STORAGE_ID);
    bytes.to_vec()
}

pub fn object_handles() -> Vec<u8> {
    let mut bytes = BytesMut::new();
    bytes.put_u32_le(2);
    bytes.put_u32_le(JPEG_HANDLE);
    bytes.put_u32_le(RAW_HANDLE);
    bytes.to_vec()
}

pub fn object_info(handle: u32) -> Option<Vec<u8>> {
    match handle {
        JPEG_HANDLE => Some(object_info_bytes("DSC_1234.JPG", 0x3801, 8_700_000, 42_000)),
        RAW_HANDLE => Some(object_info_bytes("DSC_1234.NEF", 0xB103, 39_500_000, 0)),
        _ => None,
    }
}

pub fn thumbnail(handle: u32) -> Option<Vec<u8>> {
    if handle == JPEG_HANDLE || handle == RAW_HANDLE {
        Some(vec![
            0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F', 0x00, 0x01, 0x01, 0x00,
            0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xD9,
        ])
    } else {
        None
    }
}

pub fn object_bytes(handle: u32) -> Option<Vec<u8>> {
    match handle {
        JPEG_HANDLE => Some(vec![0x4A; 1024]),
        RAW_HANDLE => Some(vec![0x4E; 2048]),
        _ => None,
    }
}

fn object_info_bytes(filename: &str, format_code: u16, size: u32, thumb_size: u32) -> Vec<u8> {
    let mut bytes = BytesMut::new();
    bytes.put_u32_le(STORAGE_ID);
    bytes.put_u16_le(format_code);
    bytes.put_u16_le(0);
    bytes.put_u32_le(size);
    bytes.put_u16_le(0x3801);
    bytes.put_u32_le(thumb_size);
    bytes.put_u32_le(320);
    bytes.put_u32_le(213);
    bytes.put_u32_le(6048);
    bytes.put_u32_le(4032);
    bytes.put_u32_le(24);
    bytes.put_u32_le(0);
    bytes.put_u16_le(0);
    bytes.put_u32_le(0);
    bytes.put_u32_le(0);
    put_ptp_string(&mut bytes, filename);
    put_ptp_string(&mut bytes, "20260520T001000");
    put_ptp_string(&mut bytes, "20260520T001100");
    put_ptp_string(&mut bytes, "");
    bytes.to_vec()
}

fn put_u16_array(bytes: &mut BytesMut, values: &[u16]) {
    bytes.put_u32_le(values.len() as u32);
    for value in values {
        bytes.put_u16_le(*value);
    }
}

fn put_ptp_string(bytes: &mut BytesMut, value: &str) {
    if value.is_empty() {
        bytes.put_u8(0);
        return;
    }

    let encoded: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
    bytes.put_u8(encoded.len() as u8);
    for unit in encoded {
        bytes.put_u16_le(unit);
    }
}
