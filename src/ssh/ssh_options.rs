use crate::ssh::hop::Hop;
use crate::Host;

trait SSHOption {
    fn extended_name(&self) -> bool;
    fn name(&self) -> &'static str;
    fn value(&self) -> Option<String>;
}

#[derive(Default)]
struct SSHOptionStore {
    options: Vec<Box<dyn SSHOption>>
}

impl SSHOptionStore {
    pub fn to_string(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut coupling = false;
        for o in self.options.iter() {
            if o.extended_name() {
                out.push(format!("--{}", o.name()));
                match o.value() {
                    Some(v) => out.push(v),
                    None => {}
                }
                coupling = false;
            } else {
                if coupling {
                    out.last_mut().unwrap().push_str(&o.name())
                } else {
                    out.push(format!("-{}", o.name()));
                    match o.value() {
                        Some(v) => {
                            out.push(v);
                            coupling = false;
                        },
                        None => {
                            coupling = true;
                        }
                    }
                }
            }
        }

        out
    }
}

struct JumpHost {
    hops: Vec<Hop>
}

impl JumpHost {
    pub fn new() -> JumpHost { JumpHost { hops: Vec::new() } }

    pub fn add_host(&mut self, h: Hop) { self.hops.push(h) }
}

impl SSHOption for JumpHost {
    fn extended_name(&self) -> bool {
        false
    }

    fn name(&self) -> &'static str {
        "J"
    }

    fn value(&self) -> Option<String> {
        Some(self.hops
            .iter()
            .map(|h| {
                h.to_string()
            })
            .collect::<Vec<String>>()
            .join(".")
        )
    }
}
