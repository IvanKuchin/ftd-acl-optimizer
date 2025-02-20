use std::str::FromStr;
use std::cmp::Ordering;

#[derive(Debug, Eq, PartialEq)]
pub struct IPv4(pub u64);

#[derive(thiserror::Error, Debug)]
pub enum IPv4Error {
    #[error("Fail to parse ipv4 address: {0}")]
    General(String),

    #[error("Failed to parse IPv4 address: {0}")]
    ParseError(#[from] std::num::ParseIntError),
}

impl FromStr for IPv4 {
    type Err = IPv4Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ip_parts: Vec<_> = s
            .split(".")
            .map(|s| s.parse::<u64>().map_err(|e| IPv4Error::ParseError(e)))
            .collect::<Result<Vec<u64>, IPv4Error>>()?;
        if ip_parts.len() != 4 {
            return Err(IPv4Error::General(format!("Invalid IP format (expected IPv4) in {}", &s).to_string()));
        }
        
        for &part in &ip_parts {
            if part > 255 {
                return Err(IPv4Error::General(format!("IP parts must be in the range 0-255 in {}", &s).to_string()));
            }
        }

        let ip = ip_parts[0] << 24 | ip_parts[1] << 16 | ip_parts[2] << 8 | ip_parts[3];
        Ok(IPv4(ip))
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
    pub fn get_broadcast(&self, mask: u64) -> IPv4 {
        Self(self.0 | ((1 << (32 - mask)) - 1))
    }
    
    pub fn get_network(&self, mask: u64) -> IPv4 {
        Self(self.0 & ((!0u64) << (32 - mask)))
    }
}
