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

    let options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(800.0, 600.0)), /*x800y450 is 16:9*/
        resizable: false,
        centered: true,
        vsync: true,
        decorated: false,
        transparent: true,
        // always_on_top: true,
        default_theme: eframe::Theme::Dark,

        ..Default::default()
    };

    eframe::run_native(
        "Lumin client",
        options,
        Box::new(|cc| Box::<ui::Ui>::new(ui::Ui::new(cc))),
    )
    .unwrap();
}
