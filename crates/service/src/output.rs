use chrono;
use std::{
    sync::mpsc,
    time::SystemTime
};

use trap_engine::io;

fn save_rgb(buf: &Vec<u8>, width: u32, height: u32, path: &str) {
    let rgb = trap_engine::convert::yuyv_to_rgb(buf);
    let result = io::save_image::<io::RgbImage>(
        path,
        rgb,
        width,
        height
    );
}

pub fn output_loop(
    rx: mpsc::Receiver<Vec<u8>>,
    width: u32,
    height: u32,
    path: String
) {
    loop {
        if let Ok(new_frame) = rx.recv() {
            let now: chrono::DateTime<chrono::Local> = SystemTime::now().into();
            let dt_str = now.format("%Y%m%d-%H%M%S-%f");
            let file_path = format!("{}/{}.jpg", path, dt_str);
            save_rgb(&new_frame, width, height, &file_path);
        }
    }
}