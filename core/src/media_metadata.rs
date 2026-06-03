use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const JPEG_SOI: [u8; 2] = [0xff, 0xd8];
const JPEG_APP1: u8 = 0xe1;
const EXIF_PREFIX: &[u8] = b"Exif\0\0";
const TIFF_HEADER_LEN: usize = 8;
const MAX_TIFF_METADATA_BYTES: usize = 1024 * 1024;

const TAG_DATE_TIME: u16 = 0x0132;
const TAG_EXIF_IFD_POINTER: u16 = 0x8769;
const TAG_DATE_TIME_ORIGINAL: u16 = 0x9003;
const TAG_DATE_TIME_DIGITIZED: u16 = 0x9004;
const TAG_SUBSEC_TIME: u16 = 0x9290;
const TAG_SUBSEC_TIME_ORIGINAL: u16 = 0x9291;
const TAG_SUBSEC_TIME_DIGITIZED: u16 = 0x9292;
const TAG_OFFSET_TIME: u16 = 0x9010;
const TAG_OFFSET_TIME_ORIGINAL: u16 = 0x9011;
const TAG_OFFSET_TIME_DIGITIZED: u16 = 0x9012;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Endian {
    Little,
    Big,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ExifTimes {
    image_datetime: Option<String>,
    original_datetime: Option<String>,
    digitized_datetime: Option<String>,
    subsec_time: Option<String>,
    subsec_original: Option<String>,
    subsec_digitized: Option<String>,
    offset_time: Option<String>,
    offset_original: Option<String>,
    offset_digitized: Option<String>,
}

pub(crate) fn extract_capture_time_ms(path: &Path) -> Option<i64> {
    let tiff = read_tiff_payload(path)?;
    parse_tiff_times(&tiff).and_then(|times| times.capture_time_ms())
}

fn read_tiff_payload(path: &Path) -> Option<Vec<u8>> {
    let mut file = File::open(path).ok()?;
    let mut header = [0_u8; 2];
    file.read_exact(&mut header).ok()?;
    file.seek(SeekFrom::Start(0)).ok()?;

    if header == JPEG_SOI {
        read_jpeg_exif_payload(file)
    } else if header == *b"II" || header == *b"MM" {
        let mut buffer = vec![0_u8; MAX_TIFF_METADATA_BYTES];
        let read = file.read(&mut buffer).ok()?;
        buffer.truncate(read);
        Some(buffer)
    } else {
        None
    }
}

fn read_jpeg_exif_payload(mut file: File) -> Option<Vec<u8>> {
    let mut marker = [0_u8; 2];
    file.read_exact(&mut marker).ok()?;
    if marker != JPEG_SOI {
        return None;
    }

    loop {
        file.read_exact(&mut marker).ok()?;
        if marker[0] != 0xff {
            return None;
        }
        let marker_kind = marker[1];
        if marker_kind == 0xd9 || marker_kind == 0xda {
            return None;
        }
        if marker_kind == 0xd8 || (0xd0..=0xd7).contains(&marker_kind) {
            continue;
        }

        let mut length_bytes = [0_u8; 2];
        file.read_exact(&mut length_bytes).ok()?;
        let segment_len = u16::from_be_bytes(length_bytes) as usize;
        if segment_len < 2 {
            return None;
        }
        let payload_len = segment_len - 2;
        let mut payload = vec![0_u8; payload_len];
        file.read_exact(&mut payload).ok()?;
        if marker_kind == JPEG_APP1 && payload.starts_with(EXIF_PREFIX) {
            return Some(payload[EXIF_PREFIX.len()..].to_vec());
        }
    }
}

fn parse_tiff_times(tiff: &[u8]) -> Option<ExifTimes> {
    if tiff.len() < TIFF_HEADER_LEN {
        return None;
    }
    let endian = match &tiff[..2] {
        b"II" => Endian::Little,
        b"MM" => Endian::Big,
        _ => return None,
    };
    if read_u16(tiff, endian, 2)? != 42 {
        return None;
    }
    let ifd0_offset = read_u32(tiff, endian, 4)? as usize;
    let mut times = ExifTimes::default();
    parse_ifd_times(tiff, endian, ifd0_offset, false, &mut times)?;
    Some(times)
}

fn parse_ifd_times(
    tiff: &[u8],
    endian: Endian,
    ifd_offset: usize,
    is_exif_ifd: bool,
    times: &mut ExifTimes,
) -> Option<()> {
    let entry_count = read_u16(tiff, endian, ifd_offset)? as usize;
    let entries_offset = ifd_offset.checked_add(2)?;
    for index in 0..entry_count {
        let entry_offset = entries_offset.checked_add(index.checked_mul(12)?)?;
        if entry_offset.checked_add(12)? > tiff.len() {
            return None;
        }
        let tag = read_u16(tiff, endian, entry_offset)?;
        let field_type = read_u16(tiff, endian, entry_offset + 2)?;
        let count = read_u32(tiff, endian, entry_offset + 4)? as usize;
        let value_offset = entry_offset + 8;

        if tag == TAG_EXIF_IFD_POINTER && !is_exif_ifd {
            let nested_offset =
                read_inline_or_offset_u32(tiff, endian, field_type, count, value_offset)? as usize;
            parse_ifd_times(tiff, endian, nested_offset, true, times)?;
            continue;
        }

        let value = match tag {
            TAG_DATE_TIME
            | TAG_DATE_TIME_ORIGINAL
            | TAG_DATE_TIME_DIGITIZED
            | TAG_SUBSEC_TIME
            | TAG_SUBSEC_TIME_ORIGINAL
            | TAG_SUBSEC_TIME_DIGITIZED
            | TAG_OFFSET_TIME
            | TAG_OFFSET_TIME_ORIGINAL
            | TAG_OFFSET_TIME_DIGITIZED => {
                read_ascii_value(tiff, endian, field_type, count, value_offset)
            }
            _ => None,
        };
        let Some(value) = value else {
            continue;
        };
        match tag {
            TAG_DATE_TIME if is_exif_ifd => {}
            TAG_DATE_TIME => times.image_datetime = Some(value),
            TAG_DATE_TIME_ORIGINAL => times.original_datetime = Some(value),
            TAG_DATE_TIME_DIGITIZED => times.digitized_datetime = Some(value),
            TAG_SUBSEC_TIME => times.subsec_time = Some(value),
            TAG_SUBSEC_TIME_ORIGINAL => times.subsec_original = Some(value),
            TAG_SUBSEC_TIME_DIGITIZED => times.subsec_digitized = Some(value),
            TAG_OFFSET_TIME => times.offset_time = Some(value),
            TAG_OFFSET_TIME_ORIGINAL => times.offset_original = Some(value),
            TAG_OFFSET_TIME_DIGITIZED => times.offset_digitized = Some(value),
            _ => {}
        }
    }
    Some(())
}

fn read_ascii_value(
    tiff: &[u8],
    endian: Endian,
    field_type: u16,
    count: usize,
    value_offset: usize,
) -> Option<String> {
    if field_type != 2 || count == 0 {
        return None;
    }
    let bytes = read_value_bytes(tiff, endian, field_type, count, value_offset)?;
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    let value = std::str::from_utf8(&bytes[..end]).ok()?.trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn read_value_bytes(
    tiff: &[u8],
    endian: Endian,
    field_type: u16,
    count: usize,
    value_offset: usize,
) -> Option<Vec<u8>> {
    let unit_size: usize = match field_type {
        1 | 2 | 7 => 1,
        3 => 2,
        4 | 9 => 4,
        5 | 10 => 8,
        _ => return None,
    };
    let size = unit_size.checked_mul(count)?;
    if size <= 4 {
        return Some(tiff.get(value_offset..value_offset + size)?.to_vec());
    }
    let offset = read_u32(tiff, endian, value_offset)? as usize;
    Some(tiff.get(offset..offset.checked_add(size)?)?.to_vec())
}

fn read_inline_or_offset_u32(
    tiff: &[u8],
    endian: Endian,
    field_type: u16,
    count: usize,
    value_offset: usize,
) -> Option<u32> {
    let bytes = read_value_bytes(tiff, endian, field_type, count, value_offset)?;
    if bytes.len() < 4 {
        return None;
    }
    read_u32(&bytes, endian, 0)
}

impl ExifTimes {
    fn capture_time_ms(&self) -> Option<i64> {
        let candidates = [
            (
                self.original_datetime.as_deref(),
                self.subsec_original.as_deref(),
                self.offset_original.as_deref(),
            ),
            (
                self.digitized_datetime.as_deref(),
                self.subsec_digitized.as_deref(),
                self.offset_digitized.as_deref(),
            ),
            (
                self.image_datetime.as_deref(),
                self.subsec_time.as_deref(),
                self.offset_time.as_deref(),
            ),
        ];
        candidates
            .into_iter()
            .find_map(|(datetime, subsec, offset)| {
                parse_exif_datetime_ms(datetime?, subsec, offset)
            })
    }
}

fn parse_exif_datetime_ms(
    datetime: &str,
    subsec: Option<&str>,
    offset: Option<&str>,
) -> Option<i64> {
    if datetime.len() != 19 {
        return None;
    }
    let year = parse_digits(datetime, 0, 4)?;
    let month = parse_digits(datetime, 5, 7)?;
    let day = parse_digits(datetime, 8, 10)?;
    let hour = parse_digits(datetime, 11, 13)?;
    let minute = parse_digits(datetime, 14, 16)?;
    let second = parse_digits(datetime, 17, 19)?;
    if datetime.as_bytes().get(4) != Some(&b':')
        || datetime.as_bytes().get(7) != Some(&b':')
        || datetime.as_bytes().get(10) != Some(&b' ')
        || datetime.as_bytes().get(13) != Some(&b':')
        || datetime.as_bytes().get(16) != Some(&b':')
        || !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return None;
    }

    let subsec_ms = parse_subsec_ms(subsec);
    let offset_ms = parse_offset_ms(offset).unwrap_or(0);
    let days = days_from_civil(year as i64, month as i64, day as i64)?;
    let local_ms = days
        .checked_mul(86_400_000)?
        .checked_add((hour as i64).checked_mul(3_600_000)?)?
        .checked_add((minute as i64).checked_mul(60_000)?)?
        .checked_add((second as i64).checked_mul(1_000)?)?
        .checked_add(subsec_ms)?;
    local_ms.checked_sub(offset_ms)
}

fn parse_digits(value: &str, start: usize, end: usize) -> Option<u32> {
    value.get(start..end)?.parse().ok()
}

fn parse_subsec_ms(value: Option<&str>) -> i64 {
    let Some(value) = value else {
        return 0;
    };
    let digits = value
        .chars()
        .filter(|char| char.is_ascii_digit())
        .take(3)
        .collect::<String>();
    if digits.is_empty() {
        return 0;
    }
    let padded = format!("{digits:0<3}");
    padded.parse::<i64>().unwrap_or(0)
}

fn parse_offset_ms(value: Option<&str>) -> Option<i64> {
    let value = value?;
    if value.len() != 6 {
        return None;
    }
    let sign = match value.as_bytes().first()? {
        b'+' => 1_i64,
        b'-' => -1_i64,
        _ => return None,
    };
    if value.as_bytes().get(3) != Some(&b':') {
        return None;
    }
    let hours = parse_digits(value, 1, 3)? as i64;
    let minutes = parse_digits(value, 4, 6)? as i64;
    if hours > 23 || minutes > 59 {
        return None;
    }
    Some(sign * (hours * 3_600_000 + minutes * 60_000))
}

fn days_from_civil(year: i64, month: i64, day: i64) -> Option<i64> {
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let year = year - (month <= 2) as i64;
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some(era * 146_097 + day_of_era - 719_468)
}

fn read_u16(bytes: &[u8], endian: Endian, offset: usize) -> Option<u16> {
    let slice = bytes.get(offset..offset.checked_add(2)?)?;
    Some(match endian {
        Endian::Little => u16::from_le_bytes([slice[0], slice[1]]),
        Endian::Big => u16::from_be_bytes([slice[0], slice[1]]),
    })
}

fn read_u32(bytes: &[u8], endian: Endian, offset: usize) -> Option<u32> {
    let slice = bytes.get(offset..offset.checked_add(4)?)?;
    Some(match endian {
        Endian::Little => u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]),
        Endian::Big => u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]),
    })
}

#[cfg(test)]
mod tests {
    use super::{parse_exif_datetime_ms, parse_subsec_ms};

    #[test]
    fn parses_exif_datetime_with_subsecond_and_offset() {
        assert_eq!(
            parse_exif_datetime_ms("2026:01:24 12:54:20", Some("55"), Some("+08:00")),
            Some(1_769_230_460_550),
        );
        assert_eq!(parse_subsec_ms(Some("160")), 160);
        assert_eq!(parse_subsec_ms(Some("7")), 700);
    }
}
