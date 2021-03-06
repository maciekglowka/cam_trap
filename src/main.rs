use rscam::{Camera, Config, CID_JPEG_COMPRESSION_QUALITY};
use std::fs::File;
use std::io;
use std::cmp::{min, max};

use tokio::sync::mpsc;
use image::{GrayImage, RgbImage};
use serde::Deserialize;
use chrono::Local;
use std::{time, thread};
// use jpeg_decoder::Decoder;

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

// fn decode_jpeg(frame: &[u8]) -> Vec::<u8> {
//     let mut decoder = Decoder::new(frame);
//     let buffer = decoder.decode().expect("Can't decode jpeg!");

//     return buffer
// }


fn grayscale_from_yuyv(buf: &[u8]) -> Vec::<u8> {
    let mut out = Vec::<u8>::new();
    out.reserve_exact(buf.len()/2);

    for x in 0..(buf.len()/2) as usize {
        out.push(buf[x*2]);
    }
    out
}


fn decode_yuyv(buf: &[u8]) -> Vec::<u8> {
    // let buf = &*frame;
    let mut out = Vec::<u8>::new();
    out.reserve_exact(buf.len()*6/4);

    for x in 0..buf.len()/4 {
        let i = 4*x;
        let y0 = buf[i] as isize;
        let u = buf[i+1] as isize;
        let y1 = buf[i+2] as isize;
        let v =  buf[i+3] as isize;

        let r_comp = (351 * (v-128)) >> 8;
        let g_comp = (179 * (v-128) + 86 * (u-128)) >> 8;
        let b_comp = (443 * (u-128)) >> 8;

        let r0 = min(255, max(0, y0 + r_comp)) as u8;
        let g0 = min(255, max(0, y0 - g_comp)) as u8;
        let b0 = min(255, max(0, y0 + b_comp)) as u8;

        let r1 = min(255, max(0, y1 + r_comp)) as u8;
        let g1 = min(255, max(0, y1 - g_comp)) as u8;
        let b1 = min(255, max(0, y1 + b_comp)) as u8;

        // let r0 = min(255, y0 + r_comp) as u8;
        // let g0 = max(0, y0 - g_comp) as u8;
        // let b0 = min(255, y0 + b_comp) as u8;

        // let r1 = min(255, y1 + r_comp) as u8;
        // let g1 = max(0, y1 - g_comp) as u8;
        // let b1 = min(255, y1 + b_comp) as u8;
        // let r0 = (y0 + r_comp) as u8;
        // let g0 = (y0 - g_comp) as u8;
        // let b0 = (y0 + b_comp) as u8;

        // let r1 = (y1 + r_comp) as u8;
        // let g1 = (y1 - g_comp) as u8;
        // let b1 = (y1 + b_comp) as u8;

        out.push(r0);
        out.push(g0);
        out.push(b0);

        out.push(r1);
        out.push(g1);
        out.push(b1);
    }

    out
}

// fn yuv_to_rgb(y: isize, u: isize, v: isize) -> Vec::<u8> {
//     let tr = y + (351 * (v-128)) / 256;
//     let tg = y - (179 * (v-128) + 86 * (u-128)) / 256;
//     let tb = y + (443 * (u-128)) / 256;
//     let r = min(255, max(tr, 0)) as u8;
//     let g = min(255, max(tg, 0)) as u8;
//     let b = min(255, max(tb, 0)) as u8;
//     vec!(r,g,b)
// }

// fn decode_yuyv(frame: rscam::Frame) -> Vec::<u8> {
//     let buf = &*frame;
//     let mut out = Vec::<u8>::new();

//     for x in 0..buf.len()/4 {
//         let i = 4*x;
//         let y0 = buf[i] as isize;
//         let u = buf[i+1] as isize;
//         let y1 = buf[i+2] as isize;
//         let v =  buf[i+3] as isize;

//         out.append(&mut yuv_to_rgb(y0, u, v));
//         out.append(&mut yuv_to_rgb(y1, u, v));
//     }
//     out
// }

