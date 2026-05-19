use bytes::{BufMut, BytesMut};
use nikon_importer_core::ptp::{DeviceInfoDataset, ObjectInfoDataset, StorageInfoDataset};
use nikon_importer_core::ObjectFormat;

#[test]
fn parses_ptp_string_fields_in_device_info() {
    let mut bytes = BytesMut::new();
    bytes.put_u16_le(100);
    bytes.put_u32_le(0);
    bytes.put_u16_le(100);
    put_ptp_string(&mut bytes, "");
    bytes.put_u16_le(0);
    put_u16_array(&mut bytes, &[0x1001, 0x1002, 0x100A]);
    put_u16_array(&mut bytes, &[]);
    put_u16_array(&mut bytes, &[]);
    put_u16_array(&mut bytes, &[]);
    put_u16_array(&mut bytes, &[0x3801, 0xB103]);
    put_ptp_string(&mut bytes, "Nikon");
    put_ptp_string(&mut bytes, "Zf");
    put_ptp_string(&mut bytes, "1.00");
    put_ptp_string(&mut bytes, "1234567");

    let dataset = DeviceInfoDataset::parse(&bytes).expect("device info should parse");

    assert_eq!(dataset.manufacturer, "Nikon");
    assert_eq!(dataset.model, "Zf");
    assert_eq!(dataset.firmware_version.as_deref(), Some("1.00"));
    assert_eq!(dataset.serial_number.as_deref(), Some("1234567"));
    assert_eq!(dataset.supported_operations, vec![0x1001, 0x1002, 0x100A]);
    assert_eq!(dataset.supported_formats, vec![0x3801, 0xB103]);
}

#[test]
fn parses_storage_info() {
    let mut bytes = BytesMut::new();
    bytes.put_u16_le(4);
    bytes.put_u16_le(3);
    bytes.put_u16_le(0);
    bytes.put_u64_le(128_000_000_000);
    bytes.put_u64_le(64_000_000_000);
    bytes.put_u32_le(1234);
    put_ptp_string(&mut bytes, "Card Slot 1");
    put_ptp_string(&mut bytes, "NIKON");

    let dataset = StorageInfoDataset::parse(&bytes).expect("storage info should parse");

    assert_eq!(dataset.max_capacity_bytes, 128_000_000_000);
    assert_eq!(dataset.free_space_bytes, 64_000_000_000);
    assert_eq!(dataset.storage_description, "Card Slot 1");
    assert_eq!(dataset.volume_label, "NIKON");
}

#[test]
fn parses_object_info_into_camera_object() {
    let mut bytes = BytesMut::new();
    bytes.put_u32_le(1);
    bytes.put_u16_le(0x3801);
    bytes.put_u16_le(0);
    bytes.put_u32_le(8_700_000);
    bytes.put_u16_le(0x3801);
    bytes.put_u32_le(42_000);
    bytes.put_u32_le(320);
    bytes.put_u32_le(213);
    bytes.put_u32_le(6048);
    bytes.put_u32_le(4032);
    bytes.put_u32_le(24);
    bytes.put_u32_le(0);
    bytes.put_u16_le(0);
    bytes.put_u32_le(0);
    bytes.put_u32_le(0);
    put_ptp_string(&mut bytes, "DSC_1234.JPG");
    put_ptp_string(&mut bytes, "20260520T001000");
    put_ptp_string(&mut bytes, "20260520T001100");
    put_ptp_string(&mut bytes, "");

    let object = ObjectInfoDataset::parse(77, &bytes).expect("object info should parse");

    assert_eq!(object.handle, 77);
    assert_eq!(object.storage_id, 1);
    assert_eq!(object.filename, "DSC_1234.JPG");
    assert_eq!(object.format, ObjectFormat::Jpeg);
    assert_eq!(object.size_bytes, 8_700_000);
    assert_eq!(object.width, Some(6048));
    assert_eq!(object.height, Some(4032));
    assert!(object.thumb_available);
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
