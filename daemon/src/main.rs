#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

#[macro_use]
extern crate log;

mod error;
mod server;
mod wallpaper;
mod window_manager;

const TARGET_TPS: f32 = 10.;

// should only be used to replace a panic!
static EXIT_REQUESTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn main() {
    shared::logger::init(Some("daemon_log.txt"));
    debug!("Daemon start");

    let wm = window_manager::windows::Explorer::new().unwrap();
    let mut s = server::Server::new();
    let mut w = wallpaper::Wallpaper::new(wm);

    debug!("{:?}", w.screens);

    let mut loop_helper = spin_sleep::LoopHelper::builder()
        .report_interval_s(0.5) // report every half a second
        .build_with_target_rate(TARGET_TPS); // limit FPS if possible

    let mut last_frame_time = 0.;
    loop {
        // Start

        let frame_start_time = std::time::Instant::now();
        loop_helper.loop_start();

        // Update

        s.update(&mut w, (last_frame_time, TARGET_TPS));

        // Check of critical error

        // This is very ugly but so is the project structure soo.. meh, i'll make a v2 in a couple of years
        if EXIT_REQUESTED.load(std::sync::atomic::Ordering::Relaxed) {
            clean_exit(&mut s, &mut w)
        }

        // End

        loop_helper.loop_sleep();
        last_frame_time = frame_start_time.elapsed().as_secs_f32();
        println!("{}", 1. / last_frame_time);
    }
}

fn clean_exit(s: &mut server::Server, w: &mut wallpaper::Wallpaper) -> ! {
    // clean every background process

    w.on_exit();

    if let Some(client) = &mut s.client {
        if let Err(e) = client.socket.send(shared::networking::DaemonMessage::Error("The daemon ran into an critical error and have to restart. please check the log file for more info".to_string())){
            error!("{e}")
        }

        if let Err(e) = client.socket.send(shared::networking::DaemonMessage::Error(
            "Well, there is no log file for the moment, rip".to_string(),
        )) {
            error!("{e}")
        }
    }

    // Wait for the socket to send messages
    std::thread::sleep(std::time::Duration::from_secs(1));

    std::process::exit(1)
}
