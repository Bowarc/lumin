// #![cfg_attr(
//     all(target_os = "windows", not(debug_assertions)),
//     windows_subsystem = "windows"
// )]

#[macro_use]
extern crate log;

mod error;
mod server;
mod wallpaper;
mod window_manager;

const TARGET_TPS: f32 = 10.;

fn main() {
    shared::logger::init(None);
    debug!("Daemon main");
    println!("Daemon main");

    let wm = window_manager::windows::Explorer::new();
    let mut s = server::Server::new();
    let mut w = wallpaper::Wallpaper::new(wm);

    debug!("{:?}", w.screens);
    println!("{:?}", w.screens);

    // w.start_player(
    //     w.screens.get(0).unwrap().clone(),
    //     "D:\\Dev\\Rust\\projects\\lumin\\research\\mpv\\shapes.mp4".into(),
    // )
    // .unwrap();

    // std::thread::sleep(std::time::Duration::from_secs(10));

    let mut loop_helper = spin_sleep::LoopHelper::builder()
        .report_interval_s(0.5) // report every half a second
        .build_with_target_rate(TARGET_TPS); // limit to 250 FPS if possible

    let mut last_frame_time = 0.;
    loop {
        let frame_start_time = std::time::Instant::now();
        loop_helper.loop_start();

        // Update
        s.update(&mut w, (last_frame_time, TARGET_TPS));

        loop_helper.loop_sleep();
        last_frame_time = frame_start_time.elapsed().as_secs_f32();
        println!("{}", 1. / last_frame_time);
    }
}

fn sleep_until_tps(dt: f32, last_frame_time: f32) -> f32 {
    let target_frame_time = 1. / TARGET_TPS;
    let overshoot = (last_frame_time - target_frame_time).clamp(0., 1.);
    debug!("Overshoot: {overshoot}");
    //  0.1
    let sleep_time_s = (1. / TARGET_TPS) - dt;
    // debug!("Slepping for {sleep_time_s}s");

    if sleep_time_s.is_sign_negative() {
        // std::time::Duration::from_secs_f32() panics if the input is negative
        debug!("skipping as the time is negative ({sleep_time_s})");
        return 0.;
    }

    std::thread::sleep(std::time::Duration::from_secs_f32(sleep_time_s));

    sleep_time_s
}
