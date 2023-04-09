#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Test message")]
    Test,
    #[error("Error while operating the socket: {0}")]
    SocketError(#[from] shared::networking::SocketError),
}
