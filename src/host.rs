use std::net::IpAddr;
use crate::ssh::hop::Hop;
use crate::Subnet;

#[derive(Debug)]
pub(crate) struct Host {
    // identity
    pub(crate) name: String,
    uuid: String,
    pub(crate) ip: IpAddr,
    pub(crate) port: u16,
    // if this is not None then the host is a network master
    pub(crate) eport: Option<u16>,
    pub(crate) user: String
}

impl Host {
    pub fn new(name: String, uuid: String, user: String, ip: IpAddr, port: u16, eport: Option<u16>) -> Host {
        Host { name, uuid, ip, port, eport, user }
    }

    pub fn is_master(&self) -> bool {
        self.eport.is_some()
    }

    pub fn get_hop(&self, subnet: Option<&Subnet>) -> Hop {
        match subnet {
            Some(s) => match self.eport {
                Some(p) => Hop::new(self.user.clone(), s.subdomain.clone(), p),
                None => panic!("cannot generate external hop for non-master hosts")
            }
            None => Hop::new(self.user.clone(), self.ip.to_string(), self.port)
        }
    }
}
