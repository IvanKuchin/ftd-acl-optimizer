use std::cmp::Ordering;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct IPv4(pub u64);

#[derive(thiserror::Error, Debug)]
pub enum IPv4Error {
    #[error("Fail to parse ipv4 address: {0}")]
    General(String),

    #[error("Failed to parse IPv4 address: {0}")]
    ParseError(#[from] std::num::ParseIntError),
}

impl Display for IPv4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let octets = [
            (self.0 >> 24) as u8,
            (self.0 >> 16) as u8,
            (self.0 >> 8) as u8,
            self.0 as u8,
        ];
        write!(f, "{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3])
    }
}

impl FromStr for IPv4 {
    type Err = IPv4Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ip_parts: Vec<_> = s
            .split(".")
            .map(|s| s.parse::<u64>().map_err(IPv4Error::ParseError))
            .collect::<Result<Vec<u64>, IPv4Error>>()?;
        if ip_parts.len() != 4 {
            return Err(IPv4Error::General(
                format!("Invalid IP format (expected IPv4) in {}", &s).to_string(),
            ));
        }

        for &part in &ip_parts {
            if part > 255 {
                return Err(IPv4Error::General(
                    format!("IP parts must be in the range 0-255 in {}", &s).to_string(),
                ));
            }
        }

        let ip = (ip_parts[0] << 24) | (ip_parts[1] << 16) | (ip_parts[2] << 8) | ip_parts[3];
        Ok(IPv4(ip))
    }
}

impl From<&IPv4> for u32 {
    fn from(me: &IPv4) -> Self {
        me.0 as u32
    }
}

impl From<u32> for IPv4 {
    fn from(me: u32) -> Self {
        IPv4(me as u64)
    }
}

use core::cmp::PartialOrd;

impl PartialOrd for IPv4 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

use core::cmp::Ord;

impl Ord for IPv4 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl IPv4 {
    pub fn get_broadcast(&self, mask_length: u8) -> IPv4 {
        Self(self.0 | ((1 << (32 - mask_length)) - 1))
    }

    pub fn get_network(&self, mask_length: u8) -> IPv4 {
        Self(self.0 & ((!0u64) << (32 - mask_length)))
    }

    pub fn next(&self) -> IPv4 {
        Self(self.0 + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_from_str_valid() {
        assert_eq!("192.168.0.1".parse::<IPv4>().unwrap(), IPv4(0xC0A80001));
        assert_eq!("0.0.0.0".parse::<IPv4>().unwrap(), IPv4(0x00000000));
        assert_eq!("255.255.255.255".parse::<IPv4>().unwrap(), IPv4(0xFFFFFFFF));
    }

    #[test]
    fn test_ipv4_from_str_invalid_format() {
        assert!("192.168.0".parse::<IPv4>().is_err());
        assert!("192.168.0.1.1".parse::<IPv4>().is_err());
        assert!("192.168.0.abc".parse::<IPv4>().is_err());
    }

    #[test]
    fn test_ipv4_from_str_invalid_range() {
        assert!("256.168.0.1".parse::<IPv4>().is_err());
        assert!("192.168.0.256".parse::<IPv4>().is_err());
    }

    #[test]
    fn test_ipv4_ordering() {
        let ip1 = "192.168.0.1".parse::<IPv4>().unwrap();
        let ip2 = "192.168.0.2".parse::<IPv4>().unwrap();
        assert!(ip1 < ip2);
        assert!(ip2 > ip1);
        assert_eq!(ip1, ip1);
    }

    #[test]
    fn test_ipv4_partial_cmp() {
        let ip1 = "10.0.0.1".parse::<IPv4>().unwrap();
        let ip2 = "10.0.0.2".parse::<IPv4>().unwrap();
        assert_eq!(ip1.partial_cmp(&ip2), Some(Ordering::Less));
        assert_eq!(ip2.partial_cmp(&ip1), Some(Ordering::Greater));
        assert_eq!(ip1.partial_cmp(&ip1), Some(Ordering::Equal));
    }

    #[test]
    fn test_ipv4_cmp() {
        let ip1 = "172.16.0.1".parse::<IPv4>().unwrap();
        let ip2 = "172.16.0.2".parse::<IPv4>().unwrap();
        assert_eq!(ip1.cmp(&ip2), Ordering::Less);
        assert_eq!(ip2.cmp(&ip1), Ordering::Greater);
        assert_eq!(ip1.cmp(&ip1), Ordering::Equal);
    }

    // #[test]
    // fn test_ipv4_get_broadcast() {
    //     let ip = "192.168.1.0".parse::<IPv4>().unwrap();
    //     let broadcast = ip.get_broadcast(24);
    //     assert_eq!(broadcast, "192.168.1.255".parse::<IPv4>().unwrap());
    // }

    // #[test]
    // fn test_ipv4_get_network() {
    //     let ip = "192.168.1.128".parse::<IPv4>().unwrap();
    //     let network = ip.get_network(24);
    //     assert_eq!(network, "192.168.1.0".parse::<IPv4>().unwrap());
    // }
}
