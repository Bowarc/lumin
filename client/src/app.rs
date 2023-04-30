#[derive(Default)]
pub enum AppState {
    #[default]
    Init,
    DaemonBootingUp {
        start_time: std::time::SystemTime,
    },
    ConnectingToDaemon,
    Running {
        socket: shared::networking::Socket<
            shared::networking::DaemonMessage,
            shared::networking::ClientMessage,
        >,
        sync_state: SyncState,
    },
}

#[derive(Default, PartialEq)]
pub enum SyncState {
    #[default]
    No,
    Requested,
    Yes,
}

// pub struct AppState {
//     pub str_anim: crate::animations::StringAnimation,
//     pub inner: AppState,
// }

#[derive(Debug, Default, Clone)]
pub enum BackgroundState {
    #[default]
    NotSent,
    Sent,
    Connected {
        id: shared::id::ID,
    },
}

#[derive(Debug, Clone)]
pub struct State<Inner> {
    pub str_anim: crate::animations::StringAnimation,
    pub inner: Inner,
}

#[derive(Default, Debug, Clone)]
pub struct Background {
    pub monitor_index: usize,
    pub video_path: String,
    pub state: State<BackgroundState>,
}

#[derive(Default)]
pub struct App {
    pub state: State<AppState>,
    pub dvar_cache: crate::dvar_cache::DVarCache,
    pub backgrounds: Vec<Background>,
}

impl App {
    pub fn try_connect_to_daemon(&mut self) -> bool {
        match &self.state.inner {
            AppState::Init => {
                if crate::utils::is_daemon_running() {
                    // don't wait for the daemon to start if it's already on
                    self.state.set_connecting_to_daemon();
                } else {
                    crate::utils::start_daemon();
                    self.state
                        .set_daemon_booting_up(std::time::SystemTime::now())
                }
                false
            }
            AppState::DaemonBootingUp { start_time } => {
                let min_runtime_connection = std::time::Duration::from_secs_f32(1.0);

                if start_time.elapsed().unwrap() < min_runtime_connection {
                    // Let a bit of time for the daemon to start
                    // warn!("Let a bit of time for the daemon to start");
                    false
                } else {
                    self.state.set_connecting_to_daemon();
                    false
                }
            }
            AppState::ConnectingToDaemon => {
                if let Some(socket) = crate::utils::try_connect_to_daemon() {
                    self.state.set_running(socket, SyncState::default())
                } else {
                    error!("Could not create the socket");
                    self.state.set_init()
                }
                true
            }
            AppState::Running { .. } => {
                warn!("Why am i trying to connect to the daemon while im already connected");
                true
            }
        }
    }

    pub fn try_send(
        &mut self,
        message: shared::networking::ClientMessage,
    ) -> Result<(), crate::error::Error> {
        if let AppState::Running { socket, .. } = &mut self.state.inner {
            Ok(socket.send(message)?)
        } else {
            Err(crate::error::Error::HesDisconnected) // https://www.youtube.com/watch?v=j-IVQDhUNsE
        }
    }

    pub fn update(&mut self, notify: &mut egui_notify::Toasts) {
        if self.state.is_running() {
            self._update(notify)
        } else {
            self.try_connect_to_daemon();
        }
    }
    pub fn setup_bg(
        &mut self,
        background_index: usize,
    ) -> Result<(shared::monitor::Monitor, std::path::PathBuf), String> {
        if !matches!(self.state.inner, AppState::Running { .. }) {
            warn!("Not connected to daemon");
            return Err(String::from("Not connected to daemon"));
        };

        if self
            .dvar_cache
            .get(&shared::vars::VarId::MonitorList)
            .is_err()
        {
            warn!("Cant't get monitor list");
            return Err(String::from("Cant't get monitor list"));
        }

        let selected_background = self.backgrounds.get_mut(background_index).unwrap();

        if selected_background.video_path.is_empty() {
            warn!("Video path empty");
            return Err(String::from("Video path is not valid"));
        }

        let content_pathbuf = std::path::PathBuf::from(selected_background.video_path.clone());

        if !content_pathbuf.exists() {
            warn!("Path does not exist");
            return Err(String::from("Video path is not valid"));
        }

        let selected_monitor = self
            .dvar_cache
            .get(&shared::vars::VarId::MonitorList)
            .unwrap()
            .monitor_list()
            .unwrap()
            .get(selected_background.monitor_index)
            .unwrap()
            .clone();

        // As we use the same signal for background setup and update
        // we need to give the id if we want it to be an update message
        let id = if let BackgroundState::Connected { id } = selected_background.state.inner {
            Some(id)
        } else {
            None
        };

        let message = shared::networking::ClientMessage::BackgroundUpdate(
            id,
            selected_monitor.clone(),
            content_pathbuf.clone(),
        );

        if let Err(send_error) = self.try_send(message) {
            error!("{send_error}");
        }

        self.backgrounds
            .get_mut(background_index)
            .unwrap()
            .state
            .set_sent();
        Ok((selected_monitor, content_pathbuf))
    }

