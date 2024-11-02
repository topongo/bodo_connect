#![cfg(feature = "serde")]

use std::collections::HashSet;

use futures::executor::block_on;
use bodo_connect::{net::{NetworkMap, Subnet}, ssh::{options::GenericOption, SSHOptionStore}};

const NETWORKMAP_EXAMPLE: &str = r#"
[
  {
    "subdomain": "earth.orbit",
    "eip": "1.2.3.4",
    "hosts": [
      {
        "name": "earth",
        "uuid": "---",
        "ip": "10.0.0.1",
        "port": 22,
        "eport": 22,
        "user": "human"
      },
      {
        "name": "moon",
        "ip": "10.0.0.2",
        "uuid": "---",
        "port": 444,
        "user": "rock",
        "waker": {
          "method": "get",
          "url": "https://earth.orbit/api/v1/wake/moon/"
        }
      }
    ]
  },
  {
    "subdomain": "example.com",
    "eip": null,
    "hosts": [
      {
        "name": "mars",
        "uuid": "---",
        "ip": "0.0.0.0",
        "port": 22,
        "eport": 22,
        "user": "martian"
      },
      {
        "name": "phobos",
        "ip": "192.168.1.2",
        "uuid": "---",
        "port": 444,
        "user": "rover",
        "waker": {
          "mac": "00:08:55:05:ef:87"
        }
      }
    ]
  }
]
"#;

#[cfg(feature = "wake")]
#[test]
fn migration_test() {
    use bodo_connect::net::Subnet;
    use bodo_connect::waker::Waker;
    use reqwest::Method;
    
    let map: Vec<Subnet> = serde_json::from_str(NETWORKMAP_EXAMPLE).unwrap();
    toml::to_string(&Waker::HttpWaker {method: Method::GET, url: "https://example.com".to_string()}).unwrap();
    let h = map[0].get_hosts()[0];
    println!("{:?}", h);
    println!("{}", serde_json::to_string_pretty(h).unwrap());
    toml::to_string(&map[0].get_hosts()[0]).unwrap();
    toml::to_string(&map).unwrap();
}

#[test]
fn parsing_test() {
    use std::net::IpAddr;
    use std::str::FromStr;
    use bodo_connect::net::Host;
    #[cfg(feature = "wake")]
    use reqwest::Method;
    use bodo_connect::net::Subnet;
    #[cfg(feature = "wake")]
    use bodo_connect::waker::Waker;

    let input = r#"
    {
        "name": "mars",
        "ip": "10.0.0.1",
        "port": 22,
        "eport": 23,
        "user": "martian"
    }
    "#;

    let host: Host = serde_json::from_str(input).unwrap();
    assert_eq!(host.name, "mars");
    assert_eq!(host.ip, IpAddr::from_str("10.0.0.1").unwrap());
    assert_eq!(host.port, 22);
    assert!(host.eport.is_some());
    assert_eq!(host.eport.unwrap(), 23);
    assert_eq!(host.user, "martian");

    let input = r#"
    {
        "subdomain": "mars.orbit",
        "eip": null,
        "hosts": []
    }
    "#;

    let mut subnet: Subnet = serde_json::from_str(input).unwrap();
    subnet.add_host(host);
    subnet.add_host(Host::new(
        "x".to_string(),
        "x".to_string(),
        IpAddr::from_str("10.8.5.2").unwrap(),
        5,
        None,
        HashSet::new(),
        #[cfg(feature = "wake")]
        Some(Waker::HttpWaker {method: Method::GET, url: "https://example.com".to_string()})
    ));

    let nm = NetworkMap::try_from(vec![subnet]).unwrap();
    assert_eq!(block_on(nm.to_ssh(
        nm.get_host("x").unwrap(),
        None,
        &["rm".to_owned(), "-rf".to_owned(), "/".to_owned()],
        None
    )).to_string(), "ssh -J martian@mars.orbit:23 -p 5 x@10.8.5.2 rm -rf /");
}

#[cfg(feature = "sshfs")]
#[test]
fn sshfs() {
    let nm = NetworkMap::try_from(serde_json::from_str::<Vec<Subnet>>(NETWORKMAP_EXAMPLE).unwrap()).unwrap();
    let target = nm.get_host("phobos").unwrap();
    let proc = block_on(nm.to_sshfs(target, None, "/home/pi".to_string(), "/mnt/temp".to_string()));
    println!("{}", proc)
}

#[tokio::test]
async fn uncertainty() {
    let nm: NetworkMap = NetworkMap::try_from(serde_json::from_str::<Vec<Subnet>>(NETWORKMAP_EXAMPLE).unwrap()).unwrap();

    for _ in 0..3 {
        let current_subnet = nm.find_current_subnet().await;

        let proc = nm.to_ssh(nm.get_host("mars").unwrap(), current_subnet, &["echo".to_owned()], None).await;
        assert_eq!(proc.to_string(), "ssh martian@example.com echo");
    }
}

#[test]
fn custom_ssh_command() {
    let nm: NetworkMap = NetworkMap::try_from(serde_json::from_str::<Vec<Subnet>>(NETWORKMAP_EXAMPLE).unwrap()).unwrap();

    let mars = nm.get_host("phobos").unwrap();
    let _sub = nm.get_host_subnet(mars);
    let mut opts = SSHOptionStore::new(Some("ssh -L 8000:localhost:5000".to_owned()));
    opts.add_option(Box::new(GenericOption::Switch("v")));
    let ssh = block_on(nm.to_ssh(mars, None, &vec!["echo".to_owned()], Some(opts)));
    assert_eq!(ssh.to_string(), "ssh -L 8000:localhost:5000 -J martian@example.com -p 444 -v rover@192.168.1.2 echo");

    // Use rsh for insecure but fastest connection
    let mut opts = SSHOptionStore::new(Some("rsh --debug".to_owned()));
    opts.add_option(Box::new(GenericOption::Value("escape", "~".to_owned())));
    let ssh = block_on(nm.to_ssh(
        nm.get_host("mars").unwrap(),
        None,
        &["echo".to_owned()],
        Some(opts)
    ));
    assert_eq!(ssh.to_string(), "rsh --debug --escape ~ martian@example.com echo");
}

