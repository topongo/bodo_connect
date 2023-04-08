#![allow(dead_code)]

#[cfg(feature = "serde")]
use serde::Deserialize;
use std::net::IpAddr;

use crate::net::Subnet;
use crate::ssh::hop::Hop;
use crate::ssh::options::PortOption;
#[cfg(feature = "wake")]
use crate::waker::Waker;

#[cfg_attr(feature = "serde", derive(Deserialize))]
#[derive(Debug)]
pub struct Host {
    // identity
    pub name: String,
    pub ip: IpAddr,
    pub port: u16,
    // if this is not None then the host is a network master
    pub eport: Option<u16>,
    pub user: String,
    #[cfg(feature = "wake")]
    pub waker: Option<Waker>,
}

impl Host {
    pub fn new(
        name: String,
        user: String,
        ip: IpAddr,
        port: u16,
        eport: Option<u16>,
        #[cfg(feature = "wake")] waker: Option<Waker>,
    ) -> Self {
        Self {
            name,
            ip,
            port,
            eport,
            user,
            #[cfg(feature = "wake")]
            waker,
        }
    }

    pub fn is_master(&self) -> bool {
        self.eport.is_some()
    }

    pub fn get_hop(&self, subnet: Option<&Subnet>) -> Hop {
        match subnet {
            Some(s) => match self.eport {
                Some(p) => Hop::new(self.user.clone(), s.subdomain.clone(), p),
                None => panic!("cannot generate external hop for non-master hosts"),
            },
            None => Hop::new(self.user.clone(), self.ip.to_string(), self.port),
        }
    }

    pub fn identity_string(&self) -> String {
        format!("{}@{}", self.user, self.ip)
    }

    pub fn port_option(&self) -> Option<PortOption> {
        if self.port == 22 {
            None
        } else {
            Some(PortOption::new(self.port))
        }
    }
}

impl PartialEq for Host {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}
