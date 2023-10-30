use std::collections::BTreeMap;
use crate::ssh::SSHOptionStore;
use crate::ssh::process::Process;

pub struct SSHFSProcess {
    ssh_options: SSHOptionStore,
    identity: String,
    remote: String,
    mountpoint: String
}

impl SSHFSProcess {
    pub fn new(
        identity: String,
        remote: String,
        mountpoint: String,
        ssh_options: SSHOptionStore
    ) -> SSHFSProcess {
        SSHFSProcess { identity, remote, mountpoint, ssh_options }
    }
}

impl Process for SSHFSProcess {
    fn get_args(&self) -> Vec<String> {
        let mut args = vec!["sudo".to_string(), "--preserve-env=SSH_AUTH_SOCK".to_string(), "sshfs".to_string()];

        let mut options: BTreeMap<String, Option<String>> = BTreeMap::from([("allow_other".to_string(), None)]);
        let ssh_options = self.ssh_options.args_gen();
        if !ssh_options.is_empty() {
            options.insert("ssh_command".to_string(), Some(format!("'ssh {}'", ssh_options.join(" "))));
        }

        if !options.is_empty() {
            args.push("-o".to_string());
            args.push(options
                .iter()
                .map(|(k, val)| {
                    match val {
                        Some(v) => format!("{}={}", k, v),
                        None => k.to_string()
                    }
                })
                .collect::<Vec<String>>()
                .join(",")
            )
        }

        args.push(format!("{}:{}", self.identity, self.remote));
        args.push(self.mountpoint.clone());
        args
    }
}
