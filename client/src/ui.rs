use eframe::{egui, emath, epaint};

const TITLE_BAR_HEIGHT: f32 = 32.0;

pub struct Ui {
    app: crate::app::App,
    notify: egui_notify::Toasts,
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
            notify: egui_notify::Toasts::default()
                .with_margin(egui::vec2(0., TITLE_BAR_HEIGHT + 4.)), // + margin
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

    fn render_daemon_health(
        &mut self,
        _ui: &mut egui::Ui,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        _content_rect: eframe::epaint::Rect,
    ) {
        egui::Area::new("my_area")
            // .fixed_pos(egui::pos2(100.0, frame.info().window_info.size.y - 50.))
            .anchor(eframe::emath::Align2::RIGHT_BOTTOM, [-10., -6.0])
            .show(ctx, |ui| {
                ui.add(egui::Label::new(egui::WidgetText::RichText(
                    egui::RichText::new(self.app.state.str_anim.get_text())
                        .color(self.app.state.str_anim.get_color())
                        .text_style(egui::TextStyle::Monospace),
                )));
            });
    }

    fn render_ui(
        &mut self,
        ui: &mut egui::Ui,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        _content_rect: eframe::epaint::Rect,
    ) {
        ui.vertical_centered(|ui| {
            if ui
                .button(egui::RichText::new("New background").size(20.))
                .clicked()
            {
                self.app.backgrounds.push(crate::app::Background::default());
            }
        });

        // All created backgrounds
        let mut todelete = vec![];

        for index in 0..self.app.backgrounds.len() {
            let bg = self.app.backgrounds.get_mut(index).unwrap();
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(format!("Background {}", index + 1));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    // Can maybe be swapped to bg.status

                    ui.add(egui::Label::new(egui::WidgetText::RichText(
                        egui::RichText::new(bg.state.str_anim.get_text())
                            .color(bg.state.str_anim.get_color())
                            .text_style(egui::TextStyle::Monospace),
                    )));

                    ui.label("Status: ")
                });
            });
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_source(index)
                    .selected_text(format!("{:?}", bg.monitor_index))
                    .width(200.)
                    .show_ui(ui, |ui| {
                        if let Ok(monitors_var) =
                            self.app.dvar_cache.get(&shared::vars::VarId::MonitorList)
                        {
                            let monitors = monitors_var.monitor_list().unwrap();
                            for (m_i, monitor) in monitors.iter().enumerate() {
                                ui.selectable_value(
                                    &mut bg.monitor_index,
                                    m_i,
                                    format!(
                                        "name{}, x{}y{}, w{}h{}",
                                        monitor.name,
                                        monitor.position.0,
                                        monitor.position.1,
                                        monitor.size.0,
                                        monitor.size.1,
                                    ),
                                );
                            }
                        }
                    });
                ui.label(
                    if let Ok(monitors_var) =
                        self.app.dvar_cache.get(&shared::vars::VarId::MonitorList)
                    {
                        let selected_monitor = monitors_var
                            .monitor_list()
                            .unwrap()
                            .get(bg.monitor_index)
                            .unwrap();

                        format!(
                            "{}, {}, size: w{} h{}",
                            selected_monitor.name,
                            selected_monitor.direction(),
                            selected_monitor.size.0,
                            selected_monitor.size.1
                        )
                    } else {
                        let txt = "Unable to retreive monitor info from daemon";
                        // self.notify.warning(txt);
                        String::from(txt)
                    },
                );
            });

            ui.horizontal(|ui| {
                ui.label("Content:");
                ui.text_edit_singleline(&mut bg.video_path);
            });

            ui.horizontal(|ui|{
                if ui.button("Send").clicked() {
                    match self.app.setup_bg(index){
                            Ok((monitor, path)) => self.notify.success(format!(
                                "Sent a backgroud request to daemon\nScreen: {monitor:?}\nContent: {path:?}",
                                path = path.as_path().display().to_string().replace("\\\\?\\", "")
                            )),
                            Err(e) => self.notify.error(format!("Could not send request\n{e}")),
                        };
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    if ui.button("delete").clicked() {
                        todelete.push(index)
                    }
                });
            });

            ui.separator();
        }

        // A bit ugly but i think it's the best way
        {
            todelete.iter().for_each(|index| {
                if let Err(e) = self.app.remove_bg(*index) {
                    self.notify
                        .warning(e)
                        .set_duration(Some(std::time::Duration::from_secs(4)));
                };
            });
        }
    }
}

impl eframe::App for Ui {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.app.update(&mut self.notify);

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

                let title_bar_rect = {
                    let mut rect = app_rect;
                    rect.max.y = rect.min.y + TITLE_BAR_HEIGHT;
                    rect
                };
                self.render_title_bar(ui, frame, title_bar_rect, "Lumin client");

                // rest of the window
                let content_rect = {
                    let mut rect = app_rect;
                    rect.min.y = title_bar_rect.max.y;
                    rect
                }
                .shrink(4.0);
                let mut content_ui = ui.child_ui(content_rect, *ui.layout());
                // if self.app.state.is_running() {
                self.render_ui(&mut content_ui, ctx, frame, content_rect);
                self.render_daemon_health(ui, ctx, frame, content_rect);
                self.notify.show(ctx)
                // } else {
                //     self.render_waiting_screen(&mut content_ui, frame, content_rect)
                // }
            });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
    }
}
