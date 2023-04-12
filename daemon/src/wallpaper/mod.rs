pub mod player;
mod utils;

fn mpv_dir() -> std::path::PathBuf {
    let lumin_root = {
        let mut o = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        o.pop(); //remove the /daemon
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
    // o.pop();
    debug!("{o:?}");

    assert!(o.exists());
    o
}

pub struct Wallpaper {
    pub screens: Vec<shared::monitor::Monitor>,
    pub players: Vec<player::Player>, // currently only supports one player
}

impl Wallpaper {
    pub fn new() -> Self {
        let screens = utils::get_screens();

        Self {
            screens,
            players: Vec::new(),
        }
    }
    pub fn add_player(&mut self, p: player::Player) {
        self.players.push(p)
    }
}
