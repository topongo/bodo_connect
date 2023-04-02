#[derive(Debug)]
pub struct Hop {
    user: String,
    host: String,
    port: u16
}

impl Hop {
    pub fn new(user: String, host: String, port: u16) -> Hop {
        Hop { user, host, port }
    }
    
    pub fn to_string(&self) -> String { format!("{}@{}:{}", self.user, self.host, self.port) }
}
