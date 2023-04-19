#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Not connected to daemon")]
    HesDisconnected,
    #[error("An error occured while operating the socket: {0}")]
    Socket(#[from] shared::networking::SocketError),
    #[error("An error occured while operating the dvar cache: {0}")]
    DVarCache(#[from] crate::dvar_cache::CacheError),
}
