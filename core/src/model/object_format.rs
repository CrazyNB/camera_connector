use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetFormatRole {
    Jpeg,
    Raw,
    Video,
    Other,
}

impl AssetFormatRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Jpeg => "jpeg",
            Self::Raw => "raw",
            Self::Video => "video",
            Self::Other => "other",
        }
    }
}

impl FromStr for AssetFormatRole {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "jpeg" | "jpg" => Ok(Self::Jpeg),
            "raw" => Ok(Self::Raw),
            "video" | "movie" => Ok(Self::Video),
            "other" | "unknown" => Ok(Self::Other),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObjectFormat {
    Jpeg,
    Nef,
    Nrw,
    Cr2,
    Cr3,
    Arw,
    Raf,
    Rw2,
    Orf,
    Pef,
    Dng,
    Mov,
    Mp4,
    Tiff,
    Unknown,
}

impl ObjectFormat {
    pub fn from_filename(filename: &str) -> Self {
        let Some((_, extension)) = filename.rsplit_once('.') else {
            return Self::Unknown;
        };

        extension.parse().unwrap_or(Self::Unknown)
    }

    pub fn role(self) -> AssetFormatRole {
        if self == Self::Jpeg {
            AssetFormatRole::Jpeg
        } else if self.is_raw() {
            AssetFormatRole::Raw
        } else if self.is_video() {
            AssetFormatRole::Video
        } else {
            AssetFormatRole::Other
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Jpeg => "jpeg",
            Self::Nef => "nef",
            Self::Nrw => "nrw",
            Self::Cr2 => "cr2",
            Self::Cr3 => "cr3",
            Self::Arw => "arw",
            Self::Raf => "raf",
            Self::Rw2 => "rw2",
            Self::Orf => "orf",
            Self::Pef => "pef",
            Self::Dng => "dng",
            Self::Mov => "mov",
            Self::Mp4 => "mp4",
            Self::Tiff => "tiff",
            Self::Unknown => "unknown",
        }
    }

    pub fn is_raw(self) -> bool {
        matches!(
            self,
            Self::Nef
                | Self::Nrw
                | Self::Cr2
                | Self::Cr3
                | Self::Arw
                | Self::Raf
                | Self::Rw2
                | Self::Orf
                | Self::Pef
                | Self::Dng
        )
    }

    pub fn is_video(self) -> bool {
        matches!(self, Self::Mov | Self::Mp4)
    }

    pub fn is_photo(self) -> bool {
        self == Self::Jpeg || self == Self::Tiff || self.is_raw()
    }

    pub fn is_supported_media(self) -> bool {
        self.is_photo() || self.is_video()
    }
}

impl FromStr for ObjectFormat {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value.trim().to_ascii_lowercase().as_str() {
            "jpg" | "jpeg" => Self::Jpeg,
            "nef" => Self::Nef,
            "nrw" => Self::Nrw,
            "cr2" => Self::Cr2,
            "cr3" => Self::Cr3,
            "arw" | "srf" | "sr2" => Self::Arw,
            "raf" => Self::Raf,
            "rw2" | "rwl" => Self::Rw2,
            "orf" => Self::Orf,
            "pef" => Self::Pef,
            "dng" => Self::Dng,
            "mov" => Self::Mov,
            "mp4" => Self::Mp4,
            "tif" | "tiff" => Self::Tiff,
            "unknown" => Self::Unknown,
            _ => return Err(()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn photo_format_predicate_matches_cross_platform_asset_semantics() {
        assert!(ObjectFormat::Jpeg.is_photo());
        assert!(ObjectFormat::Nef.is_photo());
        assert!(ObjectFormat::Tiff.is_photo());
        assert!(!ObjectFormat::Mov.is_photo());
        assert!(!ObjectFormat::Unknown.is_photo());
    }
}
