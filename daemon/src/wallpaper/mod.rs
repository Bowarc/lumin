pub mod player;
// mod utils;

fn mpv_dir() -> std::path::PathBuf {
    let lumin_root = {
        let mut o = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        o.pop(); //remove the `/daemon`
        o
    };

    let o = std::path::PathBuf::from(format!(
        "{}\\research\\mpv\\mpv.exe",
        lumin_root
            .as_path()
            .display()
            .to_string()
            .replace("\\\\?\\", "")
    ));
    debug!("{o:?}");

    assert!(o.exists());
    o
}

#[derive(Debug)]
pub enum PlayerFindMethod {
    PlayerID(shared::id::ID),        // ID of the player
    PlayerIndex(usize),              // Index of the player in the Wallpaper::player list
    MonitorName(String),             // Name of the monitor struct it's playing on.
    ContentPath(std::path::PathBuf), // Any player tha plays the path of the given media
}

pub struct Wallpaper {
    pub screens: Vec<shared::monitor::Monitor>,
    pub players: Vec<player::Player>, // currently supports only one player
    pub wm: Box<dyn crate::window_manager::WindowManager>,
}

impl Wallpaper {
    pub fn new<WM: crate::window_manager::WindowManager + 'static>(wm: WM) -> Self {
        let screens = wm.get_screen_list();

