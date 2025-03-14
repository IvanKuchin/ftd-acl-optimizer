use std::fmt;
use std::str::FromStr;

use super::tcp_udp::common;

#[derive(Debug)]
pub struct OtherProtocol {
    name: String,
    protocol: u8,
}

#[derive(thiserror::Error, Debug)]
pub enum OtherProtocolError {
    #[error("Failed to parse port list: {0}")]
    General(String),
    #[error("Failed to parse port list: {0}")]
    CommonError(#[from] common::CommonError),
}

impl fmt::Display for OtherProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (protocol {})", self.name, self.protocol)
    }
}

impl FromStr for OtherProtocol {
    type Err = OtherProtocolError;

    // Example 1
    // protocol 62

    // Example 2
    // IGMP (protocol 2)

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, proto_and_ports) = common::parse_name_and_protocol(s)?;

        let protocol = common::parse_protocol(proto_and_ports)?;

        Ok(Self {
            name: name.to_string(),
            protocol,
        })
    }
}
