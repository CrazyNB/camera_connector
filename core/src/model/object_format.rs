use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObjectFormat {
    Jpeg,
    Nef,
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
            "nef" | "nrw" => Self::Nef,
            "mov" => Self::Mov,
            "mp4" => Self::Mp4,
            "tif" | "tiff" => Self::Tiff,
            _ => Self::Unknown,
        }
    }

    pub fn is_raw(self) -> bool {
        matches!(self, Self::Nef)
    }

    pub fn is_video(self) -> bool {
        matches!(self, Self::Mov | Self::Mp4)
    }
}
