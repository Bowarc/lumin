pub mod background;
pub mod state;

#[derive(Default)]
pub struct App {
    // pub state: state::State<state::AppState>,
    // pub backgrounds: Vec<background::Background>,
    pub wallpaper: crate::wallpaper::Wallpaper,
    pub tray_menu: crate::tray::Menu,
}

impl App {
    pub fn update(&mut self, notify: &mut egui_notify::Toasts) {
        self.tray_menu.update();

        if self.wallpaper.wm.update().is_err() {
            error!("Window manager encountered an error while updating, please check logs for more info")
        }

        self.wallpaper.clean_players();
    }
    pub fn setup_bg(
        &mut self,
        monitor_indx: usize,
        video_path: String,
    ) -> Result<(crate::id::ID, crate::monitor::Monitor, std::path::PathBuf), String> {
        // let selected_background = self.backgrounds.get_mut(background_index).unwrap();

        if video_path.is_empty() {
            warn!("Video path empty");
            return Err(String::from("Video path is not valid"));
        }

        // let content = selected_background.build_content().ok_or(String::from(
        //     "Given content could not be made into an url nor a path",
        // ))?;

        // let content = {
        //     let content_pathbuf = std::path::PathBuf::from(self.content.clone());
        //     if content_pathbuf.exists() {
        //         // It was in fact a path
        //         Some(shared::background::BackgroundContent::File(content_pathbuf))
        //     } else if let Ok(content_url) = url::Url::parse(&self.content) {
        //         // It was in fact an url
        //         Some(shared::background::BackgroundContent::Url(content_url))
        //     } else {
        //         // Fuck ya
        //         None
        //     }
        // };

        let content_pathbuf = std::path::PathBuf::from(video_path.clone());
        if !content_pathbuf.exists() {
            // It was in fact a path
            // Some(shared::background::BackgroundContent::File(content_pathbuf));
            return Err(String::from("Given path does not exists"));
        }

        // if !content.is_valid() {
        //     match content {
        //         shared::background::BackgroundContent::Url(url) => {
        //             return Err(format!("Invalid url: {url}"))
        //         }
        //         shared::background::BackgroundContent::File(path) => {
        //             return Err(format!(
        //                 "Invalid path: {}",
        //                 path.as_path().as_os_str().to_str().unwrap()
        //             ))
        //         }
        //     }
        // }

        // // As we use the same signal for background setup and update
        // // we need to give the id if we want it to be an update message
        // let id = if let state::BackgroundState::Connected { id } = selected_background.state.inner {
        //     Some(id)
        // } else {
        //     None

        let selected_monitor = self
            .wallpaper
            .wm
            .get_screen_list()
            .get(monitor_indx)
            .unwrap()
            .clone();

        println!("Need heavy refactoring");
        println!("Asking for a new background on screen: {selected_monitor:?} and with file path: {video_path}");

        match self.wallpaper.start_player(
            None,
            selected_monitor.clone(),
            std::path::PathBuf::from(video_path),
        ) {
            Ok(id) => {
                debug!("Successfully created new player with id: {id:?}");
                Ok((id, selected_monitor, content_pathbuf))
            }
            Err(e) => Err(format!("{e}")),
        }

        // unimplemented!()
    }

    pub fn remove_bg(&mut self, background_id: crate::id::ID) -> Result<(), String> {
        // let deleted = if let Some(deleted) = self.backgrounds.get(background_index) {
        //     deleted
        // } else {
        //     return Err("The given idex is not in the background list".to_string());
        // };

        // if let state::BackgroundState::Connected { id } = deleted.state.inner {
        //     self.try_send(shared::networking::ClientMessage::BackgroundStop(id))
        //         .map_err(|e| format!("{e}"))
        // } else {
        //     self.backgrounds.remove(background_index);
        //     Err(String::from(
        //         "The background was not in sync with the daemon\nNo requests were made",
        //     ))
        // }
        if let Err(e) = self
            .wallpaper
            .stop_player(crate::wallpaper::PlayerFindMethod::PlayerID(background_id))
        {
            let msg =
                format!("Error happend while trying to stop background ({background_id:?}){e}");
            error!("{}", msg);

            return Err(msg);
        }
        Ok(())
        // unimplemented!()
    }
}
