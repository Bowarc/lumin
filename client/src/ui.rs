use eframe::{egui, emath, epaint};

#[rustfmt::skip]
const VIDEO_FILE_EXTENSIONS: &[&str] = &[
    "mp4", "MP4",
    "mov", "MOV",
    "webm", "WEBM",
    "mkv", "MKV"
];

const TITLE_BAR_HEIGHT: f32 = 32.0;

pub enum BackgroundIdeaActivationState {
    NotConnected,
    Running { id: crate::id::ID },
}

struct BackgroundIdea {
    monitor_index: usize,
    content_path: String,
    activation_state: crate::app::state::State<BackgroundIdeaActivationState>,
}

pub struct Ui {
    backgrounds: Vec<BackgroundIdea>,
    notify: egui_notify::Toasts,
    dl_cfg: crate::ytdl::DownloadConfig,
}

/// Normal functions
impl Ui {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        use egui::{
            FontFamily::{Monospace, Proportional},
            FontId, TextStyle,
        };

        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles = [
            (TextStyle::Heading, FontId::new(25.0, Proportional)),
            (TextStyle::Body, FontId::new(16.0, Proportional)),
            (TextStyle::Monospace, FontId::new(16.0, Monospace)),
            (TextStyle::Button, FontId::new(16.0, Proportional)),
            (TextStyle::Small, FontId::new(8.0, Proportional)),
        ]
        .into();
        cc.egui_ctx.set_style(style);

        // populate the ui background list to keep displaying it event after the app is closed and re-opened
        let app = crate::APP.lock().unwrap();

        let mut backgrounds = vec![];

        for running_bg in app.wallpaper.players.iter() {
            let mut activation_state =
                crate::app::state::State::<BackgroundIdeaActivationState>::default();
            activation_state.set_connected(running_bg.id);

            backgrounds.push(BackgroundIdea {
                monitor_index: app
                    .wallpaper
                    .screens
                    .iter()
                    .position(|s| s == &running_bg.monitor)
                    .unwrap(),
                content_path: running_bg
                    .content_path
                    .as_path()
                    .display()
                    .to_string()
                    .replace("\\\\?\\", ""),
                activation_state,
            })
        }

        Self {
            backgrounds,
            notify: egui_notify::Toasts::default()
                .with_margin(egui::vec2(0., TITLE_BAR_HEIGHT + 4.)), // + margin
            dl_cfg: crate::ytdl::DownloadConfig::default(),
        }
    }
}

