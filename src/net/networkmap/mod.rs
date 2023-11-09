#[cfg(feature = "serde")]
mod parsing;

#[cfg(feature = "serde")]
pub use parsing::NetworkMapParseError;

#[cfg(not(feature = "log"))]
use crate::{debug, info, warn};
use crate::net::external_ip::get_ip;
#[cfg(feature = "log")]
use log::{debug, info, warn};
use reachable::{IcmpTarget, ResolvePolicy, Status, Target, TcpTarget};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs};
use std::time::Duration;
#[cfg(feature = "wake")]
use subprocess::ExitStatus;
#[cfg(feature = "sshfs")]
use crate::cmd::sshfs::SSHFSProcess;

use crate::net::{Host, Subnet};
use crate::ssh::{options::*, *};
use crate::ssh::process::Process;
#[cfg(feature = "wake")]
use crate::waker::Waker;

const CLOUD_FLARE: IpAddr = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));

fn get_resolve_policy(i: IpAddr) -> ResolvePolicy {
    match i {
        IpAddr::V4(..) => ResolvePolicy::ResolveToIPv4,
        IpAddr::V6(..) => ResolvePolicy::ResolveToIPv6,
    }
}

#[derive(Debug, Default)]
pub struct NetworkMap {
    subnets: HashMap<String, Subnet>,
}

impl NetworkMap {
    pub fn add_subnet(&mut self, s: Subnet) {
        self.subnets.insert(s.subdomain.clone(), s);
    }

    pub fn get_host(&self, q: &str) -> Option<&Host> {
        for s in self.subnets.values() {
            match s.get_host(q) {
                None => {}
                Some(h) => return Some(h),
            }
        }
        None
    }

    pub fn is_available(ip: IpAddr, port: Option<u16>) -> bool {
        let rp = get_resolve_policy(ip);
        match match port {
            Some(p) => TcpTarget::new(ip.to_string(), p, Duration::from_millis(2000), rp)
                .check_availability(),
            None => IcmpTarget::new(ip.to_string(), rp).check_availability(),
        } {
            Ok(r) => matches!(r, Status::Available),
            Err(_) => false,
        }
    }

    pub fn get_subnet_by_ip(&self, ip: IpAddr) -> Option<&Subnet> {
        self.subnets.values().find(|s| {
            if s.eip.is_some() {
                s.eip.unwrap() == ip
            } else {
                format!("{}:0", s.subdomain)
                    .to_socket_addrs()
                    .unwrap()
                    .next()
                    .unwrap()
                    .ip()
                    == ip
            }
        })
    }

    pub fn get_masters(&self) -> Vec<(&Subnet, &Host)> {
        self.subnets.values().map(|s| (s, s.get_master())).collect()
    }

    /// Gets master of given host
    pub fn get_host_subnet(&self, h: &Host) -> &Subnet {
        for s in self.subnets.values() {
            if s.has_host(h) {
                return s;
            }
        }
        panic!("host is not in any subnet")
    }

    pub fn get_host_master(&self, h: &Host) -> &Host {
        self.get_host_subnet(h).get_master()
    }

    /// Gets client external ip and returns the optional matched subnet in which the client is.
    pub async fn find_current_subnet(&self) -> Option<&Subnet> {
        // are we online?
        if NetworkMap::is_available(CLOUD_FLARE, Some(80)) {
            debug!("network: we are online");
            debug!("getting external ip");
            match get_ip().await {
                Some(client_eip) => {
                    info!("external ip is {}", client_eip);
                    self.get_subnet_by_ip(client_eip)
                },
                None => {
                    warn!("cannot get external ip");
                    None
                },
            }
        } else {
            debug!("network: we are offline");
            info!("no internet connection detected");
            debug!("detecting subnet using masters...");
            // no, check if some network master is available
            for (s, m) in self.get_masters() {
                if NetworkMap::is_available(m.ip, Some(m.port)) {
                    return Some(s);
                }
                debug!("{} is unavailable", m.get_hop(Some(s)));
            }
            warn!("no internet connection, and not in a known subnet");
            None
        }
    }

    pub async fn hops_gen(&self, target: &Host, subnet: Option<&Subnet>) -> (Hop, Vec<Hop>) {
        fn actual(t_s: &Subnet) -> Vec<Hop> {
            let master = t_s.get_master();
            vec![master.get_hop(Some(t_s))]
        }

        let target_subnet = self.get_host_subnet(target);
        let target_hop = target.get_hop(if let Some(s) = subnet {
            if target_subnet != s && target.is_master() {
                Some(target_subnet)
            } else {
                None
            }
        } else if target.is_master() {
                Some(target_subnet)
        } else {
            None
        });

        let hops= if target.is_master() {
            debug!("router: target is master, connecting directly");
            vec![]
        } else {
            match subnet {
                Some(s) => {
                    debug!("router: we are in a known subnet");
                    if target_subnet == s {
                        debug!("router: client is in target's subnet");
                        vec![]
                    } else {
                        debug!("router: client is in known subnet, a different one from the target");
                        actual(target_subnet)
                    }
                }
                None => {
                    debug!("router: we are in an unknown subnet");
                    actual(target_subnet)
                }
            }
        };
        (target_hop, hops)
    }

