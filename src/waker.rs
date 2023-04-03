use std::fmt::Formatter;
use std::str::FromStr;
use mac_address::MacAddress;
use reqwest::Method;
use serde::de::{Visitor, Error, Unexpected};
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
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

pub(crate) struct WolWaker {
    mac: MacAddress
}

pub(crate) enum HttpWaker {
    Get(String),
    Post(String)
}

pub fn mac_parser<'de, D>(deserializer: D) -> Result<MacAddress, D::Error>
    where
        D: Deserializer<'de>
{
    struct MacVisitor;

    impl<'de> Visitor<'de> for MacVisitor {
        type Value = MacAddress;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a mac address, e.g. 05:8d:fe:86:21")
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E> where E: Error {
            match MacAddress::from_str(&v) {
                Ok(m) => Ok(m),
                Err(e) => Err(Error::invalid_value(Unexpected::Str(&v), &self))
            }
        }
    }

    const FIELDS: &[&str] = &["mac"];
    deserializer.deserialize_struct("MacAddress", FIELDS, MacVisitor)
}

pub fn method_parser<'de, D>(deserializer: D) -> Result<Method, D::Error>
    where
        D: Deserializer<'de>
{
    struct MethodVisitor;

    impl<'de> Visitor<'de> for MethodVisitor {
        type Value = Method;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a http method, like GET, POST, ecc.")
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E> where E: Error {
            match Method::from_str(&v) {
                Ok(m) => Ok(m),
                Err(e) => Err(Error::invalid_value(Unexpected::Str(&v), &self))
            }
        }
    }

    const FIELDS: &[&str] = &["method"];
    deserializer.deserialize_struct("Method", FIELDS, MethodVisitor)
}