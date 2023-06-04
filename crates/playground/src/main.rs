use std::time::Instant;

// use trap_camera;
use trap_engine;

fn main() {
    let frame_0 = trap_engine::io::read_image("assets/skansen_00.png").unwrap();
    let frame_1 = trap_engine::io::read_image("assets/skansen_01.png").unwrap();
    let start = Instant::now();
    for _ in 0..1 {
        // trap_engine::detect::compare(&frame_0, &frame_1, 1920, 1080);
        // trap_engine::detect::compare_old(&frame_0, &frame_1, 1920, 1080);
        let blurred_0 = trap_engine::detect::blur_down(&frame_0, 1920, 1080, 4);
        let blurred_1 = trap_engine::detect::blur_down(&frame_1, 1920, 1080, 4);
        // println!("{}", blurred.len());
        // let img = trap_engine::detect::get_diff_edges(&frame_0, &frame_1, 1920, 1080, 120);
        let img = trap_engine::detect::get_diff_edges(&blurred_0, &blurred_1, 1920/4-2, 1080/4-2, 120);
        trap_engine::io::save_img("assets/output.png", img, 1920/4-4, 1080/4-4);
        // trap_engine::io::save_img("assets/output.png", img, 1920-2, 1080-2);
    }
    println!("Exec time: {:?}", start.elapsed());
}