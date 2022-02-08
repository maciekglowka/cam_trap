use rscam::{Camera, Config, CID_JPEG_COMPRESSION_QUALITY};
use std::fs::File;
use std::io;
use std::cmp::{min, max};

use tokio::sync::mpsc;
use image::{GrayImage, RgbImage};
use serde::Deserialize;
use chrono::Local;
use std::{time, thread};
use jpeg_decoder::Decoder;

#[derive(Clone, Debug, Deserialize)]
struct Settings {
    width: u32,
    height: u32,
    device: String,
    dev_id: String,
    format: String,
    output_dir: String,
    threshold: i16,
    frame_diff_div: u32,
    delay: u64,
    debug: bool,
    timeout: u64,
    down_ratio: u32,
    interval: (u32, u32)
}

struct Pixels<'a, T: std::marker::Copy> {
    width: usize,
    height: usize,
    data: &'a Vec<T>
}

impl<'a, T: std::marker::Copy> Pixels<'a, T> {
    fn new(width: usize, height:usize, data: &Vec<T>) -> Pixels<T> {
        Pixels{width: width, height: height, data: data}
    }
    fn get(self: &Self, x: usize, y:usize) -> T {
        self.data[x + y * self.width]
    }
}

struct Kernel<T: std::marker::Copy> {
    data: [T; 9]
}

impl<T: std::marker::Copy> Kernel<T> {
    fn get(self: &Self, x: usize, y:usize) -> T {
        self.data[x + y * 3]
    }
}

const k0: Kernel::<i16> = Kernel::<i16>{data: [-1,0,1,-2,0,2,-1,0,1]};
const k1: Kernel::<i16> = Kernel::<i16>{data: [1,2,1,0,0,0,-1,-2,-1]};

fn decode_jpeg(frame: rscam::Frame) -> Vec::<u8> {
    let mut decoder = Decoder::new(&*frame);
    let buffer = decoder.decode().expect("Can't decode jpeg!");

    return buffer
}

fn blur_down(input: &Vec<u8>, width: u32, height: u32, ratio: u32) -> Vec::<u8> {
    let mut v = Vec::<u8>::new();
    let pixels = Pixels::<u8>::new(width as usize, height as usize, input);

    for y in 0..height/ratio {
        for x in 0..width/ratio {
            let mut value: f32 = 0.0;
            let mut count: f32 = 0.0;
            let px = (x*ratio) as isize;
            let py = (y*ratio) as isize;
            for wy in max(0,py-1)..min(height as isize, py+1) {
                for wx in max(0,px-1)..min(width as isize, px+1) {
                    count += 1.0;
                    value += pixels.get(wx as usize,wy as usize) as f32;
                }
            }
            v.push((value / count) as u8);
        }
    }
    v
}

fn to_grayscale(input: &Vec<u8>) -> Vec<u8> {
    let mut gray = vec![0; input.len()/3];
    let mut idx = 0;
    while idx<input.len() {
        gray[idx/3] = input[idx]/3+input[idx+1]/3+input[idx+2]/3;
        idx += 3;
    };
    return gray
}

// fn blur(input: &Vec<u8>, width: u32, height: u32) -> Vec::<u8> {
//     let mut v = Vec::<u8>::new();
//     let pixels = Pixels::<u8>::new(width as usize, height as usize, input);

//     for y in 0..height as isize{
//         for x in 0..width as isize{
//             let mut value: f32 = 0.0;
//             let mut count: f32 = 0.0;
//             for wy in max(0,y-1)..min(height as isize,y+1) {
//                 for wx in max(0,x-1)..min(width as isize,x+1) {
//                     count += 1.0;
//                     value += pixels.get(wx as usize,wy as usize) as f32;
//                 }
//             }
//             v.push((value / count) as u8);
//         }
//     }
//     return v
// }

fn sobel(input: &Vec<u8>, width: u32, height: u32, kernels: &Vec<Kernel<i16>>, threshold: i16) -> Vec::<u8> {
    let mut v = Vec::<u8>::new();
    let pixels = Pixels::<u8>::new(width as usize, height as usize, input);

    for y in 0..height as isize{
        for x in 0..width as isize{
            let mut value: i16 = 0;

             for k in kernels.iter() {
                 for wy in max(0,y-1)..min(height as isize,y+1) {
                    for wx in max(0,x-1)..min(width as isize,x+1) {
                         value += pixels.get((wx) as usize, (wy) as usize) as i16 * k.get((x-wx+1) as usize, (y-wy+1) as usize);
                     }
                 }
             }
             if value.abs() < threshold { v.push(0)} else {v.push(255)};
        }
    }
    return v
}



