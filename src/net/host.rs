#![allow(dead_code)]

#[cfg(feature = "serde")]
use serde::Deserialize;
use std::net::IpAddr;

use crate::net::Subnet;
use crate::ssh::hop::Hop;
use crate::ssh::options::PortOption;
#[cfg(feature = "wake")]
use crate::waker::Waker;


pub trait Host {
    fn ip(&self) -> IpAddr;

    fn user(&self) -> String;

    fn port(&self) -> u16;

    fn name(&self) -> String;

    fn identity(&self) -> String {
        format!("{}@{}", self.user(), self.ip())
    }

    fn port_option(&self) -> Option<PortOption>;

    fn get_hop(&self, subnet: Option<&Subnet>) -> Hop;

    fn is_master(&self) -> bool;

    #[cfg(feature = "wake")]
    fn waker(&self) -> Option<Waker>;
}

#[cfg_attr(feature = "serde", derive(Deserialize))]
#[derive(Debug)]
pub struct MasterHost {
    // identity
    pub name: String,
    pub ip: IpAddr,
    pub port: u16,
    pub eport: u16,
    pub user: String,
    #[cfg(feature = "wake")]
    pub waker: Option<Waker>,
}

#[cfg_attr(feature = "serde", derive(Deserialize))]
#[derive(Debug)]
pub struct SlaveHost {
    // identity
    pub name: String,
    pub ip: IpAddr,
    pub port: u16,
    pub user: String,
    #[cfg(feature = "wake")]
    pub waker: Option<Waker>,
}

impl SlaveHost {
    pub fn new(
        name: String,
        user: String,
        ip: IpAddr,
        port: u16,
        #[cfg(feature = "wake")]
        waker: Option<Waker>,
    ) -> Self {
        Self {
            name,
            ip,
            port,
            user,
            #[cfg(feature = "wake")]
            waker,
        }
    }

    // pub fn is_master(&self) -> bool {
    //     self.eport.is_some()
    // }

    // pub fn get_hop(&self, subnet: Option<&Subnet>) -> Hop {
    //     match subnet {
    //         Some(s) => match self.eport {
    //             Some(p) => Hop::new(self.user.clone(), s.subdomain.clone(), p),
    //             None => panic!("cannot generate external hop for non-master hosts"),
    //         },
    //         None => Hop::new(self.user.clone(), self.ip.to_string(), self.port),
    //     }
    // }


    pub fn port_option(&self) -> Option<PortOption> {
        if self.port == 22 {
            None
        } else {
            Some(PortOption::new(self.port))
        }
    }
}

impl PartialEq for SlaveHost {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}
