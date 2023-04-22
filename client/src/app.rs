#[derive(Default)]
pub enum State {
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
    },
}

#[derive(Debug, Default)]
pub enum BackgroundState {
    #[default]
    NotSent,
    Sent,
    Connected {
        id: shared::id::ID,
        string_anim: crate::animations::StringAnimation,
    },
}

#[derive(Default, Debug)]
pub struct Background {
    pub monitor_index: usize,
    pub video_path: String,
    pub state: BackgroundState,
}

#[derive(Default)]
pub struct App {
    pub state: State,
    pub dvar_cache: crate::dvar_cache::DVarCache,
    pub backgrounds: Vec<Background>,
}

impl App {
    pub fn try_connect_to_daemon(&mut self) -> bool {
        match &self.state {
            State::Init => {
                if crate::utils::is_daemon_running() {
                    // don't wait for the daemon to start if it's already on
                    self.state = State::ConnectingToDaemon;
                } else {
                    crate::utils::start_daemon();
                    self.state = State::DaemonBootingUp {
                        start_time: std::time::SystemTime::now(),
                    };
                }
                false
            }
            State::DaemonBootingUp { start_time } => {
                let min_runtime_connection = std::time::Duration::from_secs_f32(5.0);

                if start_time.elapsed().unwrap() < min_runtime_connection {
                    // Let a bit of time for the daemon to start
                    warn!("Let a bit of time for the daemon to start");
                    false
                } else {
                    self.state = State::ConnectingToDaemon;
                    false
                }
            }
            State::ConnectingToDaemon => {
                if let Some(socket) = crate::utils::try_connect_to_daemon() {
                    self.state = State::Running { socket }
                } else {
                    error!("Could not create the socket")
                }
                true
            }
            State::Running { socket: _ } => {
                warn!("Why am i trying to connect to the daemon while im already connected");
                true
            }
        }
    }

    pub fn try_send(
        &mut self,
        message: shared::networking::ClientMessage,
    ) -> Result<(), crate::error::Error> {
        if let State::Running { socket } = &mut self.state {
            Ok(socket.send(message)?)
        } else {
            Err(crate::error::Error::HesDisconnected) // https://www.youtube.com/watch?v=j-IVQDhUNsE
        }
    }

    pub fn update(&mut self) {
        if self.state.is_running() {
            self._update()
        } else {
            self.try_connect_to_daemon();
        }
    }
    pub fn setup_bg(&mut self, background_index: usize) -> Option<()> {
        let selected_background = self.backgrounds.get_mut(background_index).unwrap();

        if self
            .dvar_cache
            .get(&shared::vars::VarId::MonitorList)
            .is_err()
        {
            return None;
        }
        let content_pathbuf = std::path::PathBuf::from(selected_background.video_path.clone());

        if !content_pathbuf.exists() {
            return None;
        }

        let monitor_list = self
            .dvar_cache
            .get(&shared::vars::VarId::MonitorList)
            .unwrap()
            .monitor_list()
            .unwrap();

        let message = shared::networking::ClientMessage::BackgroundSetup(
            monitor_list
                .get(selected_background.monitor_index)
                .unwrap()
                .clone(),
            content_pathbuf,
        );

        if let Err(send_error) = self.try_send(message) {
            error!("{send_error}");
        }

        self.backgrounds.get_mut(background_index).unwrap().state = BackgroundState::Sent;
        Some(())
    }

    fn _update(&mut self) {
        let socket = self.state.get_socket().expect("'bout to kms");
        // check of vars
        if let Err(e) = self.dvar_cache.update(socket) {
            error!("{e}")
        }

        // Check if the daemon sent messages
        match socket.recv() {
            Ok(message) => {
                debug!("Got a message from daemon: {message:?}");

                match message {
                    shared::networking::DaemonMessage::Text(_txt) => {}
                    shared::networking::DaemonMessage::ValUpdate(id, val) => {
                        // debug!("Receiving {val:#?} for {id:?}");
                        if let Err(e) = self.dvar_cache.recv(id, val) {
                            error!("{e}");
                        }
                    }
                    shared::networking::DaemonMessage::BackgroundUpdate(id, monitor, content) => {
                        let mut background_opt: Option<&mut Background> = None;

                        for bg in self
                            .backgrounds
                            .iter_mut()
                            .filter(|bg| matches!(bg.state, BackgroundState::Sent))
                        {
                            let monitor_list = self
                                .dvar_cache
                                .get(&shared::vars::VarId::MonitorList)
                                .unwrap()
                                .monitor_list()
                                .unwrap();

                            if let Some(bg_monitor) = monitor_list.get(bg.monitor_index) {
                                if bg_monitor.name == monitor.name {
                                    background_opt = Some(bg)
                                }
                            }
                        }

                        if let Some(background) = background_opt {
                            background.state = BackgroundState::Connected {
                                id,
                                string_anim: crate::animations::StringAnimation::new(
                                    200,
                                    "Connected",
                                ),
                            };
                            background.video_path = content
                                .as_path()
                                .display()
                                .to_string()
                                .replace("\\\\?\\", "")
                        }
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
                    self.state = State::Init
                }
            }
        }
    }
}

impl State {
    pub fn is_running(&self) -> bool {
        matches!(self, State::Running { .. })
    }
    pub fn get_socket(
        &mut self,
    ) -> Option<
        &mut shared::networking::Socket<
            shared::networking::DaemonMessage,
            shared::networking::ClientMessage,
        >,
    > {
        if let State::Running { socket } = self {
            Some(socket)
        } else {
            None
        }
    }
}

impl Background {
    // make a function to apply the background using the socket
    pub fn enable(
        &mut self,
        socket: &mut shared::networking::Socket<
            shared::networking::DaemonMessage,
            shared::networking::ClientMessage,
        >,
    ) {
        // SEND A REQUEST TO DAEMON TO SET UP A BACKGROUD
        // self.state =
        //     BackgroundState::Connected(crate::animations::StringAnimation::new(200, "Connected"))
    }

    pub fn disable(
        &mut self,
        socket: &mut shared::networking::Socket<
            shared::networking::DaemonMessage,
            shared::networking::ClientMessage,
        >,
    ) {
    }
}