    pub fn remove_bg(&mut self, background_index: usize) -> Result<(), String> {
        let deleted = if let Some(deleted) = self.backgrounds.get(background_index) {
            deleted
        } else {
            return Err("The given idex is not in the background list".to_string());
        };

        if let BackgroundState::Connected { id } = deleted.state.inner {
            self.try_send(shared::networking::ClientMessage::BackgroundStop(id))
                .map_err(|e| format!("{e}"))
        } else {
            self.backgrounds.remove(background_index);
            Err(String::from(
                "The background was not in sync with the daemon\nNo requests were made",
            ))
        }
    }

    fn _update(&mut self, notify: &mut egui_notify::Toasts) {
        let AppState::Running {
            socket, sync_state } = &mut self.state.inner else { return  };
        // let socket = self.state.get_socket().expect("'bout to kms");

        // check of vars
        if let Err(e) = self.dvar_cache.update(socket) {
            notify.error(format!("{e}"));
            error!("{e}")
        }

        // if we have the monitor list and we didn't requested it yet,
        // we can ask for the backgrounds that the daemon is currenly running
        if self
            .dvar_cache
            .get(&shared::vars::VarId::MonitorList)
            .is_ok()
            && *sync_state == SyncState::No
        {
            debug!("requesting sync");
            if let Err(e) = socket.send(shared::networking::ClientMessage::SyncRequest) {
                notify.error(format!("{e}"));
            }
            *sync_state = SyncState::Requested
        }

        // Check if the daemon sent messages
        match socket.recv() {
            Ok(message) => {
                debug!("Got a message from daemon: {message:?}");

                match message {
                    shared::networking::DaemonMessage::Text(txt) => {
                        debug!("Daemon sent: {txt}");
                        notify.info(format!("Daemon sent {txt}"));
                    }
                    shared::networking::DaemonMessage::ValUpdate(id, val) => {
                        // debug!("Receiving {val:#?} for {id:?}");
                        if let Err(e) = self.dvar_cache.recv(id, val) {
                            error!("{e}");
                        }
                    }
                    shared::networking::DaemonMessage::BackgroundUpdate(id, monitor, content) => {
                        if let Some(monitor_list) = self
                            .dvar_cache
                            .get(&shared::vars::VarId::MonitorList)
                            .unwrap()
                            .monitor_list()
                        {
                            let mut backgrounds = self
                                .backgrounds
                                .iter_mut()
                                .filter(|bg| matches!(bg.state.inner, BackgroundState::Sent))
                                .filter(|bg| {
                                    if let Some(bg_monitor) = monitor_list.get(bg.monitor_index) {
                                        bg_monitor.name == monitor.name
                                    } else {
                                        false
                                    }
                                })
                                .collect::<Vec<&mut Background>>();

                            if let Some(background0) = backgrounds.get_mut(0) {
                                background0.state.set_connected(id);
                                background0.video_path = content
                                    .as_path()
                                    .display()
                                    .to_string()
                                    .replace("\\\\?\\", "")
                            } else {
                                error!("No background is marked as waiting for daemon confirmation")
                            }

                            // let mut background = backgrounds.get_mut(0).unwrap();
                        } else {
                            error!("Monitor list var not updated yet")
                        }
                    }
                    shared::networking::DaemonMessage::BackgroundStop(id) => {
                        debug!("{id:?} has been deleted");
                        self.backgrounds.retain(|bg| {
                            if let BackgroundState::Connected { id: bgid } = bg.state.inner {
                                bgid != id
                            } else {
                                true
                            }
                        })
                    }
                    shared::networking::DaemonMessage::Error(e) => {
                        notify.error(e);
                    }
                    shared::networking::DaemonMessage::Sync(daemon_backgrounds) => {
                        if let Ok(monitor_list_var) =
                            self.dvar_cache.get(&shared::vars::VarId::MonitorList)
                        {
                            let monitor_list = monitor_list_var.monitor_list().unwrap();
                            let mut bg_list = vec![];
                            for (id, monitor, content) in daemon_backgrounds.iter() {
                                let monitor_index = {
                                    let temp = monitor_list
                                        .iter()
                                        .enumerate()
                                        .filter(|(_i, m)| m.name == monitor.name)
                                        .map(|(i, _m)| i)
                                        .collect::<Vec<usize>>();

                                    *temp.first()
                                            .unwrap_or_else(||{
                                                notify.error("Could not synchronise Id({id}), try to hit the repair button to fix");
                                                &0
                                            })
                                };

                                let new_background = Background {
                                    monitor_index,
                                    video_path: content
                                        .as_path()
                                        .display()
                                        .to_string()
                                        .replace("\\\\?\\", ""),
                                    state: {
                                        let mut temp = State::<BackgroundState>::default();
                                        temp.set_connected(*id);
                                        temp
                                    },
                                };
                                bg_list.push(new_background);
                            }
                            self.backgrounds = bg_list;
                            debug!("Sync received");
                            *sync_state = SyncState::Yes
                        } else {
                            debug!("Could not parse sync, re-requesting");
                            *sync_state = SyncState::No;
                            // return;
                        };
                    }
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
                    // self.state.inner = InnerAppState::Init
                    self.state.set_init()
                }
            }
        }
    }
}

