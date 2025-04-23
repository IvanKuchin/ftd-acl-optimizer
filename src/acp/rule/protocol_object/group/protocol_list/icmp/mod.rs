use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use super::common;

#[derive(Debug, Clone)]
pub struct Icmp {
    name: String,
    protocol: u8,
    icmp_type: Option<u8>,
    code: Option<u8>,
}

#[derive(thiserror::Error, Debug)]
pub enum IcmpError {
    #[error("Failed to parse ICMP: {0}")]
    General(String),
    #[error("Failed to parse ICMP: {0}")]
    CommonError(#[from] common::CommonError),
}

impl fmt::Display for Icmp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(code) = self.code {
            write!(
                f,
                "{} (protocol {}, type {}, code {})",
                self.name,
                self.protocol,
                self.icmp_type
                    .expect("PANIC: ICMP type is None while ICMP code is dedined"),
                code
            )
        } else if let Some(icmp_type) = self.icmp_type {
            write!(
                f,
                "{} (protocol {}, type {})",
                self.name, self.protocol, icmp_type
            )
        } else {
            write!(f, "{} (protocol {})", self.name, self.protocol)
        }
    }
}

impl FromStr for Icmp {
    type Err = IcmpError;

    // Example 1
    // protocol 1, type 3, code 4

    // Example 2
    // ICMP (protocol 1)

    // Example 3
    // ICMP (protocol 1, type 3, code 4)

    // Example 4
    // ICMP (protocol 1, type 200)

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, proto_and_ports) = common::parse_name_and_protocol(s)?;

        let protocol = common::parse_protocol(proto_and_ports)?;

        let (icmp_type, code) = parse_type_and_code(proto_and_ports)?;

        Ok(Self {
            name: name.to_string(),
            protocol,
            icmp_type,
            code,
        })
    }
}

impl Icmp {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn is_l4(&self) -> bool {
        false
    }

    pub fn get_protocol(&self) -> u8 {
        self.protocol
    }
}

impl PartialEq for Icmp {
    fn eq(&self, other: &Self) -> bool {
        self.protocol == other.protocol
            && self.icmp_type == other.icmp_type
            && self.code == other.code
    }
}

impl Eq for Icmp {}

impl Hash for Icmp {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.protocol.hash(state);
        self.icmp_type.hash(state);
        self.code.hash(state);
    }
}

// Example 1
// protocol 1, type 3, code 4

// Example 2
// protocol 1, type 3

