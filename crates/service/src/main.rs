use std::{
    sync::mpsc,
    thread,
    time::Instant
};

use trap_engine::detect::{Pixels, PixelType};

mod camera;
mod config;
mod output;

const SETTINGS_PATH: &str = "settings.toml";

fn main() {
    let settings = config::load_settings(SETTINGS_PATH);
    let (camera_tx, camera_rx) = mpsc::channel();
    let (file_tx, file_rx) = mpsc::sync_channel(settings.output_buffer_size);

    let camera_settings = settings.camera.clone();
    thread::spawn(move || {
        camera::camera_loop(camera_tx, camera_settings);
    });

    thread::spawn(move || {
        output::output_loop(
            file_rx,
            settings.camera.width,
            settings.camera.height,
            settings.output_path.clone(),
        );
    });

    let mut last_frame = None;
    let pixel_type = match settings.camera.format.as_str() {
        "YUYV" => PixelType::YUYV,
        _ => panic!("Pixel format not supported!")
    };

    loop {
        if let Ok(new_frame) = camera_rx.recv() {
            println!("Received {} bytes", new_frame.len());
            let new_blurred = trap_engine::detect::blur_down(
                &Pixels::<u8>::new(settings.camera.width as usize, &new_frame, pixel_type),
                settings.camera.width,
                settings.camera.height,
                settings.downsample_ratio
            );
            if let Some(last_frame) = last_frame {
                let sum = trap_engine::detect::compare(
                    &last_frame,
                    &new_blurred,
                    settings.camera.width / settings.downsample_ratio - (settings.downsample_ratio / 2),
                    settings.camera.height / settings.downsample_ratio - (settings.downsample_ratio / 2),
                    settings.sobel_thresh
                );
                println!("Sum: {}", sum);
                if sum >= settings.edge_thresh {
                    file_tx.try_send(new_frame);
                    // thread::spawn(move || {
                    //     output::save_rgb(
                    //         &new_frame,
                    //         settings.camera.width,
                    //         settings.camera.height
                    //     );
                    // });
                }
            }
            last_frame = Some(new_blurred);
        }
    }
}
