// #![cfg_attr(
//     all(target_os = "windows", not(debug_assertions)),
//     windows_subsystem = "windows"
// )]
// #![feature(stmt_expr_attributes)]

#[macro_use]
extern crate log;

mod app;
mod error;
mod id;
mod logger;
mod monitor;
mod threading;
mod tray;
mod ui;
mod wallpaper;
mod window_manager;
mod ytdl;

lazy_static::lazy_static! {
    static ref APP: std::sync::Arc<std::sync::Mutex<app::App>> = std::sync::Arc::new(std::sync::Mutex::new(app::App::default()));
}

fn main() {
    logger::init(Some("lumin.log"));

    // let mut dl_state = ytdl::DownloadState::default();

    // dl_state
    //     .start_download(&ytdl::DownloadConfig {
    //         url: "https://www.youtube.com/watch?v=_HpmJr__7Jk".into(),
    //         file_name: "pando.mp4".into(),
    //     })
    //     .unwrap();

    // loop {
    //     std::thread::sleep(std::time::Duration::from_secs(1));
    //     dl_state.update();
    //     dbg!(dl_state.get_value());
    //     if let ytdl::DownloadState::Done = dl_state {
    //         break;
    //     }
    // }

    menu_test()
}

fn workerw_tests() {
    use std::io::BufRead;
    debug!("Starting the fetcher. . .");

    const FETCHER_EXE_NAME: &str = "workerw_fetcher.exe";
    let mut path = std::env::current_exe().unwrap();

    path.set_file_name(FETCHER_EXE_NAME);

    let mut child = std::process::Command::new(path)
        .stderr(std::process::Stdio::null())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    debug!("{child:?}");

    let status_opt = loop {
        println!("Fetcher is not done yet");
        match child.try_wait() {
            Ok(Some(status)) => {
                println!("exited with: {status}");
                break Some(status);
            }
            Ok(None) => {
                println!("status not ready yet, let's really wait");
                let res = child.wait();
                println!("result: {res:?}");
            }
            Err(e) => {
                println!("error attempting to wait: {e}");
                break None;
            }
        }
    };

    if let Some(status) = status_opt {
        if status.success() {
            let mut child_out = std::io::BufReader::new(child.stdout.as_mut().unwrap());
            let mut line = String::new();

            child_out.read_line(&mut line).unwrap();
            println!("fetcher wrote '{}' before dying", line);
        } else {
            // when the process fail, looking at the stdout (and reading it like the above is doing)
            // does not crash the current program as it's like channels and the client is only reading what it receied,
            // client is not atempting to call remote process
            println!("fetcher failled");
        }
    } else {
        println!("Could not get exit code")
    }
    // loop {

    //     // child_in.write("ls\n".as_bytes()).unwrap();
    // }
}

fn menu_test() {
    run_ui();
    loop {
        let command = APP.lock().unwrap().tray_menu.update(); // tray::Command implements Copy
        match command {
            tray::Command::Open => run_ui(),
            tray::Command::Exit => {
                println!("Exiting main loop");
                break;
            }
            tray::Command::None => {}
        }
    }
    APP.lock().unwrap().on_exit();
}

fn run_ui() {
    eframe::run_native(
        "Lumin",
        eframe::NativeOptions {
            // initial_window_size: Some(eframe::egui::vec2(800.0, 600.0)), /*x800y450 is 16:9*/
            // resizable: false,
            // centered: true,
            // vsync: true,
            // decorated: false,
            // transparent: true,
            // always_on_top: true,
            follow_system_theme: true,
            run_and_return: true,
            centered: true,
            vsync: true,
            viewport: eframe::egui::ViewportBuilder::default()
                .with_inner_size(eframe::egui::vec2(800.0, 600.0))
                .with_decorations(false)
                .with_transparent(true)
                .with_resizable(false)
                .with_title("Lumin"),

            // default_theme: eframe::Theme::Dark,
            ..Default::default()
        },
        Box::new(|cc| Box::<ui::Ui>::new(ui::Ui::new(cc))),
    )
    .unwrap();
}
