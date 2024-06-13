use std::path::Path;

use image::GenericImageView;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    path: String,
    media_type: String,
    width: u32,
    height: u32,
    fps: Option<f64>,
    bitrate: Option<u32>,
}

pub fn get_image_info(file_path: &str) -> Result<MediaInfo, String> {
    let img = image::open(&Path::new(file_path)).map_err(|e| e.to_string())?;
    let dimensions = img.dimensions();

    Ok(MediaInfo {
        path: file_path.to_string(),
        media_type: "image".to_string(),
        width: dimensions.0,
        height: dimensions.1,
        fps: None,
        bitrate: None,
    })
}