    pub fn gen_ssh_options(hops: Vec<Hop>, port: Option<PortOption>, extra_options: Option<SSHOptionStore>) -> SSHOptionStore {
        debug!("generating ssh options");
        let mut options = SSHOptionStore::default();

        if !hops.is_empty() {
            debug!(
                "hops required, adding to options: {:?}",
                hops.iter().map(|h| h.to_string()).collect::<Vec<String>>()
            );
            options.add_option(Box::new(JumpHosts::new(hops)));
        }
        if let Some(p_o) = port {
            debug!("port specification needed, adding to options: {:?}", p_o);
            options.add_option(Box::new(p_o))
        }
        info!("ssh options generated: {:?}", options.args_gen());

        if let Some(o) = extra_options {
            debug!("extra options present, merging");
            options.merge(o);
        }

        options
    }

    #[cfg(feature = "sshfs")]
    pub async fn to_sshfs(
        &self,
        target: &Host,
        subnet: Option<&Subnet>,
        remote: String,
        mountpoint: String
    ) -> Box<dyn Process> {
        debug!("generating route to target");
        let (target_id, route) = self.hops_gen(target, subnet).await;
        info!("route generated: {}", join_hops(&target_id, &route, " -> "));

        Box::new(SSHFSProcess::new(
            target_id.to_string(),
            remote,
            mountpoint,
            NetworkMap::gen_ssh_options(
                route,
                target.port_option(),
                None
            )
        ))
    }

    pub async fn to_ssh(
        &self,
        target: &Host,
        subnet: Option<&Subnet>,
        command: &[String],
        extra_options: Option<SSHOptionStore>,
    ) -> Box<dyn Process> {
        debug!("generating route to target");
        let (target_id, route) = self.hops_gen(target, subnet).await;

        info!("route generated: {}", join_hops(&target_id, &route, " -> "));

        debug!("generating ssh command");
        let mut output = vec!["ssh".to_string()];
        output.append(&mut NetworkMap::gen_ssh_options(
            route,
            target.port_option(),
            extra_options
        ).args_gen());
        output.push(target_id.to_string());
        output.push(command
            .iter()
            .map(|s| if s.contains(' ') { format!("'{}'", s) } else { s.to_owned() })
            .intersperse(" ".to_owned())
            .collect::<String>()
        );
        debug!("generated command: {:?}", output);

        Box::new(SSHProcess::new(output))
    }

    #[cfg(feature = "wake")]
    pub async fn wake(&self, target: &Host) -> Result<(), String> {
        match &target.waker {
            None => {
                info!("won't wake host since it hasn't any waker");
                Ok(())
            }
            Some(w) => match w {
                Waker::HttpWaker { method, url } => {
                    info!("making {} request to {}", method, url);
                    let client = reqwest::Client::new();
                    match client.request(method.clone(), url).send().await {
                        Ok(res) => {
                            debug!("status code of request: {}", res.status());
                            if res.status() == 200 {
                                Ok(())
                            } else {
                                Err(format!("http error: {}", res.status()))
                            }
                        }
                        Err(e) => Err(format!("request error: {}", e)),
                    }
                }
                Waker::WolWaker { mac } => {
                    info!("waking host with mac {} through ssh", mac);
                    let master = self.get_host_master(target);
                    info!("master to execute wake on is {}", master.name);
                    debug!("generating ssh command for wake operation");
                    let mut wake_proc = self
                        .to_ssh(
                            master,
                            None,
                            &vec!["wol".to_string(), mac.to_string()],
                            None,
                        )
                        .await;
                    debug!("ssh waker command is `{}`", wake_proc.to_string());
                    match wake_proc.run_stdout_to_stderr() {
                        Ok(e) => {
                            if let ExitStatus::Exited(n) = e {
                                if n == 0 {
                                    Ok(())
                                } else {
                                    Err(format!("ssh waker exited with code {}", n))
                                }
                            } else {
                                Err(format!("ssh waker ended: {:?}", e))
                            }
                        }
                        Err(e) => Err(format!("{:?}", e)),
                    }
                }
            },
        }
    }
}