// Example 3
// protocol 1
fn parse_type_and_code(s: &str) -> Result<(Option<u8>, Option<u8>), IcmpError> {
    let mut parts = s.split(",");

    match parts.clone().count() {
        1 => Ok((None, None)),
        2 => {
            let icmp_type = parts.nth(1).unwrap().trim();
            let icmp_type = icmp_type.split_whitespace().last().unwrap();
            let icmp_type = icmp_type.parse().map_err(|_| {
                IcmpError::General(format!(
                    "Failed to parse ICMP type: {} from {}",
                    icmp_type, s
                ))
            })?;

            Ok((Some(icmp_type), None))
        }
        3 => {
            let icmp_type = parts.nth(1).unwrap().trim();
            let icmp_type = icmp_type.split_whitespace().last().unwrap();
            let icmp_type = icmp_type.parse().map_err(|_| {
                IcmpError::General(format!(
                    "Failed to parse ICMP type: {} from {}",
                    icmp_type, s
                ))
            })?;

            let code = parts.next().unwrap().trim();
            let code = code.split_whitespace().last().unwrap();

            if code.to_lowercase() == "any" {
                return Ok((Some(icmp_type), None));
            }

            let code = code.parse().map_err(|_| {
                IcmpError::General(format!("Failed to parse ICMP code: {} from  {}", code, s))
            })?;

            Ok((Some(icmp_type), Some(code)))
        }
        _ => Err(IcmpError::General(format!("Invalid ICMP: {}", s))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_icmpv6() {
        let icmp = "ICMP-Name (protocol 58, type 3, code 4)"
            .parse::<Icmp>()
            .unwrap();

        assert_eq!(icmp.name, "ICMP-Name");
        assert_eq!(icmp.protocol, 58);
        assert_eq!(icmp.icmp_type, Some(3));
        assert_eq!(icmp.code, Some(4));
    }

    #[test]
    fn test_parse_full_icmp_1() {
        let icmp = "ICMP-Name (protocol 1, type 3, code 4)"
            .parse::<Icmp>()
            .unwrap();

        assert_eq!(icmp.name, "ICMP-Name");
        assert_eq!(icmp.protocol, 1);
        assert_eq!(icmp.icmp_type, Some(3));
        assert_eq!(icmp.code, Some(4));
    }

    #[test]
    fn test_parse_full_icmp_2() {
        let icmp = "ICMP-Name (protocol 1, type 3, code any)"
            .parse::<Icmp>()
            .unwrap();

        assert_eq!(icmp.name, "ICMP-Name");
        assert_eq!(icmp.protocol, 1);
        assert_eq!(icmp.icmp_type, Some(3));
        assert!(icmp.code.is_none());
    }

    #[test]
    fn test_parse_full_icmp_3() {
        let icmp = "ICMP-Name (protocol 1, type 3, code Any)"
            .parse::<Icmp>()
            .unwrap();

        assert_eq!(icmp.name, "ICMP-Name");
        assert_eq!(icmp.protocol, 1);
        assert_eq!(icmp.icmp_type, Some(3));
        assert!(icmp.code.is_none());
    }

    #[test]
    fn test_parse_full_icmp_4() {
        let icmp = "ICMP-Name (protocol 1, type 3, code ANY)"
            .parse::<Icmp>()
            .unwrap();

        assert_eq!(icmp.name, "ICMP-Name");
        assert_eq!(icmp.protocol, 1);
        assert_eq!(icmp.icmp_type, Some(3));
        assert!(icmp.code.is_none());
    }

    #[test]
    fn test_parse_full_icmp_5() {
        let icmp = "ICMP-Name (protocol 1, type 3, code aNY)"
            .parse::<Icmp>()
            .unwrap();

        assert_eq!(icmp.name, "ICMP-Name");
        assert_eq!(icmp.protocol, 1);
        assert_eq!(icmp.icmp_type, Some(3));
        assert!(icmp.code.is_none());
    }

    #[test]
    fn test_parse_icmp_with_type() {
        let icmp = "ICMP-Type (protocol 1, type 8)".parse::<Icmp>().unwrap();
        assert_eq!(icmp.name, "ICMP-Type");
        assert_eq!(icmp.protocol, 1);
        assert_eq!(icmp.icmp_type, Some(8));
        assert_eq!(icmp.code, None);
    }

    #[test]
    fn test_parse_basic_icmp() {
        let icmp = "Basic-ICMP (protocol 1)".parse::<Icmp>().unwrap();
        assert_eq!(icmp.name, "Basic-ICMP");
        assert_eq!(icmp.protocol, 1);
        assert_eq!(icmp.icmp_type, None);
        assert_eq!(icmp.code, None);
    }

    #[test]
    fn test_parse_icmpv6_with_type() {
        let icmp = "ICMPv6-Type (protocol 58, type 8)".parse::<Icmp>().unwrap();
        assert_eq!(icmp.name, "ICMPv6-Type");
        assert_eq!(icmp.protocol, 58);
        assert_eq!(icmp.icmp_type, Some(8));
        assert_eq!(icmp.code, None);
    }

    #[test]
    fn test_parse_basic_icmpv6() {
        let icmp = "Basic-ICMPv6 (protocol 58)".parse::<Icmp>().unwrap();
        assert_eq!(icmp.name, "Basic-ICMPv6");
        assert_eq!(icmp.protocol, 58);
        assert_eq!(icmp.icmp_type, None);
        assert_eq!(icmp.code, None);
    }

    #[test]
    fn test_parse_invalid_format() {
        assert!("Invalid (protocol 1, type, code)".parse::<Icmp>().is_err());
        assert!("Invalid (protocol 1, type a, code 4)"
            .parse::<Icmp>()
            .is_err());
        assert!("Invalid (protocol 1, type 3, code b)"
            .parse::<Icmp>()
            .is_err());
        assert!("Invalid (protocol 1, type 3, code )"
            .parse::<Icmp>()
            .is_err());
    }
}
