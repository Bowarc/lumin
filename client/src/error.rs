#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error while using the player")]
    Player(#[from] PlayerError),
    #[error("Error while using the Wallpaper")]
    Wallpaper(#[from] WallpaperError),
}

#[derive(thiserror::Error, Debug)]
pub enum WallpaperError {
    #[error("The search for the given method did not find any match")]
    PlayerDontExist(crate::wallpaper::PlayerFindMethod),
}

#[derive(thiserror::Error, Debug)]
pub enum PlayerError {
    #[error("Io: {0}")]
    Io(#[from] std::io::Error),
    #[error("Verification error: {0}")]
    Verification(String),
}
