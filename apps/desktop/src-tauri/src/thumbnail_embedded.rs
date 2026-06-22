use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use image::metadata::Orientation;
use image::DynamicImage;

use super::{raw_sensor_thumbnail_image, thumbnail_error, DesktopError};

pub(in super::super) fn embedded_jpeg_preview_image(source_path: &Path) -> Option<DynamicImage> {
    let preview = embedded_jpeg_preview_bytes(source_path)?;
    let mut image = image::load_from_memory(&preview).ok()?;
    if let Some(orientation) = embedded_preview_orientation(source_path) {
        image.apply_orientation(orientation);
    }
    Some(image)
}

fn embedded_jpeg_preview_bytes(source_path: &Path) -> Option<Vec<u8>> {
    read_jpeg_exif_payload(source_path)
        .and_then(|payload| embedded_jpeg_from_exif_payload(&payload))
        .or_else(|| read_tiff_embedded_jpeg(source_path))
}

fn embedded_preview_orientation(source_path: &Path) -> Option<Orientation> {
    read_jpeg_exif_payload(source_path)
        .and_then(|payload| tiff_orientation_from_exif_payload(&payload))
        .or_else(|| read_tiff_orientation(source_path))
}

fn read_jpeg_exif_payload(source_path: &Path) -> Option<Vec<u8>> {
    let file = File::open(source_path).ok()?;
    let mut reader = BufReader::new(file);
    let mut marker = [0u8; 2];
    reader.read_exact(&mut marker).ok()?;
    if marker != [0xff, 0xd8] {
        return None;
    }

    loop {
        let mut byte = [0u8; 1];
        if reader.read_exact(&mut byte).is_err() {
            return None;
        }
        if byte[0] != 0xff {
            continue;
        }
        loop {
            reader.read_exact(&mut byte).ok()?;
            if byte[0] != 0xff {
                break;
            }
        }
        let marker = byte[0];
        if marker == 0xda || marker == 0xd9 {
            return None;
        }
        if marker == 0x01 || (0xd0..=0xd7).contains(&marker) {
            continue;
        }
        let mut length = [0u8; 2];
        reader.read_exact(&mut length).ok()?;
        let segment_len = u16::from_be_bytes(length) as usize;
        if segment_len < 2 {
            return None;
        }
        let payload_len = segment_len - 2;
        if marker == 0xe1 {
            let mut payload = vec![0u8; payload_len];
            reader.read_exact(&mut payload).ok()?;
            if payload.starts_with(b"Exif\0\0") {
                return Some(payload);
            }
        } else {
            skip_exact(&mut reader, payload_len)?;
        }
    }
}

fn skip_exact(reader: &mut impl Read, mut len: usize) -> Option<()> {
    let mut buffer = [0u8; 4096];
    while len > 0 {
        let chunk_len = len.min(buffer.len());
        reader.read_exact(&mut buffer[..chunk_len]).ok()?;
        len -= chunk_len;
    }
    Some(())
}

fn embedded_jpeg_from_exif_payload(payload: &[u8]) -> Option<Vec<u8>> {
    let tiff = payload.strip_prefix(b"Exif\0\0")?;
    embedded_jpeg_from_tiff_payload(tiff)
}

fn tiff_orientation_from_exif_payload(payload: &[u8]) -> Option<Orientation> {
    let tiff = payload.strip_prefix(b"Exif\0\0")?;
    tiff_orientation_from_payload(tiff)
}

fn read_tiff_orientation(source_path: &Path) -> Option<Orientation> {
    let mut file = File::open(source_path).ok()?;
    let mut header = [0u8; 8];
    file.read_exact(&mut header).ok()?;
    let endian = TiffEndian::from_header(&header)?;
    let first_ifd_offset = endian.read_u32(&header, 4)? as usize;
    file.seek(SeekFrom::Start(0)).ok()?;
    let mut payload = Vec::new();
    file.read_to_end(&mut payload).ok()?;
    tiff_orientation_from_ifd_payload(&payload, endian, first_ifd_offset)
}

fn tiff_orientation_from_payload(tiff: &[u8]) -> Option<Orientation> {
    let endian = TiffEndian::from_header(tiff)?;
    let first_ifd_offset = endian.read_u32(tiff, 4)? as usize;
    tiff_orientation_from_ifd_payload(tiff, endian, first_ifd_offset)
}

fn tiff_orientation_from_ifd_payload(
    tiff: &[u8],
    endian: TiffEndian,
    ifd_offset: usize,
) -> Option<Orientation> {
    let mut orientation = None;
    for_each_ifd_entry(tiff, endian, ifd_offset, |tag, value| {
        if tag == 0x0112 {
            orientation = Orientation::from_exif(value as u8);
        }
    })?;
    orientation
}

