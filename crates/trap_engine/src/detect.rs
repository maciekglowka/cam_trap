use std::{
    cmp::{min, max},
    ops::Index
};

#[derive(Clone, Copy)]
pub enum PixelType {
    YUYV,
    Gray
}

pub struct Pixels<'a, T: std::marker::Copy> {
    width: usize,
    data: &'a [T],
    index_multi: usize
}
impl<'a, T: std::marker::Copy> Pixels<'a, T> {
    pub fn new(width: usize, data: &[T], pixel_type: PixelType) -> Pixels<T> {
        let index_multi = match pixel_type {
            PixelType::YUYV => 2,
            PixelType::Gray => 1
        };
        Pixels{width, data, index_multi}
    }
    pub fn get(self: &Self, x: usize, y:usize) -> T {
        self.data[self.index_multi * (x + y * self.width)]
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

const KERNELS: [Kernel::<i16>; 2] = [
    Kernel::<i16>{data: [-1,0,1,-2,0,2,-1,0,1]},
    Kernel::<i16>{data: [1,2,1,0,0,0,-1,-2,-1]}
];

pub fn get_diff_edges(
    a: &Vec<u8>,
    b: &Vec<u8>,
    w: u32,
    h: u32,
    thresh: i16
) -> Vec<u8> {
    let diff = vec_diff(a, b);
    sobel(&Pixels::new(w as usize, &diff, PixelType::Gray), w, h, thresh)
}

pub fn compare(
    a: &Vec<u8>,
    b: &Vec<u8>,
    w: u32,
    h: u32,
    thresh: i16
) -> u32 {
    let diff = vec_diff(a, b);
    edge_sum(&Pixels::new(w as usize, &diff, PixelType::Gray), w, h, thresh)
}

pub fn blur_down(pixels: &Pixels<u8>, width: u32, height: u32, ratio: u32) -> Vec<u8> {
    let mut v = Vec::<u8>::with_capacity((height / ratio * width / ratio) as usize);
    let r2 = max(1, ratio / 2);
    let count = (ratio * ratio) as f32;

    for y in 1..height/ratio - 1 {
        for x in 1..width/ratio - 1 {
            let mut value: f32 = 0.0;
            let px = x*ratio;
            let py = y*ratio;
            for wy in py-r2..py+r2 {
                for wx in px-r2..px+r2 {
                    value += pixels.get(wx as usize, wy as usize) as f32;
                }
            }
            v.push((value / count) as u8);
        }
    }
    v
}

fn edge_sum(pixels: &Pixels<u8>, width: u32, height: u32, threshold: i16) -> u32 {
    // run sobel and count the edge pixels above the thresh
    let mut sum = 0;

    for y in 1..(height - 1) as isize{
        for x in 1..(width - 1) as isize{
            let value = sobel_window(pixels, x as usize, y as usize);
            if value.abs() > threshold {sum += 1};
        }
    }
    sum
}

fn sobel(pixels: &Pixels<u8>, width: u32, height: u32, threshold: i16) -> Vec<u8> {
    let mut v = Vec::<u8>::with_capacity((width * height) as usize);

    for y in 1..(height-1) as isize{
        for x in 1..(width-1) as isize{
            let value = sobel_window(&pixels, x as usize, y as usize);
            if value.abs() < threshold { v.push(0)} else {v.push(255)};
        }
    }
    v
}

#[inline(always)]
fn sobel_window(pixels: &Pixels<u8>, x: usize, y: usize) -> i16 {
    let mut value = 0;
    for k in KERNELS.iter() {
        for wy in y-1..y+1 {
            for wx in x-1..x+1 {
                value += pixels.get(wx, wy) as i16 * k.get((x-wx+1) as usize, (y-wy+1) as usize);
            }
        }
    }
    value
}

fn vec_diff(v0: &Vec<u8>, v1: &Vec<u8>) -> Vec<u8> {
    v0.iter().zip(v1).map(|(a,b)| if a>b {a-b} else {b-a}).collect()
}
