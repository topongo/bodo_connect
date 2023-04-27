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

    pub fn to_string_with_port(&self) -> String {
        format!("{}{}",
                self,
                if self.port == 22 {
                    "".to_string()
                } else {
                    format!(":{}", self.port)
                })
    }
}

impl Display for Hop {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.user, self.host)
    }
}


pub fn join_hops(t: &Hop, v: &Vec<Hop>, sep: &str) -> String {
    if v.is_empty() {
        format!(
            "client -> {}",
            t.to_string_with_port()
        )
    } else {
        format!(
            "client -> {} -> {}",
            v
                .iter()
                .map(|h| h.to_string_with_port())
                .collect::<Vec<String>>()
                .join(sep),
            t.to_string_with_port()
        )
    }
}
