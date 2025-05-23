use std::str::FromStr;

use super::ipv4::{IPv4, IPv4Error};

#[derive(Debug, Clone)]
pub struct Prefix {
    name: String,
    start: IPv4,
    end: IPv4,
}

pub struct Builder {
    name: String,
    start: IPv4,
    mask_length: u8,
}

#[derive(thiserror::Error, Debug)]
pub enum PrefixError {
    #[error("Fail to parse prefix: {0}")]
    General(String),
    #[error("Failed to parse prefix: {0}")]
    ParseError(#[from] IPv4Error),
    #[error("Failed to parse integer: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
}

impl FromStr for Prefix {
    type Err = PrefixError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let name = String::from(s);
        let parts: Vec<_> = s.split("/").collect();
        match parts.len() {
            2 => {
                let start = parts[0].parse::<IPv4>()?;
                let mask_length: u8 = parts[1].parse()?;
                if !(0..=32).contains(&mask_length) {
                    return Err(PrefixError::General(
                        format!(
                            "Invalid prefix mask length (expected from 1 to 32) in {}.",
                            &name
                        )
                        .to_string(),
                    ));
                }
                let end = start.get_broadcast(mask_length);
                Ok(Prefix { name, start, end })
            }
            1 => {
                let start = parts[0].parse::<IPv4>()?;
                let mask_length = 32;
                let end = start.get_broadcast(mask_length);
                Ok(Prefix { name, start, end })
            }
            _ => Err(PrefixError::General(
                format!(
                    "Invalid prefix format (expected IPv4 or Prefix/len) in {}.",
                    &name
                )
                .to_string(),
            )),
        }
    }
}

impl Prefix {
    pub fn capacity(&self) -> u64 {
        1
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

impl Builder {
    pub fn new(name: String, start: IPv4, mask_length: u8) -> Self {
        Self {
            name,
            start,
            mask_length,
        }
    }

    pub fn build(self) -> Prefix {
        let end = self.start.get_broadcast(self.mask_length);
        Prefix {
            name: self.name,
            start: self.start,
            end,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_prefix1() {
        let prefix_str = "192.168.0.0/24";
        let prefix = prefix_str.parse::<Prefix>();
        assert!(prefix.is_ok());
        let prefix = prefix.unwrap();
        assert_eq!(prefix.name, "192.168.0.0/24");
        assert_eq!(prefix.start.0, 0xC0A80000);
        assert_eq!(prefix.end.0, 0xC0A800FF);
    }

    #[test]
    fn test_valid_prefix2() {
        let prefix_str = "192.168.0.0";
        let prefix = prefix_str.parse::<Prefix>();
        assert!(prefix.is_ok());
        let prefix = prefix.unwrap();
        assert_eq!(prefix.name, "192.168.0.0");
        assert_eq!(prefix.start.0, 0xC0A80000);
        assert_eq!(prefix.end.0, 0xC0A80000);
    }

    #[test]
    fn test_invalid_prefix_format() {
        let prefix_str = "192.168.0.0-24";
        let prefix = prefix_str.parse::<Prefix>();
        assert!(prefix.is_err());
        assert_eq!(
            format!("{}", prefix.unwrap_err()),
            "Failed to parse prefix: Failed to parse IPv4 address: invalid digit found in string"
        );
    }

    #[test]
    fn test_invalid_ipv4_format() {
        let prefix_str = "192.168.0/24";
        let prefix = prefix_str.parse::<Prefix>();
        assert!(prefix.is_err());
        assert_eq!(
            format!("{}", prefix.unwrap_err()),
            "Failed to parse prefix: Fail to parse ipv4 address: Invalid IP format (expected IPv4) in 192.168.0"
        );
    }

    #[test]
    fn test_invalid_integer() {
        let prefix_str = "192.168.0.a/24";
        let prefix = prefix_str.parse::<Prefix>();
        assert!(prefix.is_err());
        assert!(matches!(prefix.unwrap_err(), PrefixError::ParseError(_)));
    }

    #[test]
    fn test_invalid_subnet_33() {
        let prefix_str = "192.168.0.0/33";
        let prefix = prefix_str.parse::<Prefix>();
        assert!(prefix.is_err());
        assert_eq!(
            format!("{}", prefix.unwrap_err()),
            "Fail to parse prefix: Invalid prefix mask length (expected from 1 to 32) in 192.168.0.0/33."
        );
    }

    #[test]
    fn test_invalid_subnet_0() {
        let prefix_str = "192.168.0.0/0";
        let prefix = prefix_str.parse::<Prefix>().unwrap();
        assert_eq!(prefix.end.0, 0xFFFFFFFF);
    }

    #[test]
    fn test_prefix_length_missing() {
        let prefix_str = "192.168.0.0/";
        let prefix = prefix_str.parse::<Prefix>();
        assert!(prefix.is_err());
        assert_eq!(
            format!("{}", prefix.unwrap_err()),
            "Failed to parse integer: cannot parse integer from empty string"
        );
    }

    #[test]
    fn test_prefix_with_valid_length() {
        let prefix_str = "10.0.0.0/16";
        let prefix = prefix_str.parse::<Prefix>().unwrap();
        assert_eq!(prefix.start.0, 0x0A000000);
        assert_eq!(prefix.end.0, 0x0A00FFFF);
    }

    #[test]
    fn test_prefix_with_boundary_length() {
        let prefix_str = "10.0.0.0/1";
        let prefix = prefix_str.parse::<Prefix>().unwrap();
        assert_eq!(prefix.end.0, 0x7FFFFFFF);

        let prefix_str = "10.0.0.0/32";
        let prefix = prefix_str.parse::<Prefix>().unwrap();
        assert_eq!(prefix.end.0, 0x0A000000);
    }

    #[test]
    fn test_prefix_default() {
        let prefix_str = "0.0.0.0/0";
        let prefix = prefix_str.parse::<Prefix>().unwrap();
        assert_eq!(prefix.end.0, 0xFFFFFFFF);
    }
}
