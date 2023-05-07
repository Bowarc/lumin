pub mod background;
pub mod state;

#[derive(Default)]
pub struct App {
    pub state: state::State<state::AppState>,
    pub dvar_cache: crate::dvar_cache::DVarCache,
    pub backgrounds: Vec<background::Background>,
    pub tray_menu: crate::tray::Menu,
}

impl App {
    pub fn try_connect_to_daemon(&mut self) -> bool {
        match &self.state.inner {
            state::AppState::Init => {
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
            state::AppState::DaemonBootingUp { start_time } => {
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
            state::AppState::ConnectingToDaemon => {
                if let Some(socket) = crate::utils::try_connect_to_daemon() {
                    self.state.set_running(socket, state::SyncState::default())
                } else {
                    error!("Could not create the socket");
                    self.state.set_init()
                }
                true
            }
            state::AppState::Running { .. } => {
                warn!("Why am i trying to connect to the daemon while im already connected");
                true
            }
        }
    }

    pub fn try_send(
        &mut self,
        message: shared::networking::ClientMessage,
    ) -> Result<(), crate::error::Error> {
        if let state::AppState::Running { socket, .. } = &mut self.state.inner {
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
    ) -> Result<
        (
            shared::monitor::Monitor,
            shared::background::BackgroundContent,
        ),
        String,
    > {
        if !matches!(self.state.inner, state::AppState::Running { .. }) {
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

        if selected_background.content.is_empty() {
            warn!("Video path empty");
            return Err(String::from("Video path is not valid"));
        }

        let content = selected_background.build_content().ok_or(String::from(
            "Given content could not be made into an url nor a path",
        ))?;

        if !content.is_valid() {
            match content {
                shared::background::BackgroundContent::Url(url) => {
                    return Err(format!("Invalid url: {url}"))
                }
                shared::background::BackgroundContent::File(path) => {
                    return Err(format!(
                        "Invalid path: {}",
                        path.as_path().as_os_str().to_str().unwrap()
                    ))
                }
            }
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
        let id = if let state::BackgroundState::Connected { id } = selected_background.state.inner {
            Some(id)
        } else {
            None
        };

        let message = shared::networking::ClientMessage::BackgroundUpdate(
            id,
            selected_monitor.clone(),
            content.clone(),
        );

        if let Err(send_error) = self.try_send(message) {
            error!("{send_error}");
        }

        self.backgrounds
            .get_mut(background_index)
            .unwrap()
            .state
            .set_sent();
        Ok((selected_monitor, content))
    }

    pub fn remove_bg(&mut self, background_index: usize) -> Result<(), String> {
        let deleted = if let Some(deleted) = self.backgrounds.get(background_index) {
            deleted
        } else {
            return Err("The given idex is not in the background list".to_string());
        };

        if let state::BackgroundState::Connected { id } = deleted.state.inner {
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
        let state::AppState::Running {
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
            && *sync_state == state::SyncState::No
        {
            debug!("requesting sync");
            if let Err(e) = socket.send(shared::networking::ClientMessage::SyncRequest) {
                notify.error(format!("{e}"));
            }
            *sync_state = state::SyncState::Requested
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
                                .filter(|bg| matches!(bg.state.inner, state::BackgroundState::Sent))
                                .filter(|bg| {
                                    if let Some(bg_monitor) = monitor_list.get(bg.monitor_index) {
                                        bg_monitor.name == monitor.name
                                    } else {
                                        false
                                    }
                                })
                                .collect::<Vec<&mut background::Background>>();

                            if let Some(background0) = backgrounds.get_mut(0) {
                                background0.state.set_connected(id);
                                background0.content = content.to_string()
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
                            if let state::BackgroundState::Connected { id: bgid } = bg.state.inner {
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

                                let new_background = background::Background {
                                    monitor_index,
                                    content: content.to_string(),
                                    state: {
                                        let mut temp =
                                            state::State::<state::BackgroundState>::default();
                                        temp.set_connected(*id);
                                        temp
                                    },
                                };
                                bg_list.push(new_background);
                            }
                            self.backgrounds = bg_list;
                            debug!("Sync received");
                            *sync_state = state::SyncState::Yes
                        } else {
                            debug!("Could not parse sync, re-requesting");
                            *sync_state = state::SyncState::No;
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
                    // self.state.inner = Innerstate::AppState::Init
                    self.state.set_init()
                }
            }
        }
    }
}
