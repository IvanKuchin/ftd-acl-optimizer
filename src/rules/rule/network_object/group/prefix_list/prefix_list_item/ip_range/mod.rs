use std::str::FromStr;

use super::ipv4::{IPv4, IPv4Error};

#[derive(Debug)]
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
            return Err(IPRangeError::General(format!("Invalid ip range format (expected ..ipv4.. - ..ipv4..) in {}.", &name).to_string()));
        }


        let start = parts[0].parse::<IPv4>()?;
        let end = parts[1].parse::<IPv4>()?;

        if start > end {
            return Err(IPRangeError::General(format!("Start IP must be less than or equal to end IP in {}.", &name).to_string()));
        }

        Ok(IPRange {
            name,
            start,
            end,
        })
    }
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
        assert_eq!(ip_range.start, IPv4((10 << 24) | (18 << 16) | (46 << 8) | 62));
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
}