fn embedded_jpeg_from_tiff_payload(tiff: &[u8]) -> Option<Vec<u8>> {
    if tiff.len() < 8 {
        return None;
    }
    let endian = TiffEndian::from_header(tiff)?;
    if endian.read_u16(tiff, 2)? != 42 {
        return None;
    }
    let ifd0_offset = usize::try_from(endian.read_u32(tiff, 4)?).ok()?;
    let ifd1_offset = usize::try_from(next_ifd_offset(tiff, endian, ifd0_offset)?).ok()?;
    if ifd1_offset == 0 {
        return None;
    }
    let mut jpeg_offset = None;
    let mut jpeg_len = None;
    for_each_ifd_entry(tiff, endian, ifd1_offset, |tag, value| match tag {
        0x0201 => jpeg_offset = Some(value),
        0x0202 => jpeg_len = Some(value),
        _ => {}
    })?;
    let start = usize::try_from(jpeg_offset?).ok()?;
    let len = usize::try_from(jpeg_len?).ok()?;
    let end = start.checked_add(len)?;
    let data = tiff.get(start..end)?;
    if !data.starts_with(&[0xff, 0xd8]) {
        return None;
    }
    Some(data.to_vec())
}

fn read_tiff_embedded_jpeg(source_path: &Path) -> Option<Vec<u8>> {
    let mut file = File::open(source_path).ok()?;
    let mut header = [0u8; 8];
    file.read_exact(&mut header).ok()?;
    let endian = TiffEndian::from_header(&header)?;
    if endian.read_u16(&header, 2)? != 42 {
        return None;
    }
    let first_ifd_offset = u64::from(endian.read_u32(&header, 4)?);
    read_tiff_ifd_embedded_jpeg(&mut file, endian, first_ifd_offset, 0)
}

fn read_tiff_ifd_embedded_jpeg(
    file: &mut File,
    endian: TiffEndian,
    ifd_offset: u64,
    depth: u8,
) -> Option<Vec<u8>> {
    if depth > 4 || ifd_offset == 0 {
        return None;
    }
    file.seek(SeekFrom::Start(ifd_offset)).ok()?;
    let mut count_bytes = [0u8; 2];
    file.read_exact(&mut count_bytes).ok()?;
    let count = usize::from(endian.read_u16(&count_bytes, 0)?);
    let mut jpeg_offset = None;
    let mut jpeg_len = None;
    let mut child_ifd_offsets = Vec::new();

    for _ in 0..count {
        let mut entry = [0u8; 12];
        file.read_exact(&mut entry).ok()?;
        let tag = endian.read_u16(&entry, 0)?;
        let field_type = endian.read_u16(&entry, 2)?;
        let component_count = endian.read_u32(&entry, 4)?;
        let value_or_offset = endian.read_u32(&entry, 8)?;
        match tag {
            0x0201 if component_count == 1 => {
                jpeg_offset = tiff_entry_first_u32(
                    file,
                    endian,
                    field_type,
                    component_count,
                    value_or_offset,
                );
            }
            0x0202 if component_count == 1 => {
                jpeg_len = tiff_entry_first_u32(
                    file,
                    endian,
                    field_type,
                    component_count,
                    value_or_offset,
                );
            }
            0x014a => {
                child_ifd_offsets.extend(tiff_entry_u32_values(
                    file,
                    endian,
                    field_type,
                    component_count,
                    value_or_offset,
                    8,
                ));
            }
            _ => {}
        }
    }

    if let (Some(offset), Some(len)) = (jpeg_offset, jpeg_len) {
        let data = read_file_range(file, u64::from(offset), usize::try_from(len).ok()?)?;
        if data.starts_with(&[0xff, 0xd8]) {
            return Some(data);
        }
    }

    let mut next_offset = [0u8; 4];
    file.read_exact(&mut next_offset).ok()?;
    let next_ifd_offset = endian.read_u32(&next_offset, 0)?;
    if next_ifd_offset != 0 {
        child_ifd_offsets.push(next_ifd_offset);
    }

    for child_offset in child_ifd_offsets {
        if let Some(data) =
            read_tiff_ifd_embedded_jpeg(file, endian, u64::from(child_offset), depth + 1)
        {
            return Some(data);
        }
    }
    None
}

pub(in super::super) fn is_raw_extension(source_path: &Path) -> bool {
    matches!(
        source_path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase())
            .as_deref(),
        Some("nef" | "nrw" | "cr2" | "cr3" | "arw" | "raf" | "rw2" | "orf" | "pef" | "dng")
    )
}

pub(in super::super) fn is_browser_original_extension(source_path: &Path) -> bool {
    matches!(
        source_path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase())
            .as_deref(),
        Some("jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp")
    )
}

