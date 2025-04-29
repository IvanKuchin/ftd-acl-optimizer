use std::net::IpAddr;
use std::str::FromStr;

use super::ipv4::IPv4;
use std::net::ToSocketAddrs;

#[derive(Debug, Clone)]
pub struct Hostname {
    name: String,
    start: IPv4,
    end: IPv4,
}

#[derive(thiserror::Error, Debug)]
pub enum HostnameError {
    #[error("Fail to resolve name: {name}")]
    NameResolution { name: String },
    #[error("IPv6 not supported: {addr}")]
    IPv6NotSupported { addr: String },
    #[error("Transit error in Hostname from Io: {0}")]
    Io(#[from] std::io::Error),
}

impl FromStr for Hostname {
    type Err = HostnameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let addrs_iter = format!("{s}:443").to_socket_addrs()?;
        for addr in addrs_iter {
            let ip = addr.ip();

            if let IpAddr::V4(ipv4) = ip {
                let start = IPv4::from(ipv4.to_bits());
                let end = start.clone();
                return Ok(Hostname {
                    name: s.to_string(),
                    start,
                    end,
                });
            }
        }

        Err(HostnameError::NameResolution {
            name: s.to_string(),
        })
    }
}

impl Hostname {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn start_ip(&self) -> &IPv4 {
        &self.start
    }

    pub fn end_ip(&self) -> &IPv4 {
        &self.end
    }

    pub fn capacity(&self) -> u64 {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_hostname_from_str_valid_ipv4() {
        let hostname_str = "ipv4.net";
        let hostname = Hostname::from_str(hostname_str).unwrap();

        assert_eq!(hostname.get_name(), hostname_str);
        assert!(hostname.start_ip().to_string().parse::<Ipv4Addr>().is_ok());
        assert_eq!(hostname.start_ip(), hostname.end_ip());
    }

    #[test]
    fn test_hostname_from_str_valid_ipv4_2() {
        let hostname_str = "outlook.office365.com";
        let hostname = Hostname::from_str(hostname_str).unwrap();

        assert_eq!(hostname.get_name(), hostname_str);
        assert!(hostname.start_ip().to_string().parse::<Ipv4Addr>().is_ok());
        assert_eq!(hostname.start_ip(), hostname.end_ip());
    }

    #[test]
    fn test_hostname_from_str_invalid_name() {
        let invalid_hostname = "invalid_hostname";
        let result = Hostname::from_str(invalid_hostname);

        assert!(result.is_err());
    }

    #[test]
    fn test_hostname_from_str_ipv6_not_supported() {
        let ipv6_hostname = "[::1]";
        let result = Hostname::from_str(ipv6_hostname);

        assert!(result.is_err());
        dbg!(&result);
        if let Err(HostnameError::NameResolution { name }) = result {
            assert_eq!(name, "[::1]");
        } else {
            panic!("Expected IPv6NotSupported error");
        }
    }

    #[test]
    fn test_get_name() {
        let hostname = Hostname {
            name: "example.com".to_string(),
            start: IPv4::from(0),
            end: IPv4::from(0),
        };

        assert_eq!(hostname.get_name(), "example.com");
    }

    #[test]
    fn test_start_ip() {
        let start_ip = IPv4::from(12345);
        let hostname = Hostname {
            name: "example.com".to_string(),
            start: start_ip.clone(),
            end: start_ip.clone(),
        };

        assert_eq!(hostname.start_ip(), &start_ip);
    }

    #[test]
    fn test_end_ip() {
        let end_ip = IPv4::from(54321);
        let hostname = Hostname {
            name: "example.com".to_string(),
            start: end_ip.clone(),
            end: end_ip.clone(),
        };

        assert_eq!(hostname.end_ip(), &end_ip);
    }
}
