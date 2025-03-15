use std::fmt;
use std::hash::{Hash, Hasher};
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

impl OtherProtocol {
    pub fn is_l4(&self) -> bool {
        false
    }
    pub fn get_protocol(&self) -> u8 {
        self.protocol
    }
}

impl PartialEq for OtherProtocol {
    fn eq(&self, other: &Self) -> bool {
        self.protocol == other.protocol
    }
}

impl Eq for OtherProtocol {}

impl Hash for OtherProtocol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.protocol.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_protocol_only() {
        let input = "protocol 62";
        let port_obj = OtherProtocol::from_str(input).unwrap();
        assert_eq!(port_obj.name, "protocol 62");
        assert_eq!(port_obj.protocol, 62);
    }

    #[test]
    fn parse_with_name() {
        let input = "IGMP (protocol 2)";
        let port_obj = OtherProtocol::from_str(input).unwrap();
        assert_eq!(port_obj.name, "IGMP");
        assert_eq!(port_obj.protocol, 2);
    }

    #[test]
    fn parse_invalid_format() {
        let input = "invalid format";
        assert!(OtherProtocol::from_str(input).is_err());
    }

    #[test]
    fn parse_invalid_protocol() {
        let input = "Test (protocol abc)";
        assert!(OtherProtocol::from_str(input).is_err());
    }

    #[test]
    fn test_display() {
        let port_obj = OtherProtocol {
            name: "IGMP".to_string(),
            protocol: 2,
        };
        assert_eq!(port_obj.to_string(), "IGMP (protocol 2)");
    }

    #[test]
    fn test_is_mergable() {
        let port_obj = OtherProtocol {
            name: "IGMP".to_string(),
            protocol: 2,
        };
        assert!(!port_obj.is_l4());
    }
}
