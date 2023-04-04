// use std::error::Error;
// use std::fmt::{Debug, Display, Formatter};
//
// pub(crate) struct Ip {
//     bytes: Vec<u8>
// }
//
// #[derive(Debug)]
// pub(crate) struct InvalidIp {
//
// }
//
// impl Display for InvalidIp {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }
//
// impl Error for InvalidIp {
//
// }
//
// impl Ip {
//     pub fn new(bytes: Vec<u8>) -> Ip {
//         if bytes.len() != 4 { panic!("ip address with more than 4 bytes") }
//         Ip { bytes }
//     }
//
//     pub fn from_string(st: String) -> Result<Ip, InvalidIp> {
//         let mut bytes: Vec<u8> = Vec::new();
//         for i in st.split(".") {
//             bytes.push(i.parse().unwrap())
//         }
//         Ok(Ip::new(bytes))
//     }
//
//     pub fn to_string(&self) -> String {
//         self.bytes
//             .iter()
//             .map(|b| b.to_string())
//             .collect::<Vec<String>>()
//             .join(".")
//     }
// }
//
// impl Debug for Ip {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Ip {{ {} }}", self.to_string())
//     }
// }
