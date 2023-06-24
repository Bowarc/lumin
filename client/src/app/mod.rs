pub mod state;

#[derive(Default)]
pub struct App {
    pub wallpaper: crate::wallpaper::Wallpaper,
    pub tray_menu: crate::tray::Menu,
    pub downloader: crate::ytdl::DownloadState,
    dl_state: crate::ytdl::DownloadState,
}
impl App {
    pub fn update(&mut self, _notify: &mut egui_notify::Toasts) {
        if self.wallpaper.wm.update().is_err() {
            error!("Window manager encountered an error while updating, please check logs for more info")
        }

        self.wallpaper.clean_players();

        // after cleaning player list, check if there is any running, else, inform the ui
        if self.wallpaper.players.is_empty() {
            // error!("[TODO] reset the background displayed in the ui");
            // notify.error("There has been a problem with the background, please retry");
        }

        self.wallpaper.clean_players();
    }
    pub fn update_bg(
        &mut self,
        id_opt: Option<crate::id::ID>,
        monitor_indx: usize,
        video_path: String,
    ) -> Result<(crate::id::ID, crate::monitor::Monitor, std::path::PathBuf), String> {
        if video_path.is_empty() {
            warn!("Video path empty");
            return Err(String::from("Video path is not valid"));
        }

        let content_pathbuf = std::path::PathBuf::from(video_path.clone());
        if !content_pathbuf.exists() {
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
        if let Some(id) = id_opt {
            debug!("Updating background on screen: {selected_monitor:?} and with file path: {video_path}");
            match self.wallpaper.update_player(
                id,
                selected_monitor.clone(),
                std::path::PathBuf::from(video_path),
            ) {
                Ok(()) => {
                    debug!("Successfully created new player with id: {id:?}");
                    Ok((id, selected_monitor, content_pathbuf))
                }
                Err(e) => Err(format!("{e}")),
            }
        } else {
            debug!("Starting background on screen: {selected_monitor:?} and with file path: {video_path}");
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
        }
        // unimplemented!()
    }

    pub fn remove_bg(&mut self, background_id: crate::id::ID) -> Result<(), String> {
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
    }

    pub fn on_exit(&mut self) {
        debug!("Cleaning App");
        self.wallpaper.on_exit();
    }
}
