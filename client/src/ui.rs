use eframe::{egui, emath, epaint};

enum State {
    WaitingForDaemon {
        daemon_running: bool,
    },
    Running {
        socket: shared::networking::Socket<
            shared::networking::DaemonMessage,
            shared::networking::ClientMessage,
        >,
    },
}
#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("Not connected to daemon")]
    HesDisconnected,
    #[error("An error occured while operating the socket: {0}")]
    SocketError(#[from] shared::networking::SocketError),
}

pub struct Ui {
    state: State,
    monitors: crate::request::Request<Vec<shared::monitor::Monitor>>,
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
            (TextStyle::Monospace, FontId::new(12.0, Monospace)),
            (TextStyle::Button, FontId::new(16.0, Proportional)),
            (TextStyle::Small, FontId::new(8.0, Proportional)),
        ]
        .into();
        cc.egui_ctx.set_style(style);
        Self {
            state: State::WaitingForDaemon {
                daemon_running: crate::utils::is_daemon_running(),
            },
            monitors: crate::request::Request::NotSent,
        }
    }
}

/// Application /IPC related functions
impl Ui {
    fn try_connect_to_daemon(&mut self) -> bool {
        if let State::WaitingForDaemon { daemon_running } = &mut self.state {
            *daemon_running = crate::utils::is_daemon_running();
            if !*daemon_running {
                crate::utils::start_daemon();
                return false;
            }

            match crate::utils::try_connect_to_daemon() {
                Some(socket) => {
                    self.state = State::Running { socket };

                    if let State::Running { socket } = &mut self.state {
                        socket.send(shared::networking::ClientMessage::Text("Salut".to_string()));
                    }
                    return true;
                }
                None => return false,
            }
        }
        false
    }
    fn try_send(&mut self, message: shared::networking::ClientMessage) -> Result<(), Error> {
        if let State::Running { socket } = &mut self.state {
            Ok(socket.send(message)?)
        } else {
            Err(Error::HesDisconnected)
        }
    }

    /// Receives message from the daemon and send some back
    fn update_app(&mut self) {
        if let State::Running { socket } = &mut self.state {
            // ask for self.monitors
            debug!("Receiving..");
            match socket.recv() {
                Ok(message) => {
                    debug!("Got a message from daemon: {message:?}");

                    match message {
                        shared::networking::DaemonMessage::Text(txt) => {}
                    };
                }
                Err(e) => {
                    if if let shared::networking::SocketError::Io(ref a) = e {
                        a.kind() == std::io::ErrorKind::WouldBlock
                    } else {
                        false
                    } {
                        // Error kind is WouldBlock, skipping
                    } else {
                        error!("Error while listening for message: {e}");
                        // Err(e)?;
                    }
                }
            }
        } else {
            if self.try_connect_to_daemon() {
                debug!("yay we're connected")
            } else {
                warn!("Could not connect to daemon")
            }
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
        _frame: &mut eframe::Frame,
        _content_rect: eframe::epaint::Rect,
    ) {
        ui.label("Content");
        if ui.button("Send Hi").clicked() {
            // ignore error for now
            if let Err(e) = self.try_send(shared::networking::ClientMessage::Text("Hi".to_string()))
            {
                error!("{e}")
            }
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
        self.update_app();

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
                match self.state {
                    State::WaitingForDaemon { .. } => {
                        self.render_waiting_screen(&mut content_ui, frame, content_rect)
                    }
                    State::Running { .. } => self.render_ui(&mut content_ui, frame, content_rect),
                }
            });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
    }
}

impl State {
    fn is_running(&self) -> bool {
        matches!(self, State::Running { .. })
    }
}
