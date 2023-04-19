use eframe::{egui, emath, epaint};

pub struct Ui {
    app: crate::app::App,
    daemon_connected_animation: crate::animations::StringAnimation,
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

        Self {
            app: crate::app::App::default(),
            daemon_connected_animation: crate::animations::StringAnimation::new(
                200,
                "Connected".to_string(),
            ),
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

    fn render_ui(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        _content_rect: eframe::epaint::Rect,
    ) {
        ui.label("Content");
        if ui.button("Send Hi").clicked() {
            // ignore error for now
            if let Err(e) = self
                .app
                .try_send(shared::networking::ClientMessage::Text("Hi".to_string()))
            {
                error!("{e}")
            }

            self.app
                .dvar_cache
                .request(shared::vars::VarId::MonitorList)
        }
        // ui.label(format!(
        //     "{:#?}",
        //     self.app.dvar_cache.get(&shared::vars::VarId::MonitorList)
        // ));

        // this part is a test, i'll clean it later
        {
            struct Display {
                monitor_index: usize,
                video_path: String,
            }

            let displays = vec![
                Display {
                    monitor_index: 0,
                    video_path: String::from("salut"),
                },
                Display {
                    monitor_index: 1,
                    video_path: String::from("2nd video"),
                },
            ];
            for display in displays {
                if let Ok(monitors) = self.app.dvar_cache.get(&shared::vars::VarId::MonitorList) {
                    let monitor = &monitors.monitor_list().unwrap()[display.monitor_index];
                    ui.label(format!(
                        "id {}, x{}y{}, w{}h{}",
                        display.monitor_index,
                        monitor.position.0,
                        monitor.position.1,
                        monitor.size.0,
                        monitor.size.1
                    ));
                    ui.label(display.video_path.to_string());
                    if ui.button("push").clicked() {
                        // send a reques to the daemon
                    }
                }
                ui.separator();
            }

            egui::Area::new("my_area")
                // .fixed_pos(egui::pos2(100.0, frame.info().window_info.size.y - 50.))
                .anchor(eframe::emath::Align2::RIGHT_BOTTOM, [-10., -6.0])
                .show(ctx, |ui| {
                    let target_text = self.daemon_connected_animation.get();

                    ui.add(egui::Label::new(egui::WidgetText::RichText(
                        egui::RichText::new(target_text)
                            .color(egui::Color32::GREEN)
                            .text_style(egui::TextStyle::Monospace),
                    )));
                });
        }
    }
    fn render_waiting_screen(
        &mut self,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
        _content_rect: eframe::epaint::Rect,
    ) {
        ui.heading("Waiting for daemon to start. . .");
        ui.label(
            "If the windows appears to be lagging, it's normal, this part is not multi threaded.",
        );
    }
}

impl eframe::App for Ui {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.app.update();

        ctx.set_debug_on_hover(true);
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
                let title_bar_height = 32.0;
                let title_bar_rect = {
                    let mut rect = app_rect;
                    rect.max.y = rect.min.y + title_bar_height;
                    rect
                };
                self.render_title_bar(ui, frame, title_bar_rect, "egui with custom frame");

                // rest of the window
                let content_rect = {
                    let mut rect = app_rect;
                    rect.min.y = title_bar_rect.max.y;
                    rect
                }
                .shrink(4.0);
                let mut content_ui = ui.child_ui(content_rect, *ui.layout());
                if self.app.state.is_running() {
                    self.render_ui(&mut content_ui, ctx, frame, content_rect)
                } else {
                    self.render_waiting_screen(&mut content_ui, frame, content_rect)
                }
            });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
    }
}
