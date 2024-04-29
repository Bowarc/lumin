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

        self.downloader.update();

        self.wallpaper.clean_players();
    }

    // Updates or create a new player depending on the id option
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

        // As we use the same signal for background setup and update
        // we need to give the id if we want it to be an update message

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
    }

    pub fn remove_bg(&mut self, background_id: crate::id::ID) -> Result<(), String> {
        if let Err(e) = self.wallpaper.stop_player(background_id) {
            return Err(format!(
                "Failed to stop background ({background_id:?}) due to: {e}"
            ));
        }
        Ok(())
    }

    pub fn on_exit(&mut self) {
        self.wallpaper.on_exit();
    }
}
