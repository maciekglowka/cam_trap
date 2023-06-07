use std::sync::mpsc;

use trap_camera;

use super::config::CameraSettings;

pub fn camera_loop(
    tx: mpsc::Sender<Vec<u8>>,
    settings: CameraSettings
) {
    let mut camera = trap_camera::Camera::new(
        settings.width,
        settings.height,
        &settings.format,
        settings.frame_rate
    );
    camera.start();
    loop {
        if let Some(frame) = camera.capture() {
            tx.send(frame);
        }
    }
}