use std::str::FromStr;

use super::{
    ipv4::{IPv4, IPv4Error},
    prefix,
    prefix::Prefix,
};

#[derive(Debug, Clone)]
pub struct IPRange {
    name: String,
    start: IPv4,
    end: IPv4,
}

#[derive(thiserror::Error, Debug)]
pub enum IPRangeError {
    #[error("Fail to parse ip range: {0}")]
    General(String),

    #[error("Failed to parse IPv4 address: {0}")]
    IPv4Error(#[from] IPv4Error),
}

impl FromStr for IPRange {
    type Err = IPRangeError;

    // String example:
    // 10.18.46.62-10.18.46.69
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let name = String::from(s);
        let parts: Vec<_> = s.split("-").collect();
        if parts.len() != 2 {
            return Err(IPRangeError::General(
                format!(
                    "Invalid ip range format (expected ..ipv4.. - ..ipv4..) in {}.",
                    &name
                )
                .to_string(),
            ));
        }

        let start = parts[0].trim().parse::<IPv4>()?;
        let end = parts[1].trim().parse::<IPv4>()?;

        if start > end {
            return Err(IPRangeError::General(
                format!(
                    "Start IP must be less than or equal to end IP in {}.",
                    &name
                )
                .to_string(),
            ));
        }

        Ok(IPRange { name, start, end })
    }
}

impl IPRange {
    pub fn new(name: String, start: IPv4, end: IPv4) -> Self {
        if start > end {
            panic!("Start IP must be less than or equal to end IP in {}.", name);
        }
        IPRange { name, start, end }
    }

