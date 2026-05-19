use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{CameraObject, ObjectFormat};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CameraAssetGroup {
    pub group_key: String,
    pub primary: CameraObject,
    pub jpeg: Option<CameraObject>,
    pub raw: Option<CameraObject>,
    pub video: Option<CameraObject>,
}

pub fn group_camera_objects(objects: Vec<CameraObject>) -> Vec<CameraAssetGroup> {
    let mut grouped: BTreeMap<String, Vec<CameraObject>> = BTreeMap::new();

    for object in objects {
        let group_key = object
            .group_key
            .clone()
            .unwrap_or_else(|| format!("HANDLE_{}", object.handle));
        grouped.entry(group_key).or_default().push(object);
    }

    let mut groups: Vec<_> = grouped
        .into_iter()
        .map(|(group_key, mut members)| {
            members.sort_by_key(|object| format_rank(object.format));

            let jpeg = members
                .iter()
                .find(|object| object.format == ObjectFormat::Jpeg)
                .cloned();
            let raw = members
                .iter()
                .find(|object| object.format.is_raw())
                .cloned();
            let video = members
                .iter()
                .find(|object| object.format.is_video())
                .cloned();
            let primary = jpeg
                .clone()
                .or_else(|| raw.clone())
                .or_else(|| video.clone())
                .unwrap_or_else(|| members[0].clone());

            CameraAssetGroup {
                group_key,
                primary,
                jpeg,
                raw,
                video,
            }
        })
        .collect();

    groups.sort_by(|left, right| {
        right
            .primary
            .capture_time_ms
            .cmp(&left.primary.capture_time_ms)
            .then_with(|| left.group_key.cmp(&right.group_key))
    });

    groups
}

fn format_rank(format: ObjectFormat) -> u8 {
    match format {
        ObjectFormat::Jpeg => 0,
        ObjectFormat::Nef => 1,
        ObjectFormat::Mov | ObjectFormat::Mp4 => 2,
        ObjectFormat::Tiff => 3,
        ObjectFormat::Unknown => 4,
    }
}
