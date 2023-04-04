use std::str::FromStr;
use mac_address::MacAddress;
use reqwest::Method;
use serde::de::Error;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Waker {
    WolWaker {
        #[serde(deserialize_with = "mac_parser")]
        mac: MacAddress
    },
    HttpWaker {
        #[serde(deserialize_with = "method_parser")]
        method: Method,
        url: String
    }
}

pub fn mac_parser<'de, D>(deserializer: D) -> Result<MacAddress, D::Error> where D: Deserializer<'de> {
    let method_string = String::deserialize(deserializer)?;
    MacAddress::from_str(&method_string).map_err(Error::custom)
}

pub fn method_parser<'de, D>(deserializer: D) -> Result<Method, D::Error> where D: Deserializer<'de> {
    let method_string = String::deserialize(deserializer)?;
    Method::from_str(&method_string).map_err(Error::custom)
}