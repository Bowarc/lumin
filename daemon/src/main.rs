#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

#[macro_use]
extern crate log;

mod error;
mod server;

const TARGET_TPS: f32 = 10.;

fn main() {
    println!("Daemon main");
    shared::logger::init(None);

    std::thread::sleep(std::time::Duration::from_secs(5));

    let mut s = server::Server::new();

    let mut last_loop_time: f32 = 0.;
    loop {
        let sleep_time = sleep_until_tps(last_loop_time);
        // println!("Loop {sleep_time:.5}s");

        let start_time = std::time::Instant::now();

        s.update();
        last_loop_time = start_time.elapsed().as_secs_f32();
    }
}

fn sleep_until_tps(dt: f32) -> f32 {
    let sleep_time_s = (1. / TARGET_TPS) - dt;
    // println!("Slepping for {sleep_time_s}s");

    if sleep_time_s.is_sign_negative() {
        // std::time::Duration::from_secs_f32() panics if the input is negative
        println!("skipping as the time is negative ({sleep_time_s})");
        return 0.;
    }

    std::thread::sleep(std::time::Duration::from_secs_f32(sleep_time_s));

    sleep_time_s
}
