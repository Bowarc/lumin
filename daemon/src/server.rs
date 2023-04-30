// copied form my other project <github.com/Bowarc/WTBC>

pub struct Server {
    client: Option<Client>,
    tcp_listener: std::net::TcpListener,
}

pub struct Client {
    socket: shared::networking::Socket<
        shared::networking::ClientMessage, // Reading
        shared::networking::DaemonMessage, // Writing
    >,
}

impl Server {
    pub fn new() -> Self {
        let listener = std::net::TcpListener::bind(shared::networking::DEFAULT_ADDRESS).unwrap();
        listener.set_nonblocking(true).unwrap();

        Self {
            client: None,
            tcp_listener: listener,
        }
    }

    pub fn update(&mut self, w: &mut crate::wallpaper::Wallpaper, frame_measurements: (f32, f32)) {
        if let Some(client) = &mut self.client {
            if client.update(w, frame_measurements).is_err() {
                warn!(
                    "Client ({}) encountered an error, shutting down the socket. .",
                    client.socket.remote_addr()
                );
                self.client = None
            }
        } else {
            match self.tcp_listener.accept() {
                Ok((stream, addr)) => {
                    debug!("New client {addr:?}");
                    stream.set_nodelay(true).unwrap(); // ?

                    self.client = Some(Client::new(stream));
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // wait until network socket is ready, typically implemented
                    // via platform-specific APIs such as epoll or IOCP
                    // println!("Would block");
                    // continue;

                    // About this part, as the implementation is non-blocking,
                    // i'll assume that the program will do some other job before getting back to this part,
                    // therefore the socket will have time to do it's things
                }

                Err(e) => {
                    error!("Error while listening for clients: {e:?}");
                }
            }
        }
    }
}

impl Client {
    pub fn new(stream: std::net::TcpStream) -> Self {
        Self {
            socket: shared::networking::Socket::<
                shared::networking::ClientMessage, // Reading
                shared::networking::DaemonMessage, // Writing
            >::new(stream),
        }
    }

    pub fn update(
        &mut self,
        w: &mut crate::wallpaper::Wallpaper,
        frame_measurements: (f32, f32),
    ) -> Result<(), crate::error::Error> {
        self.socket.send(shared::networking::DaemonMessage::Tick(
            frame_measurements.0, // total frame time
            frame_measurements.1, // target tps
        ))?;
        match self.socket.recv() {
            Ok(message) => {
                debug!("Got a message from client: {message:?}");

                let response = match message {
                    shared::networking::ClientMessage::Text(txt) => {
                        shared::networking::DaemonMessage::Text(txt)
                    }
                    shared::networking::ClientMessage::VarRequest(id) => match id {
                        shared::vars::VarId::MonitorList => {
                            shared::networking::DaemonMessage::ValUpdate(
                                id,
                                shared::vars::Var::MonitorList(w.screens.clone()),
                            )
                        }
                    },
                    shared::networking::ClientMessage::BackgroundSetup(monitor, content) => match w
                        .start_player(monitor.clone(), content.clone())
                    {
                        Ok(new_player_id) => shared::networking::DaemonMessage::BackgroundUpdate(
                            new_player_id,
                            monitor,
                            content,
                        ),
                        Err(e) => {
                            error!("{e}");
                            panic!("{e}")
                        }
                    },
                    shared::networking::ClientMessage::BackgroundStop(id) => {
                        match w.stop_player(crate::wallpaper::PlayerFindMethod::PlayerID(id)) {
                            Ok(_) => shared::networking::DaemonMessage::BackgroundStop(id),
                            Err(e) => {
                                error!("{e}");
                                panic!("{e}")
                            }
                        }
                    }
                };
                self.socket.send(response)?;
            }
            Err(e) => {
                // This might be the strangest lines of code that i've ever wrote
                if if let shared::networking::SocketError::Io(ref a) = e {
                    a.kind() == std::io::ErrorKind::WouldBlock
                } else {
                    false
                } {
                    // Error kind is WouldBlock, skipping
                } else {
                    error!("Error while listening for message: {e}");
                    Err(e)?;
                }
            }
        }

        Ok(())
    }
}
