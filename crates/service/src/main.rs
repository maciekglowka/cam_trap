use std::time::Instant;

use trap_camera;
use trap_engine::{
    io,
    detect::{Pixels, PixelType}
};

fn main() {
    let mut buffer = [&0_u8; 1920 * 1080];
    let mut camera = trap_camera::Camera::new(1920, 1080, "YUYV", 5);
    camera.start();
    let start = Instant::now();
    let frame_0 = camera.capture().unwrap();
    // let frame = camera.capture().unwrap();
    println!("Capture exec time: {:?}", start.elapsed());
    println!("Frame: {}", frame_0.len());
    let start = Instant::now();
    let frame_1 = camera.capture().unwrap();
    // let frame = camera.capture().unwrap();
    println!("Capture exec time: {:?}", start.elapsed());
    // println!("Captured Single Frame of {}", frame.len());
    // let start = Instant::now();
    // let gray = trap_engine::convert::yuyv_to_grayscale(&frame_0);
    // println!("Gray exec time: {:?}", start.elapsed());

    let start = Instant::now();
    // let arr: [u8; 1920 * 1080] = gray.map(|a| *a)
    //     .collect::<Vec<_>>()
    //     .try_into()
    //     .unwrap();
    // buffer[..1920 * 1080].copy_from_slice(&gray.collect());

    // let v0: Vec<u8> = gray.map(|a| *a).collect();
    // // io::save_image::<io::GrayImage>("test.png", v, 1920, 1080);
    let pixels = Pixels::<u8>::new(1920, &frame_0, PixelType::YUYV);
    let v0 = trap_engine::detect::blur_down(&pixels, 1920, 1080, 4);
    println!("Blur exec time: {:?}", start.elapsed());

    let pixels = Pixels::<u8>::new(1920, &frame_1, PixelType::YUYV);
    let v1 = trap_engine::detect::blur_down(&pixels, 1920, 1080, 4);

    // let gray = trap_engine::convert::yuyv_to_grayscale(&frame_1);
    // let v1: Vec<u8> = gray.map(|a| *a).collect();
    // let v1 = trap_engine::detect::blur_down(&v1, 1920, 1080, 4);

    let start = Instant::now();
    // let sum = trap_engine::detect::compare(&frame_0, &frame_1, 1920, 1080, 75);
    let sum = trap_engine::detect::compare(&v0, &v1, 1920 / 4 - 2, 1080 /4 - 2, 15);
    println!("Compare exec time: {:?}, sum: {}", start.elapsed(), sum);
    // io::save_image::<io::GrayImage>("test.png", v, 1920 / 4 - 2, 1080 / 4 - 2);
}