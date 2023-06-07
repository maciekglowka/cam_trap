use serde::Deserialize;
use std::fs;
use toml;

#[derive(Clone, Deserialize)]
pub struct CameraSettings {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub frame_rate: u32
}

#[derive(Deserialize)]
pub struct Settings {
    pub camera: CameraSettings,
    pub downsample_ratio: u32,
    pub sobel_thresh: i16,
    pub edge_thresh: u32,
    pub output_path: String,
    pub output_buffer_size: usize
}

pub fn load_settings(path: &str) -> Settings {
    let s = fs::read_to_string(path).expect("Error reading the settings file!");
    toml::from_str::<Settings>(&s).expect("Settings cannot be parsed!")
}