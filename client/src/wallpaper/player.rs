// Got some ideas from https://github.com/DaZiYuan/livewallpaper/blob/v3.x-rs/src-tauri/src/render/mpv_player.rs

#[derive(Debug)]
pub struct Player {
    pub id: crate::id::ID,
    pub monitor: crate::monitor::Monitor,
    pub window_id: usize,
    pub process: std::process::Child,
    pub content_path: std::path::PathBuf,
}

impl Player {
    pub fn new(
        monitor: crate::monitor::Monitor,
        window_id: usize,
        process: std::process::Child,
        content_path: std::path::PathBuf,
    ) -> Self {
        Self {
            id: crate::id::ID::new(),
            monitor,
            window_id,
            process,
            content_path,
        }
    }

    pub fn stop(&mut self) -> Result<(), std::io::Error> {
        self.process.kill()
    }

    pub fn is_dead(&mut self) -> bool {
        // false
        // self.process.
        // self.process.
        // self.process.try_wait().is_ok()
        match self.process.try_wait() {
            Ok(Some(status)) => {
                debug!("Player ({id:?})exited with: {status}", id = self.id);
                true
            }
            Ok(None) => {
                // debug!("Player is still running");
                false
            }
            Err(e) => {
                error!(
                    "error attempting to wait for player ({id:?}): {e}",
                    id = self.id
                );
                true
            }
        }
    }
}
