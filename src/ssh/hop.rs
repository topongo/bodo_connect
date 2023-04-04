use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct Hop {
    user: String,
    host: String,
    port: u16,
}

impl Hop {
    pub fn new(user: String, host: String, port: u16) -> Hop {
        Hop { user, host, port }
    }
}

impl Display for Hop {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}:{}", self.user, self.host, self.port)
    }
}
