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

#[derive(Default)]
pub struct App {
    pub state: State,
    pub dvar_cache: crate::dvar_cache::DVarCache,
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
