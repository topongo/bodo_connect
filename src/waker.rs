use mac_address::MacAddress;
use reqwest::Method;

#[derive(Debug)]
pub enum Waker {
    WolWaker(MacAddress),
    HttpWaker(Method, String)
}

pub(crate) struct WolWaker {
    mac: MacAddress
}

pub(crate) enum HttpWaker {
    Get(String),
    Post(String)
}


