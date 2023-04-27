#![cfg_attr(not(feature = "log"), allow(unused_variables))]
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;
#[cfg(feature = "log")]
use log::debug;
#[cfg(not(feature = "log"))]
use crate::debug;

pub async fn get_ip() -> Option<IpAddr> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(1))
        .build()
        .unwrap();
    match client.get("https://icanhazip.com/").send().await {
        Ok(res) => match res.text().await {
            Ok(s) => match IpAddr::from_str(s.trim()) {
                Ok(i) => return Some(i),
                Err(e) => debug!("{:?} while parsing {}", e, s)
            }
            Err(e) => debug!("{:?}", e)
        },
        Err(e) => debug!("{:?}", e)
    }
    None
}