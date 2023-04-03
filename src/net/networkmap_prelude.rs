use std::collections::HashSet;
use crate::net::{NetworkMap, Subnet};
use serde::Deserialize;


#[derive(Deserialize)]
pub struct NetworkMapPrelude {
    subdomains: Vec<Subnet>
}

impl NetworkMapPrelude {
    pub fn resolve(self) -> Result<NetworkMap, String> {
        let mut n = NetworkMap::new();
        let mut subs = HashSet::new();
        for s in self.subdomains.into_iter() {
            if subs.contains(&s.subdomain) {
                return Err(s.subdomain)
            } else {
                subs.insert(s.subdomain.clone());
                n.add_subnet(s);
            }
        }
        Ok(n)
    }
}
