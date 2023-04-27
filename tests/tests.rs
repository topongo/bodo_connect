#![cfg(feature = "serde")]

use futures::executor::block_on;
use bodo_connect::net::{Subnet, NetworkMap};

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
        #[cfg(feature = "wake")]
        Some(Waker::HttpWaker {method: Method::GET, url: "https://example.com".to_string()})
    ));

    let nm = NetworkMap::try_from(vec![subnet]).unwrap();
    assert_eq!(block_on(nm.to_ssh(
        nm.get_host("x").unwrap(),
        None,
        &mut vec!["rm", "-rf", "/"],
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

    for _ in 0..10 {
        let current_subnet = nm.find_current_subnet().await;
        match current_subnet {
            Some(_) => {},
            None => {}
        }

        let proc = nm.to_ssh(nm.get_host("mars").unwrap(), current_subnet, &mut vec!["echo"], None).await;
        assert_eq!(proc.to_string(), "ssh martian@example.com echo");
    }
}
