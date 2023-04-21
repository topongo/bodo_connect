use mac_address::MacAddress;
use reqwest::Method;
#[cfg(feature = "serde")]
use serde::{de::Error, {Deserialize, Deserializer}};
#[cfg(feature = "serde")]
use std::str::FromStr;

#[cfg_attr(feature = "serde", derive(Deserialize))]
#[derive(Debug)]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Waker {
    WolWaker {
        #[cfg_attr(feature = "serde", serde(deserialize_with = "mac_parser"))]
        mac: MacAddress,
    },
    HttpWaker {
        #[cfg_attr(feature = "serde", serde(deserialize_with = "method_parser"))]
        method: Method,
        url: String,
    },
}

#[cfg(feature = "serde")]
pub fn mac_parser<'de, D>(deserializer: D) -> Result<MacAddress, D::Error>
where
    D: Deserializer<'de>,
{
    let mac_string = String::deserialize(deserializer)?;
    MacAddress::from_str(&mac_string).map_err(Error::custom)
}

#[cfg(feature = "serde")]
pub fn method_parser<'de, D>(deserializer: D) -> Result<Method, D::Error>
where
    D: Deserializer<'de>,
{
    let method_string = String::deserialize(deserializer)?;
    Method::from_str(&method_string).map_err(Error::custom)
}
