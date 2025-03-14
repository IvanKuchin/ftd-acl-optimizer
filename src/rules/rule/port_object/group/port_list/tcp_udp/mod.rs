use std::fmt;
use std::str::FromStr;

pub mod common;

#[derive(Debug)]
pub struct TcpUdp {
    name: String,
    protocol: u8,
    start: u16,
    end: u16,
}

#[derive(thiserror::Error, Debug)]
pub enum TcpUdpError {
    #[error("Failed to parse port list: {0}")]
    General(String),
    #[error("Failed to parse port list: {0}")]
    CommonError(#[from] common::CommonError),
}

impl fmt::Display for TcpUdp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start == self.end {
            write!(
                f,
                "{} (protocol {}, port {})",
                self.name, self.protocol, self.start
            )
        } else {
            write!(
                f,
                "{} (protocol {}, port {}-{})",
                self.name, self.protocol, self.start, self.end
            )
        }
    }
}

impl FromStr for TcpUdp {
    type Err = TcpUdpError;

    // Example 1
    // protocol 6, port 17444

    // Example 2
    // HTTP (protocol 6, port 80)

    // Example 3
    // HTTP (protocol 6, port 80-81)

    // Example 4
    // HTTP (protocol 6)

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, proto_and_ports) = common::parse_name_and_protocol(s)?;

        let protocol = common::parse_protocol(proto_and_ports)?;

        let (start, end) = parse_ports(proto_and_ports)?;

        Ok(Self {
            name: name.to_string(),
            protocol,
            start,
            end,
        })
    }
}

fn parse_ports(s: &str) -> Result<(u16, u16), TcpUdpError> {
    let mut parts = s.split("port");

    let ports = match parts.nth(1) {
        Some(ports) => ports.trim(),
        None => return Ok((0, 65535)),
    };

    let mut split = ports.split('-');

    let start = split
        .next()
        .ok_or_else(|| TcpUdpError::General(format!("Missing start port ({})", ports)))?
        .trim();

    let start = start
        .parse::<u16>()
        .map_err(|_| TcpUdpError::General(format!("Invalid start port number {}", start)))?;

    let end = split.next();
    let end = match end {
        Some(end) => end
            .trim()
            .parse::<u16>()
            .map_err(|_| TcpUdpError::General(format!("Invalid end port number {}", end)))?,
        None => start,
    };

    Ok((start, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ports_single_port() {
        let input = "protocol 6, port 17444";
        let ports = parse_ports(input).unwrap();
        assert_eq!(ports, (17444, 17444));
    }

    #[test]
    fn test_parse_ports_range() {
        let input = "protocol 6, port 17444-17445";
        let ports = parse_ports(input).unwrap();
        assert_eq!(ports, (17444, 17445));
    }

    #[test]
    fn test_parse_ports_missing_ports() {
        let input = "protocol 6";
        let ports = parse_ports(input).unwrap();
        assert_eq!(ports, (0, 65535));
    }

    #[test]
    fn test_parse_ports_invalid_ports() {
        let input = "protocol 6, port 17444-";
        let result = parse_ports(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_single_port() {
        let input = "protocol 6, port 17444";
        let port_list = input.parse::<TcpUdp>().unwrap();
        assert_eq!(port_list.name, "protocol 6, port 17444");
        assert_eq!(port_list.protocol, 6);
        assert_eq!(port_list.start, 17444);
        assert_eq!(port_list.end, 17444);
    }

    #[test]
    fn test_named_single_port() {
        let input = "HTTP (protocol 6, port 80)";
        let port_list = input.parse::<TcpUdp>().unwrap();
        assert_eq!(port_list.name, "HTTP");
        assert_eq!(port_list.protocol, 6);
        assert_eq!(port_list.start, 80);
        assert_eq!(port_list.end, 80);
    }

    #[test]
    fn test_named_port_range() {
        let input = "HTTP (protocol 6, port 80-81)";
        // let port_list = input.parse::<PortList>().unwrap();
        let port_list = TcpUdp::from_str(input).unwrap();
        assert_eq!(port_list.name, "HTTP");
        assert_eq!(port_list.protocol, 6);
        assert_eq!(port_list.start, 80);
        assert_eq!(port_list.end, 81);
    }

    #[test]
    fn test_invalid_format() {
        let input = "Invalid format";
        let result = input.parse::<TcpUdp>();
        assert!(result.is_err());
    }
    #[test]
    fn test_empty_string() {
        let input = "";
        let result = input.parse::<TcpUdp>();
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_protocol() {
        let input = "HTTP (port 80)";
        let result = input.parse::<TcpUdp>();
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_port() {
        let input = "TCP (protocol 6)";
        let port_list = input.parse::<TcpUdp>().unwrap();
        assert_eq!(port_list.name, "TCP");
        assert_eq!(port_list.protocol, 6);
        assert_eq!(port_list.start, 0);
        assert_eq!(port_list.end, 65535);
    }

    #[test]
    fn test_invalid_port_range() {
        let input = "HTTP (protocol 6, port 81-)";
        let result = input.parse::<TcpUdp>();
        assert!(result.is_err());
    }

    #[test]
    fn test_extra_whitespace() {
        let input = "  HTTP  (  protocol 6 ,  port 80-81  )  ";
        let port_list = input.parse::<TcpUdp>().unwrap();
        assert_eq!(port_list.name, "HTTP");
        assert_eq!(port_list.protocol, 6);
        assert_eq!(port_list.start, 80);
        assert_eq!(port_list.end, 81);
    }

    #[test]
    fn test_invalid_protocol() {
        let input = "HTTP (protocol six, port 80)";
        let result = input.parse::<TcpUdp>();
        assert!(result.is_err());
    }
}
