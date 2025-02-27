use mac_address::MacAddress;
use reqwest::Method;
#[cfg(feature = "serde")]
use serde::{de::Error, {Deserialize, Deserializer, Serialize, Serializer}};
#[cfg(feature = "serde")]
use std::str::FromStr;

#[cfg_attr(feature = "serde", derive(Deserialize,Serialize))]
#[derive(Debug)]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Waker {
    WolWaker {
        #[cfg_attr(feature = "serde", serde(deserialize_with = "mac_parser", serialize_with = "mac_serializer"))]
        mac: MacAddress,
    },
    HttpWaker {
        #[cfg_attr(feature = "serde", serde(deserialize_with = "method_parser", serialize_with = "method_serializer"))]
        method: Method,
        url: String,
    },
}

#[cfg(feature = "serde")]
pub fn mac_parser<'de, D>(deserializer: D) -> Result<MacAddress, D::Error>
where
    D: Deserializer<'de>,
{
    let method_string = String::deserialize(deserializer)?;
    MacAddress::from_str(&method_string).map_err(Error::custom)
}

#[cfg(feature = "serde")]
pub fn method_parser<'de, D>(deserializer: D) -> Result<Method, D::Error>
where
    D: Deserializer<'de>,
{
    let method_string = String::deserialize(deserializer)?;
    Method::from_str(&method_string.to_uppercase()).map_err(Error::custom)
}

#[cfg(feature = "serde")]
pub fn method_serializer<S>(m: &Method, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(m.as_ref())
}

#[cfg(feature = "serde")]
pub fn mac_serializer<S>(m: &MacAddress, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&m.to_string())
}