fn operate_camera(tx: tokio::sync::mpsc::Sender<Vec<u8>>, settings: Settings) {
    let mut camera = Camera::new(&settings.device).unwrap();
    if settings.format == "JPEG" {
        camera.set_control(CID_JPEG_COMPRESSION_QUALITY, &95).expect("Camera setting failed!");
    }
    
    camera.start(&Config {
        interval: settings.interval,
        resolution: (settings.width, settings.height),
        format: settings.format.as_bytes(),
        ..Default::default()
    }).unwrap();
    println!("Got camera");

    let delay = time::Duration::from_millis(settings.delay);

    loop {
        let frame = camera.capture().expect("Can't access frame!");
        let v = decode_jpeg(frame);
        tx.blocking_send(v);
        thread::sleep(delay);
    }
}

fn camera_reset(dev_id: &str) {
    if dev_id != "" {
        println!("Restarting camera");   
        let cmd_result = std::process::Command::new("usbreset").arg(dev_id).output().expect("Failed to reset!");
        println!("{:?}", std::str::from_utf8(&cmd_result.stdout));
    } else {
        println!("No dev id specified");
    }
}

#[tokio::main]
async fn main() {
    println!("Starting at {}", Local::now().format("%Y-%m-%d %H:%M:%S"));
    let file = File::open("settings.json").expect("Settings file not found!");
    let reader = io::BufReader::new(file);

    let settings: Settings = serde_json::from_reader(reader).expect("Settings cannot be parsed!");

    println!("{:?}", settings);

    camera_reset(&settings.dev_id);

    let duration = time::Duration::from_millis(settings.timeout);
    let (tx, mut rx) = mpsc::channel(100);

    let camera_settings = settings.clone();
    let cam_thread = thread::spawn(move || {
        operate_camera(tx, camera_settings);
    });

    let mut last: Vec::<u8> = vec!(0; (settings.width * settings.height) as usize);
    let kernels = vec!(k0, k1);

    let dw = settings.width/settings.down_ratio;
    let dh = settings.height/settings.down_ratio;

    let frame_thresh = (dw * dh / settings.frame_diff_div) as i16;
    if settings.debug {println!("Thresh sum: {}", frame_thresh)};

    loop {
        if let Ok(result) = tokio::time::timeout(duration, rx.recv()).await {
            let buf = result.unwrap();
            if settings.debug {println!("Received {} from the camera thread", buf.len())};
            let gray = to_grayscale(&buf);
            // let blurred = blur(&gray, settings.width, settings.height);
            let blurred = blur_down(&gray, settings.width, settings.height, settings.down_ratio);
            let cur = sobel(&blurred, dw, dh, &kernels, settings.threshold);

            let mut sum = 0;
            let mut diff = Vec::new();

            for idx in 0..cur.len() {
                if cur[idx] != last[idx] {
                    sum+=1;
                    diff.push(255)
                } else {
                    diff.push(0)
                }
            }
            if sum > frame_thresh {
                println!("Movement detected. {}", sum);
                let img = RgbImage::from_raw(settings.width, settings.height, buf).expect("Can't create RGB image!");
                let path = format!("{}output_{}.png", settings.output_dir, Local::now().format("%Y%m%d_%H_%M_%S"));
                if settings.debug {println!("Saving to: {}", path)};
                img.save(path).expect("Cannot save image file!");

                // let dif_img = GrayImage::from_raw(dw, dh, diff).unwrap();
                // let dif_path = format!("{}output_diff_{}.png", settings.output_dir, Local::now().format("%Y%m%d_%H_%M_%S"));
                // dif_img.save(dif_path).expect("Cannot save image file!");
            } else if settings.debug {
                println!("No movement. {}", sum);
                // let img = GrayImage::from_raw(dw, dh, diff).unwrap();
                // let path = format!("{}output_sobel.png", settings.output_dir);
                // img.save(path).expect("Cannot save image file!");
            }

            last = cur;
        } else {
            println!("Timed out");
            camera_reset(&settings.dev_id);
            std::process::exit(1);
        }
    }
}