#![allow(dead_code)]

#[cfg(feature = "serde")]
use serde::{Deserialize,Serialize,Serializer,Deserializer};
use std::fmt::{Debug, Formatter};
use std::net::IpAddr;

use crate::net::Host;

#[cfg_attr(feature = "serde", derive(Deserialize,Serialize))]
pub struct Subnet {
    // identity
    pub subdomain: String,
    hosts: Vec<Host>,
    #[cfg_attr(feature = "serde", serde(
        serialize_with = "none_filter_ser", 
        deserialize_with = "none_filter_de",
        default,
        skip_serializing_if = "Option::is_none",
    ))]
    pub eip: Option<IpAddr>,
}

fn get_none() -> Option<IpAddr> {
    None
}

impl Subnet {
    pub fn new(subdomain: String, eip: Option<IpAddr>) -> Subnet {
        Subnet {
            subdomain,
            eip,
            hosts: Vec::new(),
        }
    }

    pub fn add_host(&mut self, h: Host) {
        self.hosts.push(h);
    }

    pub fn get_host(&self, q: &str) -> Option<&Host> {
        self.hosts.iter().find(|h| h.name == q)
    }

    pub fn get_hosts(&self) -> Vec<&Host> {
        self.hosts.iter().collect()
    }

    pub fn get_master(&self) -> &Host {
        match self.hosts.iter().find(|h| h.is_master()) {
            Some(h) => h,
            None => panic!("subnet {} has no master", self.subdomain),
        }
    }

    pub fn has_host(&self, h: &Host) -> bool {
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
fn none_filter_ser<S>(v: &Option<IpAddr>, s: S) -> Result<S::Ok, S::Error> 
    where S: Serializer
{
    match v {
        Some(ip) => s.serialize_str(&ip.to_string()),
        None => s.serialize_str(""),
    }
}

#[cfg(feature = "serde")]
fn none_filter_de<'de, D>(deserializer: D) -> Result<Option<IpAddr>, D::Error>
    where D: Deserializer<'de>
{
    let opt: Result<Option<IpAddr>, _> = Option::deserialize(deserializer);
    match opt {
        Ok(p) => Ok(p),
        Err(e) => {
            println!("{:?}", e);
            todo!()
        }
    }
}

