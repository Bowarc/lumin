// Got some ideas from https://github.com/DaZiYuan/livewallpaper/blob/v3.x-rs/src-tauri/src/render/mpv_player.rs

#[derive(thiserror::Error, Debug)]
pub enum PlayerError {
    #[error("Io: {0}")]
    Io(#[from] std::io::Error),
    #[error("Verification error: {0}")]
    Verification(String),
}

pub struct Player {
    monitor: shared::monitor::Monitor,
    window_id: winapi::shared::windef::HWND,
    process: std::process::Child,
}

impl Player {
    pub fn run(
        monitor: shared::monitor::Monitor,
        path: std::path::PathBuf,
    ) -> Result<Self, PlayerError> {
        if !path.exists() {
            return Err(PlayerError::Verification(format!(
                "The given video path does not exists: '{path:?}'"
            )));
        }
        let target_window_id = crate::wallpaper::utils::get_workerw_id().ok_or(
            PlayerError::Verification("Could not get the workerW's id".to_string()),
        )?;

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
            .spawn()?;

        // Set the position&size of the workerW to fit exacly the screen
        crate::wallpaper::utils::move_window(target_window_id, monitor.position, monitor.size);

        // Still need to restore the worker on close tho ^

        Ok(Self {
            monitor,
            window_id: target_window_id,
            process,
        })
    }
}
