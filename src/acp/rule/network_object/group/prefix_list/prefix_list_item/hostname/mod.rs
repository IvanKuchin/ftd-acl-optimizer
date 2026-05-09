use std::net::IpAddr;
use std::str::FromStr;

use super::ip_range::IPRange;
use super::ipv4::IPv4;
use std::net::ToSocketAddrs;

#[derive(Debug, Clone)]
pub struct Hostname {
    name: String,
    ips: Vec<IPv4>,
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

        let mut ipv4_addresses = Vec::new();
        let mut ipv6_count = 0;

        for addr in addrs_iter {
            let ip = addr.ip();

            match ip {
                IpAddr::V4(ipv4) => {
                    ipv4_addresses.push(IPv4::from(ipv4.to_bits()));
                }
                IpAddr::V6(ipv6) => {
                    ipv6_count += 1;
                    if ipv6_count == 1 {
                        eprintln!("Warning: IPv6 address {} for hostname '{}' is not supported and will be skipped", ipv6, s);
                    }
                }
            }
        }

        // Log additional IPv6 addresses if more than one was encountered
        if ipv6_count > 1 {
            eprintln!(
                "Warning: {} additional IPv6 address(es) for hostname '{}' were skipped",
                ipv6_count - 1,
                s
            );
        }

        if !ipv4_addresses.is_empty() {
            Ok(Hostname {
                name: s.to_string(),
                ips: ipv4_addresses,
            })
        } else {
            Err(HostnameError::NameResolution {
                name: s.to_string(),
            })
        }
    }
}

impl Hostname {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Convert hostname's multiple IPs into individual IPRange items
    pub fn to_ip_ranges(&self) -> Vec<IPRange> {
        self.ips
            .iter()
            .map(|ip| IPRange::new(self.name.clone(), ip.clone(), ip.clone()))
            .collect()
    }

    #[deprecated(
        note = "Hostname with multiple IPs cannot provide single start IP - use to_ip_ranges() instead"
    )]
    pub fn start_ip(&self) -> &IPv4 {
        panic!("Hostname with multiple IPs cannot provide single start/end IP - use to_ip_ranges() instead")
    }

    #[deprecated(
        note = "Hostname with multiple IPs cannot provide single end IP - use to_ip_ranges() instead"
    )]
    pub fn end_ip(&self) -> &IPv4 {
        panic!("Hostname with multiple IPs cannot provide single start/end IP - use to_ip_ranges() instead")
    }

    pub fn capacity(&self) -> u64 {
        self.ips.len() as u64
    }

    #[cfg(test)]
    pub(crate) fn with_ipv4s(name: impl Into<String>, ips: Vec<IPv4>) -> Self {
        Self {
            name: name.into(),
            ips,
        }
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
        assert!(!hostname.ips.is_empty());
        assert!(hostname.ips[0].to_string().parse::<Ipv4Addr>().is_ok());
    }

    #[test]
    fn test_hostname_from_str_valid_ipv4_2() {
        let hostname_str = "outlook.office365.com";
        let hostname = Hostname::from_str(hostname_str).unwrap();

        assert_eq!(hostname.get_name(), hostname_str);
        assert!(!hostname.ips.is_empty());
        assert!(hostname.ips[0].to_string().parse::<Ipv4Addr>().is_ok());
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
            ips: vec![IPv4::from(0)],
        };

        assert_eq!(hostname.get_name(), "example.com");
    }

    #[test]
    fn test_to_ip_ranges() {
        let hostname = Hostname {
            name: "example.com".to_string(),
            ips: vec![IPv4::from(12345), IPv4::from(12346)],
        };

        let ranges = hostname.to_ip_ranges();
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0].get_name(), "example.com");
        assert_eq!(ranges[1].get_name(), "example.com");
        assert_eq!(ranges[0].start_ip(), &IPv4::from(12345));
        assert_eq!(ranges[1].start_ip(), &IPv4::from(12346));
    }

    #[test]
    fn test_capacity() {
        let hostname = Hostname {
            name: "example.com".to_string(),
            ips: vec![IPv4::from(54321), IPv4::from(54322), IPv4::from(54323)],
        };

        assert_eq!(hostname.capacity(), 3);
    }

    #[test]
    fn test_hostname_with_both_ipv4_and_ipv6() {
        // This test demonstrates that IPv6 addresses are skipped with a warning
        // while ALL IPv4 addresses are captured successfully.
        // Note: google.com typically has both IPv4 and IPv6 addresses.
        // If this test fails due to DNS resolution, it may indicate network issues.
        let hostname_str = "google.com";
        let result = Hostname::from_str(hostname_str);

        // The hostname should resolve successfully to IPv4 address(es)
        // even if IPv6 addresses are present (they'll be logged as warnings to stderr)
        assert!(
            result.is_ok(),
            "Expected google.com to resolve to at least one IPv4 address"
        );

        let hostname = result.unwrap();
        assert_eq!(hostname.get_name(), hostname_str);
        assert!(!hostname.ips.is_empty());
        assert!(hostname.ips[0].to_string().parse::<Ipv4Addr>().is_ok());

        // Google typically returns multiple A records
        println!(
            "Resolved {} IPv4 addresses for {}",
            hostname.ips.len(),
            hostname_str
        );
    }
}
