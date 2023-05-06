#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

#[macro_use]
extern crate log;

mod animations;
mod app;
mod dvar_cache;
mod error;
mod ui;
mod utils;

fn main() {
    shared::logger::init(None);

    workerw_tests();

    // let options = eframe::NativeOptions {
    //     initial_window_size: Some(eframe::egui::vec2(800.0, 600.0)), /*x800y450 is 16:9*/
    //     resizable: false,
    //     centered: true,
    //     vsync: true,
    //     decorated: false,
    //     transparent: true,
    //     // always_on_top: true,
    //     default_theme: eframe::Theme::Dark,

    //     ..Default::default()
    // };

    // eframe::run_native(
    //     "Lumin client",
    //     options,
    //     Box::new(|cc| Box::<ui::Ui>::new(ui::Ui::new(cc))),
    // )
    // .unwrap();
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

    let exit_status = child.wait().unwrap();

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
