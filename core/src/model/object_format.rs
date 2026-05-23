use serde::{Deserialize, Serialize};

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

        match extension.to_ascii_lowercase().as_str() {
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
            _ => Self::Unknown,
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

    pub fn is_supported_media(self) -> bool {
        self == Self::Jpeg || self == Self::Tiff || self.is_raw() || self.is_video()
    }
}