fn blur_down(input: &Vec<u8>, width: u32, height: u32, ratio: u32) -> Vec::<u8> {
    let mut v = Vec::<u8>::new();
    v.reserve_exact((height / ratio * width / ratio) as usize);
    let pixels = Pixels::<u8>::new(width as usize, height as usize, input);
    let r2 = max(1,ratio / 2) as isize;

    for y in 0..height/ratio {
        for x in 0..width/ratio {
            let mut value: f32 = 0.0;
            let mut count: f32 = 0.0;
            let px = (x*ratio) as isize;
            let py = (y*ratio) as isize;
            for wy in max(0,py-r2)..min(height as isize, py+r2) {
                for wx in max(0,px-r2)..min(width as isize, px+r2) {
                    count += 1.0;
                    value += pixels.get(wx as usize, wy as usize) as f32;
                }
            }
            v.push((value / count) as u8);
        }
    }
    v
}

// fn to_grayscale(input: &Vec<u8>) -> Vec<u8> {
//     let mut gray = vec![0; input.len()/3];
//     let mut idx = 0;
//     while idx<input.len() {
//         gray[idx/3] = input[idx]/3+input[idx+1]/3+input[idx+2]/3;
//         idx += 3;
//     };
//     return gray
// }

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
    v.reserve_exact((width * height) as usize);
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

fn vec_diff(v0: &Vec<u8>, v1: &Vec<u8>) -> Vec<u8> {
    v0.iter().zip(v1).map(|(a,b)| if a>b {a-b} else {b-a}).collect()
}



fn operate_camera(tx: tokio::sync::mpsc::Sender<rscam::Frame>, settings: Settings) {
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
        let start_time = time::Instant::now();
        let frame = camera.capture().expect("Can't access frame!");

        // let v;
        // let gray;
        // if settings.format == "YUYV" { 
        //     v=decode_yuyv(frame);
        //     gray = to_grayscale(&v);
        // } else {
        //     v = decode_jpeg(frame);
        //     gray = to_grayscale(&v);
        // };
        let elapsed = time::Instant::now() - start_time;

        // if settings.debug {
        //     println!("Frame capture took: {}ms", elapsed.as_millis());
        // }
        tx.blocking_send(frame); //expect("Sendind frame to async thread failed!");
        
        if elapsed < delay {
            thread::sleep(delay - elapsed);
        }
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

    let frame_thresh = dw * dh / settings.frame_diff_div;
    if settings.debug {println!("Thresh sum: {}", frame_thresh)};

    loop {
        if let Ok(result) = tokio::time::timeout(duration, rx.recv()).await {
            let start_time = time::Instant::now();
            let buf = result.unwrap();
            if settings.debug {println!("Received {} from the camera thread", buf.len())};

            //let mut rgb;
            let gray=grayscale_from_yuyv(&*buf);
            // if settings.format == "YUYV" { 
            //     // rgb=decode_yuyv(&*buf);
            //     gray = grayscale_from_yuyv(&*buf);
            // } else {
            //     rgb = decode_jpeg(&*buf);
            //     gray = to_grayscale(&rgb);
            // };

            // let gray = to_grayscale(&buf);
            let blurred = blur_down(&gray, settings.width, settings.height, settings.down_ratio);
            
            let diff = vec_diff(&blurred, &last);
            
            let edges = sobel(&diff, dw, dh, &kernels, settings.threshold);

            let sum = edges.iter().fold(0, |acc, a| acc + *a as u32) / 255;

            if sum > frame_thresh {
                println!("Movement detected. {}", sum);
                let rgb = decode_yuyv(&*buf);
                let img = RgbImage::from_raw(settings.width, settings.height, rgb).expect("Can't create RGB image!");
                let path = format!("{}output_{}_{}.jpg", settings.output_dir, Local::now().format("%Y%m%d_%H_%M_%S"), sum);
                if settings.debug {println!("Saving to: {}", path)};
                img.save(path).expect("Cannot save image file!");

                // let dif_img = GrayImage::from_raw(dw, dh, diff).unwrap();
                // let dif_path = format!("{}output_diff_{}.png", settings.output_dir, Local::now().format("%Y%m%d_%H_%M_%S"));
                // dif_img.save(dif_path).expect("Cannot save image file!");
            } else if settings.debug {
                println!("No movement. {}", sum);
                // let img = GrayImage::from_raw(dw, dh, diff).unwrap();
                // let path = format!("{}output_gray_{}_{}.png", settings.output_dir, Local::now().format("%Y%m%d_%H_%M_%S"), sum);
                // img.save(path).expect("Cannot save image file!");
            }

            last = blurred;
            if settings.debug {
                println!("Frame calculation took: {}ms", (time::Instant::now() - start_time).as_millis());
            }
        } else {
            println!("Timed out");
            camera_reset(&settings.dev_id);
            std::process::exit(1);
        }
    }
}