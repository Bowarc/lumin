#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error while operating the socket: {0}")]
    Socket(#[from] shared::networking::SocketError),
    #[error("Error while using the player")]
    Player(#[from] PlayerError),
    #[error("Error while using the Wallpaper")]
    Wallpaper(#[from] WallpaperError),
    #[error("Not connected to daemon")]
    HesDisconnected,
    // #[error("An error occured while operating the dvar cache: {0}")]
    // DVarCache(#[from] crate::dvar_cache::CacheError),
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
