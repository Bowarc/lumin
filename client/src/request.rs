pub enum Request<T> {
    NotSent,
    Sent,
    Received(T),
}

impl<T> Request<T> {
    pub fn unwrap(self) -> T {
        if let Request::Received(t) = self {
            t
        } else {
            panic!("You're a dumbass")
        }
    }
}
