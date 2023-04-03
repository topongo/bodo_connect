use std::net::IpAddr;
use serde::Deserialize;
use crate::ssh::hop::Hop;
use crate::ssh::options::PortOption;
use crate::net::Subnet;
use crate::waker::{Waker};

#[derive(Debug, Deserialize)]
pub struct Host {
    // identity
    pub name: String,
    uuid: String,
    pub ip: IpAddr,
    pub port: u16,
    // if this is not None then the host is a network master
    pub eport: Option<u16>,
    pub user: String,
    pub waker: Option<Waker>
}

impl Host {
    pub fn new(
        name: String,
        uuid: String,
        user: String,
        ip: IpAddr,
        port: u16,
        eport: Option<u16>,
        waker: Option<Waker>
    ) -> Host {
        Host { name, uuid, ip, port, eport, user, waker }
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

    pub fn identity_string(&self) -> String {
        return format!("{}@{}", self.user, self.ip)
    }

    pub fn port_option(&self) -> Option<PortOption> {
        if self.port == 22 {
            None
        } else {
            Some(PortOption::new(self.port))
        }
    }
}
