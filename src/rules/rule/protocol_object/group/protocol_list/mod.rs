use std::fmt;
use std::str::FromStr;

use tcp_udp::common;

mod icmp;
mod other_protocol;
pub mod tcp_udp;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ProtocolList {
    Icmp(icmp::Icmp),
    TcpUdp(tcp_udp::TcpUdp),
    OtherProtocol(other_protocol::OtherProtocol),
}

#[derive(thiserror::Error, Debug)]
pub enum PortListError {
    #[error("Failed to parse port list: {0}")]
    General(String),
    #[error("Failed to parse port list: {0}")]
    IcmpError(#[from] icmp::IcmpError),
    #[error("Failed to parse port list: {0}")]
    TcpUdpError(#[from] tcp_udp::TcpUdpError),
    #[error("Failed to parse port list: {0}")]
    OtherProtocolError(#[from] other_protocol::OtherProtocolError),
    #[error("Failed to parse port list: {0}")]
    CommonError(#[from] common::CommonError),
}

impl fmt::Display for ProtocolList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolList::TcpUdp(tcp_udp) => write!(f, "{}", tcp_udp),
            ProtocolList::OtherProtocol(other_protocol) => write!(f, "{}", other_protocol),
            ProtocolList::Icmp(icmp) => write!(f, "{}", icmp),
        }
    }
}

impl FromStr for ProtocolList {
    type Err = PortListError;

    // Example 1
    // protocol 6, port 17444

    // Example 2
    // FTP (protocol 6, port 20-21)

    // Example 3
    // DNS (protocol 17, port 53)

    // Example 4
    // IGMP (protocol 2)

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_name, ports) = common::parse_name_and_protocol(s)?;

        let protocol = common::parse_protocol(ports)?;

        match protocol {
            6 | 17 => {
                let tcp_udp = tcp_udp::TcpUdp::from_str(s)?;
                Ok(Self::TcpUdp(tcp_udp))
            }
            1 | 58 => {
                let icmp = icmp::Icmp::from_str(s)?;
                Ok(Self::Icmp(icmp))
            }
            _ => {
                let other_protocol = other_protocol::OtherProtocol::from_str(s)?;
                Ok(Self::OtherProtocol(other_protocol))
            }
        }
    }
}

impl ProtocolList {
    pub fn is_l4(&self) -> bool {
        match self {
            ProtocolList::TcpUdp(tcp_udp) => tcp_udp.is_l4(),
            ProtocolList::OtherProtocol(other_protocol) => other_protocol.is_l4(),
            ProtocolList::Icmp(icmp) => icmp.is_l4(),
        }
    }
    pub fn get_name(&self) -> &str {
        match self {
            ProtocolList::TcpUdp(tcp_udp) => tcp_udp.get_name(),
            ProtocolList::OtherProtocol(other_protocol) => other_protocol.get_name(),
            ProtocolList::Icmp(icmp) => icmp.get_name(),
        }
    }
    pub fn get_protocol(&self) -> u8 {
        match self {
            ProtocolList::TcpUdp(tcp_udp) => tcp_udp.get_protocol(),
            ProtocolList::OtherProtocol(other_protocol) => other_protocol.get_protocol(),
            ProtocolList::Icmp(icmp) => icmp.get_protocol(),
        }
    }
    pub fn get_ports(&self) -> (u16, u16) {
        match self {
            ProtocolList::TcpUdp(tcp_udp) => tcp_udp.get_ports(),
            _ => (0, 0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_port() {
        let port_list = ProtocolList::from_str("protocol 6, port 17444").unwrap();
        assert_eq!(
            port_list.to_string(),
            "protocol 6, port 17444 (protocol 6, port 17444)"
        );
    }

    #[test]
    fn test_ftp_ports() {
        let port_list = ProtocolList::from_str("FTP (protocol 6, port 20-21)").unwrap();
        assert_eq!(port_list.to_string(), "FTP (protocol 6, port 20-21)");
    }

    #[test]
    fn test_dns_port() {
        let port_list = ProtocolList::from_str("DNS (protocol 17, port 53)").unwrap();
        assert_eq!(port_list.to_string(), "DNS (protocol 17, port 53)");
    }

    #[test]
    fn test_igmp() {
        let port_list = ProtocolList::from_str("IGMP (protocol 2)").unwrap();
        assert_eq!(port_list.to_string(), "IGMP (protocol 2)");
    }

    #[test]
    fn test_igmp_with_ports() {
        let port_list = ProtocolList::from_str("IGMP (protocol 2, ports 123)").unwrap();
        assert_eq!(port_list.to_string(), "IGMP (protocol 2)");
    }

    #[test]
    fn test_invalid_protocol() {
        assert!(ProtocolList::from_str("Invalid (protocol 999, port 80)").is_err());
    }

    #[test]
    fn test_malformed_input() {
        assert!(ProtocolList::from_str("malformed input").is_err());
    }
}
