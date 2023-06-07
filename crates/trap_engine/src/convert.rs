use std::cmp::{min, max};

pub fn yuyv_to_rgb(buf: &[u8]) -> Vec<u8> {
    // old code, consider refactoring
    let mut out = Vec::with_capacity(buf.len() * 6/4);

    for x in 0..buf.len() / 4 {
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

        out.push(r0);
        out.push(g0);
        out.push(b0);

        out.push(r1);
        out.push(g1);
        out.push(b1);
    }
    out
}