use image::metadata::Orientation;
use image::{DynamicImage, ImageBuffer, Rgb, RgbImage};

use super::{thumbnail_error, DesktopError};

pub(in crate::commands) fn raw_sensor_thumbnail_image(
    raw: &mut rawloader::RawImage,
) -> Result<DynamicImage, DesktopError> {
    let image = match &raw.data {
        rawloader::RawImageData::Integer(data) => raw_integer_to_rgb_image(raw, data)?,
        rawloader::RawImageData::Float(data) => raw_float_to_rgb_image(raw, data)?,
    };
    let mut image = DynamicImage::ImageRgb8(image);
    image.apply_orientation(rawloader_orientation_to_image(raw.orientation));
    Ok(image)
}

fn raw_integer_to_rgb_image(
    raw: &rawloader::RawImage,
    data: &[u16],
) -> Result<RgbImage, DesktopError> {
    let width = raw.width;
    let height = raw.height;
    if data.len() < width.saturating_mul(height).saturating_mul(raw.cpp.max(1)) {
        return Err(thumbnail_error("raw source has incomplete sensor data"));
    }
    let mut image = ImageBuffer::new(width as u32, height as u32);
    for y in 0..height {
        for x in 0..width {
            let rgb = if raw.cpp >= 3 {
                let base = (y * width + x) * raw.cpp;
                [
                    scale_raw_value(data[base], raw.blacklevels[0], raw.whitelevels[0]),
                    scale_raw_value(data[base + 1], raw.blacklevels[1], raw.whitelevels[1]),
                    scale_raw_value(data[base + 2], raw.blacklevels[2], raw.whitelevels[2]),
                ]
            } else {
                demosaic_raw_pixel(raw, data, x, y)
            };
            image.put_pixel(x as u32, y as u32, Rgb(rgb));
        }
    }
    Ok(image)
}

fn raw_float_to_rgb_image(
    raw: &rawloader::RawImage,
    data: &[f32],
) -> Result<RgbImage, DesktopError> {
    let width = raw.width;
    let height = raw.height;
    if data.len() < width.saturating_mul(height).saturating_mul(raw.cpp.max(1)) {
        return Err(thumbnail_error("raw source has incomplete sensor data"));
    }
    let mut image = ImageBuffer::new(width as u32, height as u32);
    for y in 0..height {
        for x in 0..width {
            let value = if raw.cpp >= 3 {
                let base = (y * width + x) * raw.cpp;
                [
                    scale_float_value(data[base]),
                    scale_float_value(data[base + 1]),
                    scale_float_value(data[base + 2]),
                ]
            } else {
                let gray = scale_float_value(data[y * width + x]);
                [gray, gray, gray]
            };
            image.put_pixel(x as u32, y as u32, Rgb(value));
        }
    }
    Ok(image)
}

fn demosaic_raw_pixel(raw: &rawloader::RawImage, data: &[u16], x: usize, y: usize) -> [u8; 3] {
    if !raw.cfa.is_valid() {
        let gray = scale_raw_value(
            data[y * raw.width + x],
            raw.blacklevels[0],
            raw.whitelevels[0],
        );
        return [gray, gray, gray];
    }
    let mut sum = [0u32; 3];
    let mut count = [0u32; 3];
    let y_start = y.saturating_sub(1);
    let y_end = (y + 1).min(raw.height.saturating_sub(1));
    let x_start = x.saturating_sub(1);
    let x_end = (x + 1).min(raw.width.saturating_sub(1));
    for sample_y in y_start..=y_end {
        for sample_x in x_start..=x_end {
            let color = raw.cfa.color_at(sample_y, sample_x).min(2);
            sum[color] += u32::from(data[sample_y * raw.width + sample_x]);
            count[color] += 1;
        }
    }
    let white = raw.whitelevels[0]
        .max(raw.whitelevels[1])
        .max(raw.whitelevels[2]);
    let black = raw.blacklevels[0]
        .min(raw.blacklevels[1])
        .min(raw.blacklevels[2]);
    [
        scale_raw_value((sum[0] / count[0].max(1)) as u16, black, white),
        scale_raw_value((sum[1] / count[1].max(1)) as u16, black, white),
        scale_raw_value((sum[2] / count[2].max(1)) as u16, black, white),
    ]
}

fn scale_raw_value(value: u16, black: u16, white: u16) -> u8 {
    let black = u32::from(black);
    let white = u32::from(white.max(black as u16 + 1));
    let value = u32::from(value).saturating_sub(black).min(white - black);
    ((value * 255) / (white - black)) as u8
}

fn scale_float_value(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn rawloader_orientation_to_image(orientation: rawloader::Orientation) -> Orientation {
    match orientation {
        rawloader::Orientation::HorizontalFlip => Orientation::FlipHorizontal,
        rawloader::Orientation::Rotate180 => Orientation::Rotate180,
        rawloader::Orientation::VerticalFlip => Orientation::FlipVertical,
        rawloader::Orientation::Transpose => Orientation::Rotate90FlipH,
        rawloader::Orientation::Rotate90 => Orientation::Rotate90,
        rawloader::Orientation::Transverse => Orientation::Rotate270FlipH,
        rawloader::Orientation::Rotate270 => Orientation::Rotate270,
        rawloader::Orientation::Normal | rawloader::Orientation::Unknown => {
            Orientation::NoTransforms
        }
    }
}
