#![allow(dead_code)]

use std::collections::BTreeMap;
use crate::ssh::hop::Hop;

pub trait SSHOption {
    fn extended_name(&self) -> bool;
    fn name(&self) -> &'static str;
    fn value(&self) -> Option<String>;
}

#[derive(Default)]
pub struct SSHOptionStore {
    options: BTreeMap<&'static str, Box<dyn SSHOption>>,
    pub cmd: Option<Vec<String>>
}

impl SSHOptionStore {
    pub fn new(cmd: Option<String>) -> Self {
        Self {
            cmd: cmd.map(|f| 
                     f.split(' ')
                     .map(|s| s.to_string())
                     .collect::<Vec<String>>()
            ),
            options: BTreeMap::new(),
        }
    }

    pub fn args_gen(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut coupling = false;
        for o in self.options.values() {
            if o.extended_name() {
                out.push(format!("--{}", o.name()));
                if let Some(v) = o.value() {
                    out.push(v)
                }
                coupling = false;
            } else if coupling {
                out.last_mut().unwrap().push_str(o.name())
            } else {
                out.push(format!("-{}", o.name()));
                match o.value() {
                    Some(v) => {
                        out.push(v);
                        coupling = false;
                    }
                    None => {
                        coupling = true;
                    }
                }
            }
        }

        out
    }

    pub fn add_option(&mut self, option: Box<dyn SSHOption>) {
        self.options.insert(option.name(), option);
    }

    pub fn merge(&mut self, other: Self) {
        for o in other.options.into_values() {
            self.add_option(o)
        }
    }
}

pub struct JumpHosts {
    hops: Vec<Hop>,
}

impl JumpHosts {
    pub fn new(hosts: Vec<Hop>) -> JumpHosts {
        JumpHosts { hops: hosts }
    }

    pub fn add_host(&mut self, h: Hop) {
        self.hops.push(h)
    }
}

impl SSHOption for JumpHosts {
    fn extended_name(&self) -> bool {
        false
    }

    fn name(&self) -> &'static str {
        "J"
    }

    fn value(&self) -> Option<String> {
        Some(
            self.hops
                .iter()
                .map(|h| h.to_string_with_port())
                .collect::<Vec<String>>()
                .join("."),
        )
    }
}

pub enum GenericOption {
    Switch(&'static str),
    Value(&'static str, String),
}

impl SSHOption for GenericOption {
    fn extended_name(&self) -> bool {
        match self {
            Self::Switch(s) => s,
            Self::Value(s, _) => s,
        }.len() > 1
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Switch(s) => s,
            Self::Value(s, _) => s,
        }
    }

    fn value(&self) -> Option<String> {
        match self {
            Self::Switch(_) => None,
            Self::Value(_, s) => Some(s.clone()),
        }
    }
}

#[derive(Debug)]
pub struct PortOption {
    port: u16,
}

impl PortOption {
    pub fn new(port: u16) -> Self {
        Self { port }
    }
}

impl SSHOption for PortOption {
    fn extended_name(&self) -> bool {
        false
    }

    fn name(&self) -> &'static str {
        "p"
    }

    fn value(&self) -> Option<String> {
        Some(self.port.to_string())
    }
}