impl State<AppState> {
    pub fn set_init(&mut self) {
        self.str_anim = crate::animations::StringAnimation::new(
            300,
            "Offline", // Initializing
            eframe::egui::Color32::RED,
        );
        self.inner = AppState::Init
    }
    pub fn set_daemon_booting_up(&mut self, start_time: std::time::SystemTime) {
        self.str_anim = crate::animations::StringAnimation::new(
            300,
            "Initializing", // Daemon is booting up
            eframe::egui::Color32::YELLOW,
        );
        self.inner = AppState::DaemonBootingUp { start_time }
    }

    pub fn set_connecting_to_daemon(&mut self) {
        self.str_anim = crate::animations::StringAnimation::new(
            100,
            "Connecting. . .",
            eframe::egui::Color32::YELLOW,
        );
        self.inner = AppState::ConnectingToDaemon
    }

    pub fn set_running(
        &mut self,
        socket: shared::networking::Socket<
            shared::networking::DaemonMessage,
            shared::networking::ClientMessage,
        >,
        sync_state: SyncState,
    ) {
        self.str_anim =
            crate::animations::StringAnimation::new(200, "Connected", eframe::egui::Color32::GREEN);
        self.inner = AppState::Running { socket, sync_state }
    }
    pub fn is_running(&self) -> bool {
        matches!(self.inner, AppState::Running { .. })
    }
    pub fn get_socket(
        &mut self,
    ) -> Option<
        &mut shared::networking::Socket<
            shared::networking::DaemonMessage,
            shared::networking::ClientMessage,
        >,
    > {
        if let AppState::Running { socket, .. } = &mut self.inner {
            Some(socket)
        } else {
            None
        }
    }
}

impl Default for State<AppState> {
    fn default() -> Self {
        let mut o = Self {
            str_anim: crate::animations::StringAnimation::new(
                0,
                "..",
                eframe::egui::Color32::TRANSPARENT,
            ),
            inner: AppState::Init,
        };
        o.set_init();
        o
    }
}

impl State<BackgroundState> {
    pub fn set_not_sent(&mut self) {
        self.str_anim = crate::animations::StringAnimation::new(
            300,
            "Not yet sent", // Initializing
            eframe::egui::Color32::RED,
        );
        self.inner = BackgroundState::NotSent
    }
    pub fn set_sent(&mut self) {
        self.str_anim = crate::animations::StringAnimation::new(
            100,
            "Sent, waiting for a response", // Initializing
            eframe::egui::Color32::YELLOW,
        );
        self.inner = BackgroundState::Sent
    }
    pub fn set_connected(&mut self, id: shared::id::ID) {
        self.str_anim = crate::animations::StringAnimation::new(
            100,
            "Connected", // Initializing
            eframe::egui::Color32::GREEN,
        );
        self.inner = BackgroundState::Connected { id }
    }
}

impl Default for State<BackgroundState> {
    fn default() -> Self {
        let mut o = Self {
            str_anim: crate::animations::StringAnimation::new(
                0,
                "..",
                eframe::egui::Color32::TRANSPARENT,
            ),
            inner: BackgroundState::NotSent,
        };
        o.set_not_sent();
        o
    }
}

impl SyncState {
    pub fn is_synched(&self) -> bool {
        matches!(self, SyncState::Yes)
    }

    pub fn is_requested(&self) -> bool {
        matches!(self, SyncState::Requested)
    }
}