        Self {
            screens,
            players: Vec::new(),
            wm: Box::new(wm),
        }
    }
    pub fn start_player(
        &mut self,
        monitor: shared::monitor::Monitor,
        path: std::path::PathBuf,
    ) -> Result<shared::id::ID, crate::error::Error> {
        if !path.exists() {
            return Err(crate::error::PlayerError::Verification(format!(
                "The given video path does not exists: '{path:?}'"
            ))
            .into());
        }
        let target_window_id =
            self.wm
                .get_bg_window_checked()
                .ok_or(crate::error::PlayerError::Verification(
                    "Could not get workerW id from the window manager".to_string(),
                ))?;

        let pretty_mpv_path = crate::wallpaper::mpv_dir()
            .as_path()
            .display()
            .to_string()
            .replace("\\\\?\\", "");
        let pretty_path = path.as_path().display().to_string().replace("\\\\?\\", "");
        let args = vec![
            format!("--player-operation-mode=pseudo-gui"),
            format!("--force-window=yes"),
            format!("--terminal=no"),
            format!("--no-audio"),
            format!("--loop=inf"),
            format!("--wid={:?}", target_window_id),
            format!("{pretty_path}"),
        ];
        debug!("Running mpv({pretty_mpv_path}) with args: {args:?}");

        let process = std::process::Command::new(pretty_mpv_path)
            .args(args)
            .stderr(std::process::Stdio::null())
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .spawn()
            .map_err(crate::error::PlayerError::from)?;

        // Set the position&size of the workerW to fit exacly the screen
        self.wm.prepare_for_monitor(monitor.clone());
        // crate::wallpaper::utils::move_window(target_window_id, monitor.position, monitor.size);

        // Still need to restore the worker on close tho ^

        let new_player = player::Player::new(monitor, target_window_id, process, path);

        let new_player_id = new_player.id;

        self.players.push(new_player);
        Ok(new_player_id)
    }
    pub fn update_player(
        &mut self,
        id: shared::id::ID,
        monitor: shared::monitor::Monitor,
        path: std::path::PathBuf,
    ) -> Result<(), crate::error::Error> {
        error!("This is a placeholder method, please rework it");

        // as a placeholder we're gonna kill and start a new player, but in the future
        // Please implement a messaging system between the player and it's process (check livewallpaper for an example)

        self.stop_player(PlayerFindMethod::PlayerID(id))?;

        {
            if !path.exists() {
                return Err(crate::error::PlayerError::Verification(format!(
                    "The given video path does not exists: '{path:?}'"
                ))
                .into());
            }
            let target_window_id =
                self.wm
                    .get_bg_window_checked()
                    .ok_or(crate::error::PlayerError::Verification(
                        "Could not get the window id from the window manager".to_string(),
                    ))?;

            // let target_window_id = crate::wallpaper::utils::get_workerw_id().ok_or(
            //     crate::error::PlayerError::Verification("Could not get the workerW's id".to_string()),
            // )?;

            let pretty_mpv_path = crate::wallpaper::mpv_dir()
                .as_path()
                .display()
                .to_string()
                .replace("\\\\?\\", "");
            let pretty_path = path.as_path().display().to_string().replace("\\\\?\\", "");
            let args = vec![
                format!("--player-operation-mode=pseudo-gui"),
                format!("--force-window=yes"),
                format!("--terminal=no"),
                format!("--no-audio"),
                format!("--loop=inf"),
                format!("--wid={:?}", target_window_id),
                format!("{pretty_path}"),
            ];
            debug!("Running mpv({pretty_mpv_path}) with args: {args:?}");

            let process = std::process::Command::new(pretty_mpv_path)
                .args(args)
                .stderr(std::process::Stdio::null())
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .spawn()
                .map_err(crate::error::PlayerError::from)?;

            // Set the position&size of the workerW to fit exacly the screen
            self.wm.prepare_for_monitor(monitor.clone());
            // crate::wallpaper::utils::move_window(target_window_id, monitor.position, monitor.size);

            // Still need to restore the worker on close tho ^

            let mut new_player = player::Player::new(monitor, target_window_id, process, path);

            new_player.id = id;

            self.players.push(new_player);
        }

        Ok(())
    }
    pub fn stop_player(&mut self, method: PlayerFindMethod) -> Result<(), crate::error::Error> {
        let player_index = match method {
            PlayerFindMethod::PlayerID(id) => {
                let players = self
                    .players
                    .iter()
                    .enumerate()
                    .filter(|(_i, p)| p.id == id)
                    .collect::<Vec<(usize, &player::Player)>>();

                if players.is_empty() {
                    return Err(crate::error::WallpaperError::PlayerDontExist(method).into());
                } else if players.len() > 1 {
                    warn!(
                        "Multiple player has been found with method: {method:?}, players: {:?}",
                        self.players
                    );
                }

                players.get(0).unwrap().0
            }
            PlayerFindMethod::PlayerIndex(index) => {
                if self.players.get(index).is_some() {
                    // self.players.swap_remove(index);
                    // self.players.get_mut(index).unwrap()
                    index
                } else {
                    return Err(crate::error::WallpaperError::PlayerDontExist(method).into());
                }
            }
            PlayerFindMethod::MonitorName(ref name) => {
                // self.players.retain(|player| player.monitor.name != name);

                let players = self
                    .players
                    .iter()
                    .enumerate()
                    .filter(|(_i, p)| &p.monitor.name == name)
                    .collect::<Vec<(usize, &player::Player)>>();

                if players.is_empty() {
                    return Err(crate::error::WallpaperError::PlayerDontExist(method).into());
                } else if players.len() > 1 {
                    warn!(
                        "Multiple player has been found with method: {method:?}, players: {:?}",
                        self.players
                    );
                }

                players.get(0).unwrap().0
                // Debug?
            }
            PlayerFindMethod::ContentPath(ref path) => {
                // self.players.retain(|player| player.content_path != path);

                let players = self
                    .players
                    .iter()
                    .enumerate()
                    .filter(|(_i, p)| &p.content_path == path)
                    .collect::<Vec<(usize, &player::Player)>>();

                if players.is_empty() {
                    return Err(crate::error::WallpaperError::PlayerDontExist(method).into());
                } else if players.len() > 1 {
                    warn!(
                        "Multiple player has been found with method: {method:?}, players: {:?}",
                        self.players
                    );
                }

                players.get(0).unwrap().0
            }
        };

        if self.players.get(player_index).is_none() {
            error!(
                "Tf did you do ??? Index: {player_index}, players: {:?}",
                self.players
            )
        }

        let player = self.players.get_mut(player_index).unwrap();

        player.stop().map_err(crate::error::PlayerError::from)?;

        self.players.swap_remove(player_index);

        debug!("Successfully killed player with method: {method:?}");

        Ok(())
    }
    pub fn clean_players(&mut self) {
        let mut todelete = vec![];

        for (i, p) in self.players.iter_mut().enumerate() {
            if p.is_dead() {
                debug!("Removing player ({:?})", p.id);
                todelete.push(i)
            }
        }

        let mut indx = 0;
        self.players.retain(|_p| {
            indx += 1;
            !todelete.contains(&(indx - 1))
        });

        debug!("Remaining players: {:?}", self.players)
    }
    pub fn on_exit(&mut self) {
        let ids = self
            .players
            .iter()
            .map(|player| player.id)
            .collect::<Vec<shared::id::ID>>();
        for player_id in ids.iter() {
            if let Err(e) = self.stop_player(PlayerFindMethod::PlayerID(*player_id)) {
                error!("Could not clean player ({player_id:?}): {e}")
            }
        }
        self.clean_players();

        self.wm.on_exit()
    }
}
