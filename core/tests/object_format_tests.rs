use std::str::FromStr;

use camera_connector_core::{AssetFormatRole, ObjectFormat};

#[test]
fn object_format_parses_cli_aliases_from_core() {
    assert_eq!(ObjectFormat::from_str("jpg").unwrap(), ObjectFormat::Jpeg);
    assert_eq!(ObjectFormat::from_str("SRF").unwrap(), ObjectFormat::Arw);
    assert_eq!(ObjectFormat::from_str("rwl").unwrap(), ObjectFormat::Rw2);
    assert_eq!(ObjectFormat::from_str("tiff").unwrap(), ObjectFormat::Tiff);
}

#[test]
fn object_format_exposes_shared_role_classification() {
    assert_eq!(ObjectFormat::Jpeg.role(), AssetFormatRole::Jpeg);
    assert_eq!(ObjectFormat::Nef.role(), AssetFormatRole::Raw);
    assert_eq!(ObjectFormat::Mp4.role(), AssetFormatRole::Video);
    assert_eq!(ObjectFormat::Tiff.role(), AssetFormatRole::Other);
    assert_eq!(
        AssetFormatRole::from_str("raw").unwrap(),
        AssetFormatRole::Raw
    );
}
