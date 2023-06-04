// use std::io::Cursor;
use image::{
    io::Reader as ImageReader,
    ImageBuffer,
    Luma,
    Rgb,
    Pixel
};
use std::error::Error;

pub type RgbImage = Rgb<u8>;
pub type GrayImage = Luma<u8>;

pub fn read_image(path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let img = ImageReader::open(path)?.decode()?;
    let luma = img.to_luma8();
    let buf = luma.as_raw().to_owned();
    Ok(buf)
}

pub fn save_image<P: Pixel<Subpixel = u8> + image::PixelWithColorType> (path: &str, buf: Vec<u8>, w: u32, h: u32) -> Result<(), image::ImageError> {
    let img: ImageBuffer<P, _> = ImageBuffer::from_raw(w, h, buf).unwrap();
    img.save(path)
}