/// Interface related functions
impl Ui {
    fn render_title_bar(
        &mut self,
        ui: &mut egui::Ui,
        frame: &mut eframe::Frame,
        title_bar_rect: eframe::epaint::Rect,
        title: &str,
    ) {
        let painter = ui.painter();

        let title_bar_response = ui.interact(
            title_bar_rect,
            egui::Id::new("title_bar"),
            egui::Sense::click(),
        );

        // Paint the title:
        painter.text(
            title_bar_rect.center(),
            emath::Align2::CENTER_CENTER,
            title,
            epaint::FontId::proportional(20.0),
            ui.style().visuals.text_color(),
        );

        // Paint the line under the title:
        painter.line_segment(
            [
                title_bar_rect.left_bottom() + epaint::vec2(1.0, 0.0),
                title_bar_rect.right_bottom() + epaint::vec2(-1.0, 0.0),
            ],
            ui.visuals().widgets.noninteractive.bg_stroke,
        );

        // Interact with the title bar (drag to move window):
        if title_bar_response.double_clicked() {
            // frame.set_maximized(!frame.info().window_info.maximized);
        } else if title_bar_response.is_pointer_button_down_on() {
            frame.drag_window();
        }

        // Show toggle button for light/dark mode
        ui.allocate_ui_at_rect(title_bar_rect, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.visuals_mut().button_frame = false;
                ui.add_space(8.0);
                egui::widgets::global_dark_light_mode_switch(ui);
            });
        });

        // Show some close/maximize/minimize buttons for the native window.
        ui.allocate_ui_at_rect(title_bar_rect, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.visuals_mut().button_frame = false;
                ui.add_space(8.0);

                let button_height = 12.0;

                if ui
                    .add(egui::Button::new(
                        egui::RichText::new("❌").size(button_height),
                    ))
                    .on_hover_text("Close the window")
                    .clicked()
                {
                    frame.close();
                }

                let (hover_text, clicked_state) = if frame.info().window_info.maximized {
                    ("Restore window", false)
                } else {
                    ("Maximize window", true)
                };

                if ui
                    .add(egui::Button::new(
                        egui::RichText::new("🗗").size(button_height),
                    ))
                    .on_hover_text(hover_text)
                    .clicked()
                {
                    frame.set_maximized(clicked_state);
                }

                if ui
                    .add(egui::Button::new(
                        egui::RichText::new("🗕").size(button_height),
                    ))
                    .on_hover_text("Minimize the window")
                    .clicked()
                {
                    frame.set_minimized(true);
                }
            });
        });
    }

    fn render_backgrounds(
        &mut self,
        ui: &mut egui::Ui,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        _content_rect: eframe::epaint::Rect,
    ) {
        let mut app = crate::APP.lock().unwrap();

        ui.vertical_centered(|ui| {
            if ui
                .button(egui::RichText::new("New background").size(20.))
                .clicked()
            {
                if self.backgrounds.is_empty() {
                    self.backgrounds.push(BackgroundIdea {
                        monitor_index: 0,
                        content_path: "".into(),
                        activation_state:
                            crate::app::state::State::<BackgroundIdeaActivationState>::default(),
                    });
                } else {
                    self.notify
                        .warning("Only one animated background is possible for now");
                }
            }
        });

        let mut index = 0;

        while index < self.backgrounds.len() {
            let mut deleted = false;

            let bg = self.backgrounds.get_mut(index).unwrap();
            ui.separator();
            // first line, display background index and status

            // Display and select monitor
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_source(index)
                    .selected_text(format!("{:?}", bg.monitor_index))
                    .width(200.)
                    .show_ui(ui, |ui| {
                        // if let Ok(monitors) = app.wallpaper.wm.get_screen_list() {
                        let monitors = app.wallpaper.wm.get_screen_list();
                        for (m_i, monitor) in monitors.iter().enumerate() {
                            ui.selectable_value(
                                &mut bg.monitor_index,
                                m_i,
                                format!(
                                    "{}\nx{}y{}, w{}h{}",
                                    monitor.name,
                                    monitor.position.0,
                                    monitor.position.1,
                                    monitor.size.0,
                                    monitor.size.1,
                                ),
                            );
                        }
                    });
                ui.label({
                    // if let Ok(monitors) = Ok(app.wallpaper.wm.get_screen_list()) {
                    let monitors = app.wallpaper.wm.get_screen_list();
                    let selected_monitor = monitors.get(bg.monitor_index).unwrap();

                    format!(
                        "{}, {}, size: w{} h{}",
                        selected_monitor.name,
                        selected_monitor.direction(),
                        selected_monitor.size.0,
                        selected_monitor.size.1
                    )
                    // } else {
                    //     let txt = "Unable to retreive monitor info from daemon";
                    //     // self.notify.warning(txt);
                    //     String::from(txt)
                    // },
                    // );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    // Can maybe be swapped to bg.status

                    ui.add(egui::Label::new(egui::WidgetText::RichText(
                        egui::RichText::new(bg.activation_state.text.clone())
                            .color(bg.activation_state.color)
                            .text_style(egui::TextStyle::Monospace),
                    )));

                    ui.label("Status: ")
                });
            });
            // Display and select content
            ui.horizontal(|ui| {
                ui.label("Content:");
                ui.text_edit_singleline(&mut bg.content_path);
                if ui.button("Open").clicked() {
                    let dll_file = rfd::AsyncFileDialog::new()
                        .add_filter("Videos", VIDEO_FILE_EXTENSIONS)
                        .set_directory(std::env::current_dir().unwrap())
                        .pick_file();

                    let path = futures::executor::block_on(dll_file);

                    if let Some(..) = path {
                        // self.dll_path_button_text = path.as_ref().unwrap().file_name();
                        bg.content_path = path
                            .unwrap()
                            .path()
                            .as_os_str()
                            .to_str()
                            .unwrap()
                            .to_string();
                    }
                }
            });
            // Last line, send and remove the background
            ui.horizontal(|ui| {
                if ui.button("Send").clicked() {
                    let id = match bg.activation_state.inner {
                        BackgroundIdeaActivationState::NotConnected => None,
                        BackgroundIdeaActivationState::Running { id } => Some(id),
                    };

                    match app.update_bg(id, bg.monitor_index, bg.content_path.clone()) {
                        Ok((id, monitor, content)) => {
                            self.notify.success(format!(
                                "Updating background . .\nScreen: {monitor:?}\nContent: {}",
                                content
                                    .as_path()
                                    .display()
                                    .to_string()
                                    .replace("\\\\?\\", "")
                            ));

                            bg.activation_state.set_connected(id)
                        }
                        Err(e) => {
                            self.notify
                                .error(format!("Could not create background\n{e}"));
                        }
                    };
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    if ui.button("delete").clicked() {
                        deleted = true;
                    }
                });
            });
            ui.separator();

            if deleted {
                if let BackgroundIdeaActivationState::Running { id } = bg.activation_state.inner {
                    if let Err(e) = app.remove_bg(id) {
                        error!("Could not delete background ({id:?}) Index: {index} due to: {e}")
                    } else {
                        // Connected
                        debug!("Removing background {index} (Connected)");
                        self.backgrounds.remove(index);
                    }
                } else {
                    // Client side only
                    debug!("Removing background {index} (Client side only)");

                    self.backgrounds.remove(index);
                }
            } else {
                index += 1;
            }
        }
    }
    fn render_downloader(
        &mut self,
        ui: &mut egui::Ui,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        _content_rect: eframe::epaint::Rect,
    ) {
        let mut app = crate::APP.lock().unwrap();
        app.downloader.update();

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Output file name:");
            ui.add_space(10.);
            ui.add(
                egui::widgets::TextEdit::singleline(&mut self.dl_cfg.file_name)
                    .desired_width(150.)
                    .horizontal_align(eframe::egui::Align::Max),
            );
            ui.label(".mp4")
        });
        ui.add_space(20.);
        ui.horizontal(|ui| {
            ui.label("Youtube link:");
            ui.add(
                egui::widgets::TextEdit::singleline(&mut self.dl_cfg.url)
                    .hint_text("Video link or video ID"),
            );
        });
        ui.add_space(20.);

        ui.horizontal(|ui| {
            if app.downloader.is_running() {
                ui.add(
                    eframe::egui::widgets::ProgressBar::new(
                        app.downloader.get_value().unwrap() / 100.,
                    )
                    .animate(true)
                    .show_percentage()
                    .desired_width(200.),
                );
            } else {
                ui.add_space(450.);
                if ui
                    .add(eframe::egui::widgets::Button::new("Start").min_size((100., 100.).into()))
                    .clicked()
                {
                    debug!(
                        "Starting dl with options: {:?}\n{:?}",
                        self.dl_cfg,
                        app.downloader.start_download(&{
                            let mut o = self.dl_cfg.clone();
                            o.file_name.push_str(".mp4");
                            o
                        })
                    );
                }
            }
        });
    }
}

