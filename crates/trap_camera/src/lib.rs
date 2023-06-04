use nokhwa::{
    Camera as NokhwaCamera,
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType, FrameFormat, Resolution, CameraFormat}
};
use std::str::FromStr;


pub struct Camera {
    camera: NokhwaCamera
}
impl Camera {
    pub fn new(
        w: u32, h: u32, format: &str, frame_rate: u32
    ) -> Self {
        let index = CameraIndex::Index(0);
        let format = CameraFormat::new(Resolution::new(w, h), FrameFormat::from_str(format).unwrap(), frame_rate);
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(format));
        let mut camera = NokhwaCamera::new(index, requested).unwrap();
        Camera { camera }
    }
    pub fn start(&mut self) {
        self.camera.open_stream();
        println!("{:?}", self.camera.frame_format());
    }
    pub fn capture(&mut self) -> Option<Vec<u8>> {
        let raw = self.camera.frame_raw().ok()?;
        Some(raw.into())
    }
}