pub(in super::super) fn raw_thumbnail_image_from_file(
    source_path: &Path,
) -> Result<DynamicImage, DesktopError> {
    let mut raw = rawloader::decode_file(source_path)
        .map_err(|error| thumbnail_error(format!("raw source could not be decoded: {error}")))?;
    raw_sensor_thumbnail_image(&mut raw)
}

fn tiff_entry_first_u32(
    file: &mut File,
    endian: TiffEndian,
    field_type: u16,
    component_count: u32,
    value_or_offset: u32,
) -> Option<u32> {
    tiff_entry_u32_values(
        file,
        endian,
        field_type,
        component_count,
        value_or_offset,
        1,
    )
    .into_iter()
    .next()
}

fn tiff_entry_u32_values(
    file: &mut File,
    endian: TiffEndian,
    field_type: u16,
    component_count: u32,
    value_or_offset: u32,
    max_values: usize,
) -> Vec<u32> {
    let value_size = match field_type {
        3 => 2usize,
        4 => 4usize,
        _ => return Vec::new(),
    };
    let count = usize::try_from(component_count).unwrap_or_default();
    let inline_bytes = count.saturating_mul(value_size);
    if inline_bytes <= 4 {
        return match field_type {
            3 => vec![match endian {
                TiffEndian::Little => value_or_offset & 0xffff,
                TiffEndian::Big => value_or_offset >> 16,
            }],
            4 => vec![value_or_offset],
            _ => Vec::new(),
        };
    }
    let bytes_to_read = count.min(max_values).saturating_mul(value_size);
    let Some(data) = read_file_range(file, u64::from(value_or_offset), bytes_to_read) else {
        return Vec::new();
    };
    let mut values = Vec::new();
    for index in 0..count.min(max_values) {
        let offset = index * value_size;
        let value = match field_type {
            3 => data
                .get(offset..offset + 2)
                .and_then(|bytes| endian.read_u16(bytes, 0))
                .map(u32::from),
            4 => data
                .get(offset..offset + 4)
                .and_then(|bytes| endian.read_u32(bytes, 0)),
            _ => None,
        };
        if let Some(value) = value {
            values.push(value);
        }
    }
    values
}

fn read_file_range(file: &mut File, offset: u64, len: usize) -> Option<Vec<u8>> {
    if len > 64 * 1024 * 1024 {
        return None;
    }
    file.seek(SeekFrom::Start(offset)).ok()?;
    let mut data = vec![0u8; len];
    file.read_exact(&mut data).ok()?;
    Some(data)
}

fn next_ifd_offset(tiff: &[u8], endian: TiffEndian, ifd_offset: usize) -> Option<u32> {
    let count = usize::from(endian.read_u16(tiff, ifd_offset)?);
    let next_offset_position = ifd_offset
        .checked_add(2)?
        .checked_add(count.checked_mul(12)?)?;
    endian.read_u32(tiff, next_offset_position)
}

fn for_each_ifd_entry(
    tiff: &[u8],
    endian: TiffEndian,
    ifd_offset: usize,
    mut visit: impl FnMut(u16, u32),
) -> Option<()> {
    let count = usize::from(endian.read_u16(tiff, ifd_offset)?);
    let entries_start = ifd_offset.checked_add(2)?;
    for entry_index in 0..count {
        let entry = entries_start.checked_add(entry_index.checked_mul(12)?)?;
        let tag = endian.read_u16(tiff, entry)?;
        let field_type = endian.read_u16(tiff, entry + 2)?;
        let component_count = endian.read_u32(tiff, entry + 4)?;
        if component_count != 1 {
            continue;
        }
        let value = match field_type {
            3 => u32::from(endian.read_u16(tiff, entry + 8)?),
            4 => endian.read_u32(tiff, entry + 8)?,
            _ => continue,
        };
        visit(tag, value);
    }
    Some(())
}

#[derive(Debug, Clone, Copy)]
enum TiffEndian {
    Little,
    Big,
}

impl TiffEndian {
    fn from_header(tiff: &[u8]) -> Option<Self> {
        match tiff.get(0..2)? {
            b"II" => Some(Self::Little),
            b"MM" => Some(Self::Big),
            _ => None,
        }
    }

    fn read_u16(self, bytes: &[u8], offset: usize) -> Option<u16> {
        let value = bytes.get(offset..offset + 2)?;
        Some(match self {
            Self::Little => u16::from_le_bytes([value[0], value[1]]),
            Self::Big => u16::from_be_bytes([value[0], value[1]]),
        })
    }

    fn read_u32(self, bytes: &[u8], offset: usize) -> Option<u32> {
        let value = bytes.get(offset..offset + 4)?;
        Some(match self {
            Self::Little => u32::from_le_bytes([value[0], value[1], value[2], value[3]]),
            Self::Big => u32::from_be_bytes([value[0], value[1], value[2], value[3]]),
        })
    }
}
