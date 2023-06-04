// pub fn rgb_to_grayscale(buf: &Vec<u8>) -> Vec<u8> {
//     buf.chunks(3)
//         .map(|p| p[0] / 3 + p[1] / 3 + p[2] / 3)
//         .collect()
// }

pub fn yuyv_to_grayscale<'a>(buf: &'a[u8]) -> impl Iterator<Item=&'a u8> {
    buf.iter().step_by(2)
}