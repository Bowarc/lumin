use std::sync::mpsc;
// use thiserror::Error;

#[derive(Debug)]
pub enum ChannelError<T> {
    SendError(mpsc::SendError<T>),
    TrySendError(mpsc::TrySendError<T>),
    RecvError(mpsc::RecvError),
    TryRecvError(mpsc::TryRecvError),
    RecvTimeoutError(mpsc::RecvTimeoutError),
    Other(String),
}
impl<T> ChannelError<T> {
    fn map<U, E: Into<ChannelError<T>>>(res: Result<U, E>) -> Result<U, ChannelError<T>> {
        res.map_err(Into::into)
    }
}

impl<T> From<String> for ChannelError<T> {
    fn from(error: String) -> ChannelError<T> {
        ChannelError::Other(error)
    }
}
impl<T> From<mpsc::SendError<T>> for ChannelError<T> {
    fn from(error: mpsc::SendError<T>) -> ChannelError<T> {
        ChannelError::SendError(error)
    }
}
impl<T> From<mpsc::TrySendError<T>> for ChannelError<T> {
    fn from(error: mpsc::TrySendError<T>) -> ChannelError<T> {
        ChannelError::TrySendError(error)
    }
}
impl<T> From<mpsc::RecvError> for ChannelError<T> {
    fn from(error: mpsc::RecvError) -> ChannelError<T> {
        ChannelError::RecvError(error)
    }
}
impl<T> From<mpsc::TryRecvError> for ChannelError<T> {
    fn from(error: mpsc::TryRecvError) -> ChannelError<T> {
        ChannelError::TryRecvError(error)
    }
}
impl<T> From<mpsc::RecvTimeoutError> for ChannelError<T> {
    fn from(error: mpsc::RecvTimeoutError) -> ChannelError<T> {
        ChannelError::RecvTimeoutError(error)
    }
}
pub struct Channel<T> {
    sender: mpsc::Sender<T>,
    receiver: mpsc::Receiver<T>,
}

impl<T: std::cmp::PartialEq> Channel<T> {
    pub fn new_pair() -> (Channel<T>, Channel<T>) {
        let (sender1, receiver1) = mpsc::channel::<T>();
        let (sender2, receiver2) = mpsc::channel::<T>();

        let com1 = Channel {
            sender: sender1,
            receiver: receiver2,
        };
        let com2 = Channel {
            sender: sender2,
            receiver: receiver1,
        };
        (com1, com2)
    }

    pub fn wait_for(&self, waited_message: T) {
        loop {
            let message = self.receiver.recv().unwrap();
            if message == waited_message {
                break;
            }
        }
    }
    pub fn wait_for_or_timeout(
        &self,
        waited_message: T,
        timeout: std::time::Duration,
    ) -> Result<(), ChannelError<T>> {
        let start_time = std::time::Instant::now();

        let internal_timeout = timeout / 100;
        while start_time.elapsed() < timeout {
            // we map the internal_timeout to be very small to be able to quit as soon as the timeout is done
            // + having a dynamic internal_timeout is adding to the consistency
            match self.recv_timeout(internal_timeout) {
                Ok(message) => {
                    if message == waited_message {
                        return Ok(());
                    }
                }
                Err(err) => match err {
                    ChannelError::RecvTimeoutError(_mpsc_timeout_error) => {
                        // warn!("mpsc_timeout_error: {mpsc_timeout_error}")
                    }
                    _ => return Err(err),
                },
            }
        }
        Err(mpsc::RecvTimeoutError::Timeout.into())
    }
    pub fn send(&self, t: T) -> Result<(), ChannelError<T>> {
        ChannelError::map(self.sender.send(t))
    }
    pub fn iter(&self) -> mpsc::Iter<'_, T> {
        self.receiver.iter()
    }
    pub fn try_iter(&self) -> mpsc::TryIter<'_, T> {
        self.receiver.try_iter()
    }
    pub fn recv(&self) -> Result<T, ChannelError<T>> {
        ChannelError::map(self.receiver.recv())
    }
    pub fn try_recv(&self) -> Result<T, ChannelError<T>> {
        ChannelError::map(self.receiver.try_recv())
    }
    pub fn recv_timeout(&self, timeout: std::time::Duration) -> Result<T, ChannelError<T>> {
        ChannelError::map(self.receiver.recv_timeout(timeout))
    }
}