impl eframe::App for Ui {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Update app and release lock when done
        {
            let mut app = crate::APP.lock().unwrap();
            app.update(&mut self.notify);
            if crate::tray::Command::Exit == app.tray_menu.update() {
                frame.close();
            }
        }

        // ctx.set_debug_on_hover(true);
        ctx.request_repaint();
        egui::CentralPanel::default()
            .frame(
                eframe::egui::Frame::none()
                    .fill(ctx.style().visuals.window_fill())
                    .rounding(10.0)
                    .stroke(ctx.style().visuals.widgets.noninteractive.fg_stroke)
                    .outer_margin(0.5),
            )
            .show(ctx, |ui| {
                let app_rect = ui.max_rect();

                // draw the title bar

                let title_bar_rect = {
                    let mut rect = app_rect;
                    rect.max.y = rect.min.y + TITLE_BAR_HEIGHT;
                    rect
                };
                self.render_title_bar(ui, frame, title_bar_rect, "Lumin client");

                // rest of the window
                let bg_content_rect = {
                    let mut rect = app_rect;
                    rect.min.y = title_bar_rect.max.y;
                    rect.max.y = app_rect.max.y / 2. + TITLE_BAR_HEIGHT / 2.;
                    rect
                }
                .shrink(4.0);
                let mut bg_ui = ui.child_ui(bg_content_rect, *ui.layout());
                // if self.app.state.is_running() {

                self.render_backgrounds(&mut bg_ui, ctx, frame, bg_content_rect);

                let dl_content_rect = {
                    let mut rect = app_rect;
                    rect.min.y = bg_content_rect.max.y;
                    rect
                };
                let mut dl_ui = ui.child_ui(dl_content_rect, *ui.layout());

                self.render_downloader(&mut dl_ui, ctx, frame, dl_content_rect);
                // self.render_daemon_health(ui, ctx, frame, content_rect);
                self.notify.show(ctx)
                // ctx.settings_ui(ui);

                // } else {
                //     self.render_waiting_screen(&mut content_ui, frame, content_rect)
                // }
            });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
    }
}