    pub fn capacity(&self) -> u64 {
        let subnets = split_ip_range_into_prefixes(&self.start, &self.end);

        subnets.len() as u64
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn start_ip(&self) -> &IPv4 {
        &self.start
    }

    pub fn end_ip(&self) -> &IPv4 {
        &self.end
    }
}

fn split_ip_range_into_prefixes(start: &IPv4, end: &IPv4) -> Vec<Prefix> {
    let mut prefixes = Vec::new();
    let mut current_ip = start.clone();

    while current_ip <= *end {
        let mut mask = 0u8;

        // Find the largest valid mask for the current IP range
        while mask <= 32 {
            let network_start = current_ip.get_network(mask);
            let network_end = current_ip.get_broadcast(mask);

            if network_start == current_ip && network_end <= *end {
                // Valid prefix found
                break;
            }

            mask += 1;
        }

        // Build the prefix and add it to the list
        let prefix =
            prefix::Builder::new(format!("{}/{}", current_ip, mask), current_ip.clone(), mask)
                .build();

        prefixes.push(prefix);

        // Move to the next IP range
        let next_ip = current_ip.get_broadcast(mask).next();
        if next_ip > *end {
            break;
        }
        current_ip = next_ip;
    }

    prefixes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ip_range() {
        let ip_range_str = "10.18.46.62-10.18.46.69";
        let ip_range = ip_range_str.parse::<IPRange>();
        assert!(ip_range.is_ok());
        let ip_range = ip_range.unwrap();
        assert_eq!(ip_range.name, ip_range_str);
        assert_eq!(
            ip_range.start,
            IPv4((10 << 24) | (18 << 16) | (46 << 8) | 62)
        );
        assert_eq!(ip_range.end, IPv4((10 << 24) | (18 << 16) | (46 << 8) | 69));
    }

    #[test]
    fn test_invalid_ip_range_format() {
        let ip_range_str = "10.18.46.62_10.18.46.69";
        let ip_range = ip_range_str.parse::<IPRange>();
        assert!(ip_range.is_err());
        assert_eq!(
            format!("{}", ip_range.unwrap_err()),
            "Fail to parse ip range: Invalid ip range format (expected ..ipv4.. - ..ipv4..) in 10.18.46.62_10.18.46.69."
        );
    }

    #[test]
    fn test_invalid_start_ip_format() {
        let ip_range_str = "10.18.46-10.18.46.69";
        let ip_range = ip_range_str.parse::<IPRange>();
        assert!(ip_range.is_err());
        assert_eq!(
            format!("{}", ip_range.unwrap_err()),
            "Failed to parse IPv4 address: Fail to parse ipv4 address: Invalid IP format (expected IPv4) in 10.18.46"
        );
    }

    #[test]
    fn test_invalid_end_ip_format() {
        let ip_range_str = "10.18.46.62-10.18.46";
        let ip_range = ip_range_str.parse::<IPRange>();
        assert!(ip_range.is_err());
        assert_eq!(
            format!("{}", ip_range.unwrap_err()),
            "Failed to parse IPv4 address: Fail to parse ipv4 address: Invalid IP format (expected IPv4) in 10.18.46"
        );
    }

    #[test]
    fn test_invalid_ip_part() {
        let ip_range_str = "10.18.46.abc-10.18.46.69";
        let ip_range = ip_range_str.parse::<IPRange>();
        assert!(ip_range.is_err());
        assert!(matches!(ip_range.unwrap_err(), IPRangeError::IPv4Error(_)));
    }

    #[test]
    fn test_edge_case_ip_range() {
        let ip_range_str = "10.18.46.69-10.18.46.69";
        let ip_range = ip_range_str.parse::<IPRange>();
        assert!(ip_range.is_ok());
        let ip_range = ip_range.unwrap();
        assert_eq!(ip_range.name, ip_range_str);
        assert_eq!(ip_range.start, ip_range.end);
    }

    #[test]
    fn test_large_ip_range() {
        let ip_range_str = "10.0.0.0-10.0.0.255";
        let ip_range = ip_range_str.parse::<IPRange>();
        assert!(ip_range.is_ok());
        let ip_range = ip_range.unwrap();
        assert_eq!(ip_range.name, ip_range_str);
        assert_eq!(ip_range.start, IPv4(10 << 24));
        assert_eq!(ip_range.end, IPv4((10 << 24) | 255));
    }

    #[test]
    fn test_capacity() {
        let ip_range_str = "10.0.0.0-10.0.0.255";
        let ip_range = ip_range_str.parse::<IPRange>().unwrap();
        assert_eq!(ip_range.capacity(), 1);
    }

    #[test]
    fn test_single_ip_capacity() {
        let ip_range_str = "10.0.0.1-10.0.0.1";
        let ip_range = ip_range_str.parse::<IPRange>().unwrap();
        assert_eq!(ip_range.capacity(), 1);
    }

    #[test]
    fn test_reverse_ip_range() {
        let ip_range_str = "10.0.0.255-10.0.0.0";
        let ip_range = ip_range_str.parse::<IPRange>();
        assert!(ip_range.is_err());
        assert_eq!(
            format!("{}", ip_range.unwrap_err()),
            "Fail to parse ip range: Start IP must be less than or equal to end IP in 10.0.0.255-10.0.0.0."
        );
    }

    #[test]
    fn test_non_contiguous_ip_range() {
        let ip_range_str = "10.0.0.1-10.0.0.3";
        let ip_range = ip_range_str.parse::<IPRange>().unwrap();
        assert_eq!(ip_range.capacity(), 2);
    }

    #[test]
    fn test_ip_range_with_large_gap() {
        let ip_range_str = "10.0.0.1-10.255.255.255";
        let ip_range = ip_range_str.parse::<IPRange>();
        assert!(ip_range.is_ok());
        let ip_range = ip_range.unwrap();
        assert_eq!(ip_range.name, ip_range_str);
        assert_eq!(ip_range.capacity(), 24);
        assert_eq!(ip_range.start, IPv4((10 << 24) + 1));
    }

    #[test]
    fn test_ip_range_with_minimum_values() {
        let ip_range_str = "0.0.0.0-0.0.0.0";
        let ip_range = ip_range_str.parse::<IPRange>();
        assert!(ip_range.is_ok());
        let ip_range = ip_range.unwrap();
        assert_eq!(ip_range.name, ip_range_str);
        assert_eq!(ip_range.start, IPv4(0));
        assert_eq!(ip_range.end, IPv4(0));
        assert_eq!(ip_range.capacity(), 1);
    }

    #[test]
    fn test_ip_range_with_maximum_values() {
        let ip_range_str = "255.255.255.255-255.255.255.255";
        let ip_range = ip_range_str.parse::<IPRange>();
        assert!(ip_range.is_ok());
        let ip_range = ip_range.unwrap();
        assert_eq!(ip_range.name, ip_range_str);
        assert_eq!(ip_range.start, IPv4(u32::MAX as u64));
        assert_eq!(ip_range.end, IPv4(u32::MAX as u64));
        assert_eq!(ip_range.capacity(), 1);
    }

    #[test]
    fn test_ip_range_with_overlapping_ips() {
        let ip_range_str = "192.168.1.1-192.168.1.255";
        let ip_range = ip_range_str.parse::<IPRange>();
        assert!(ip_range.is_ok());
        let ip_range = ip_range.unwrap();
        assert_eq!(ip_range.name, ip_range_str);
        assert_eq!(ip_range.capacity(), 8);
    }

    #[test]
    fn test_ip_range_with_invalid_characters() {
        let ip_range_str = "192.168.1.a-192.168.1.10";
        let ip_range = ip_range_str.parse::<IPRange>();
        assert!(ip_range.is_err());
        assert!(matches!(ip_range.unwrap_err(), IPRangeError::IPv4Error(_)));
    }

    #[test]
    fn test_ip_range_with_whitespace() {
        let ip_range_str = " 192.168.1.1 - 192.168.1.10 ";
        let ip_range = ip_range_str.trim().parse::<IPRange>();
        assert!(ip_range.is_ok());
        let ip_range = ip_range.unwrap();
        assert_eq!(ip_range.name, ip_range_str.trim());
        assert_eq!(ip_range.capacity(), 5);
    }

    #[test]
    fn test_ip_range_with_single_octet() {
        let ip_range_str = "192-192.168.1.10";
        let ip_range = ip_range_str.parse::<IPRange>();
        assert!(ip_range.is_err());
        assert_eq!(
            format!("{}", ip_range.unwrap_err()),
            "Failed to parse IPv4 address: Fail to parse ipv4 address: Invalid IP format (expected IPv4) in 192"
        );
    }

    #[test]
    fn test_ip_range_with_partial_octets() {
        let ip_range_str = "192.168-192.168.1.10";
        let ip_range = ip_range_str.parse::<IPRange>();
        assert!(ip_range.is_err());
        assert_eq!(
            format!("{}", ip_range.unwrap_err()),
            "Failed to parse IPv4 address: Fail to parse ipv4 address: Invalid IP format (expected IPv4) in 192.168"
        );
    }

    #[test]
    fn test_split_ip_range_into_prefixes_1() {
        let start = ("192.168.10.1").parse::<IPv4>().unwrap();
        let end = ("192.168.10.10").parse::<IPv4>().unwrap();
        let ip_range = split_ip_range_into_prefixes(&start, &end);
        assert_eq!(ip_range.len(), 5);
    }
}
