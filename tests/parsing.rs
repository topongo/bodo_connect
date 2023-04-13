#![cfg(feature = "serde")]

use futures::executor::block_on;
use bodo_connect::net::NetworkMap;

#[test]
fn parsing_test() {
    use std::net::IpAddr;
    use std::str::FromStr;
    use bodo_connect::net::Host;
    use reqwest::Method;
    use bodo_connect::net::Subnet;
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
