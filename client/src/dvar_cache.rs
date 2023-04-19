#[derive(thiserror::Error, Debug)]
pub enum CacheError {
    #[error("Client did not received {0} yet")]
    NotYetReceived(shared::vars::VarId),
    #[error("SocketError: {0}")]
    SocketError(#[from] shared::networking::SocketError),
    // #[error("CustomError: {0}")]
    // Custom(String),
}

pub enum Future {
    NotSent,
    Sent,
    Received(shared::vars::Var),
}
#[derive(Default)]
pub struct DVarCache {
    cache: std::collections::HashMap<shared::vars::VarId, Future>,
}

impl DVarCache {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }
    pub fn request(&mut self, id: shared::vars::VarId) {
        // Would maybe be smart to not delete the old value as it's a cache.
        // meh not a big problem for now
        self.cache.insert(id, Future::NotSent);
    }
    pub fn update(
        &mut self,
        socket: &mut shared::networking::Socket<
            shared::networking::DaemonMessage,
            shared::networking::ClientMessage,
        >,
    ) -> Result<(), CacheError> {
        for (id, req) in &mut self.cache {
            if matches!(req, Future::NotSent) {
                socket.send(shared::networking::ClientMessage::VarRequest(*id))?;
                *req = Future::Sent;
                debug!("Requesting {id}")
            }
        }
        Ok(())
    }
    pub fn recv(
        &mut self,
        id: shared::vars::VarId,
        val: shared::vars::Var,
    ) -> Result<(), CacheError> {
        self.cache.insert(id, Future::Received(val));

        Ok(())
    }
    pub fn get(&mut self, id: &shared::vars::VarId) -> Result<&shared::vars::Var, CacheError> {
        if self.cache.get(id).is_none() {
            self.request(*id);
            return Err(CacheError::NotYetReceived(*id));
        }

        let req = self.cache.get(id).unwrap();

        if let Future::Received(val) = req {
            Ok(val)
        } else {
            Err(CacheError::NotYetReceived(*id))
        }
    }
    pub fn get_mut(
        &mut self,
        id: &shared::vars::VarId,
    ) -> Result<&mut shared::vars::Var, CacheError> {
        if self.cache.get(id).is_none() {
            self.request(*id);
            return Err(CacheError::NotYetReceived(*id));
        }

        let req = self.cache.get_mut(id).unwrap();

        if let Future::Received(val) = req {
            Ok(val)
        } else {
            Err(CacheError::NotYetReceived(*id))
        }
    }
}
