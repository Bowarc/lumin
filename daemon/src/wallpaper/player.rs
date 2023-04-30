// Got some ideas from https://github.com/DaZiYuan/livewallpaper/blob/v3.x-rs/src-tauri/src/render/mpv_player.rs

#[derive(Debug)]
pub struct Player {
    pub id: shared::id::ID,
    pub monitor: shared::monitor::Monitor,
    pub window_id: usize,
    pub process: std::process::Child,
    pub content_path: std::path::PathBuf,
}

impl Player {
    pub fn new(
        monitor: shared::monitor::Monitor,
        window_id: usize,
        process: std::process::Child,
        content_path: std::path::PathBuf,
    ) -> Self {
        Self {
            id: shared::id::ID::new(),
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
                println!("Player ({id:?})exited with: {status}", id = self.id);
                true
            }
            Ok(None) => {
                println!("status not ready yet, let's really wait");
                false
            }
            Err(e) => {
                println!("error attempting to wait: {e}");
                true
            }
        }
    }
}
