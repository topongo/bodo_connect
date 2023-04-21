#![allow(dead_code)]

#[cfg(feature = "serde")]
use serde::Deserialize;
use std::fmt::{Debug, Formatter};
use std::net::IpAddr;
use serde::de::Error;
use serde::Deserializer;

use crate::net::Host;
use crate::net::host::MasterHost;

#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct Subnet {
    // identity
    pub subdomain: String,
    #[cfg_attr(feature = "serde", serde(deserialize_with = "hosts_parser"))]
    hosts: Vec<Box<dyn Host>>,
    pub eip: Option<IpAddr>,
}

impl Subnet {
    pub fn new(subdomain: String, eip: Option<IpAddr>) -> Subnet {
        Subnet {
            subdomain,
            eip,
            hosts: Vec::new(),
        }
    }

    pub fn add_host(&mut self, h: Box<dyn Host>) {
        self.hosts.push(h);
    }

    pub fn get_host(&self, q: &str) -> Option<&dyn Host> {
        self.hosts.iter().find(|h| h.name() == q)
    }

    pub fn get_hosts(&self) -> Vec<&dyn Host> {
        self.hosts.iter().collect()
    }

    pub fn get_master(&self) -> &dyn Host {
        match self.hosts.iter().find(|h| h.is_master()) {
            Some(h) => h,
            None => panic!("subnet {} has no master", self.subdomain),
        }
    }

    pub fn has_host(&self, h: &dyn Host) -> bool {
        self.hosts.contains(h)
    }
}

impl Debug for Subnet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Subnet {{ subdomain: \"{}\", eip: {:?}, hosts: {} }}",
            self.subdomain,
            self.eip,
            self.hosts.len()
        )
    }
}

impl PartialEq for Subnet {
    fn eq(&self, other: &Self) -> bool {
        self.subdomain.eq(&other.subdomain)
    }
}


#[cfg(feature = "serde")]
fn hosts_parser<'de, D>(deserializer: D) -> Result<Vec<Box<dyn Host>>, D::Error>
    where
        D: Deserializer<'de>,
{
    let method_string = ::deserialize(deserializer)?;
    Host::from_str(&method_string).map_err(Error::custom)
}
