use std::str::FromStr;

mod prefix;
use prefix::Prefix;

pub mod ip_range;
use ip_range::IPRange;

pub mod ipv4;
use ipv4::IPv4;

pub mod hostname;
use hostname::Hostname;

#[derive(Debug, Clone)]
pub enum PrefixListItem {
    Prefix(Prefix),
    IPRange(IPRange),
    Hostname(Hostname),
}

#[derive(thiserror::Error, Debug)]
pub enum PrefixListItemError {
    // #[error("Failed to parse prefix list item: {0}")]
    // General(String),
    #[error("Transit error in prefix list item from IPRange: {0}")]
    IPRangeError(#[from] ip_range::IPRangeError),

    #[error("Transit error in prefix list item from Prefix: {0}")]
    PrefixError(#[from] prefix::PrefixError),

    #[error("Transit error in prefix list item from Hostname: {0}")]
    HostnameError(#[from] hostname::HostnameError),

    #[error("Unknown type of prefix list item: {0}")]
    UnknownType(String),

    #[error("Empty line")]
    EmptyLine,
}

impl FromStr for PrefixListItem {
    type Err = PrefixListItemError;

    // Example line:
    // 10.0.0.0/8
    // or
    // 10.11.12.13-10.11.12.18
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        if is_ip_range(line) {
            let ip_range = line.parse::<IPRange>()?;
            Ok(PrefixListItem::IPRange(ip_range))
        } else if is_ip_prefix(line) {
            let prefix = line.parse::<Prefix>()?;
            Ok(PrefixListItem::Prefix(prefix))
        } else if is_hostname(line) {
            let hostname = line.parse::<Hostname>()?;
            Ok(PrefixListItem::Hostname(hostname))
        } else if line.trim().is_empty() {
            Err(PrefixListItemError::EmptyLine)
        } else {
            Err(PrefixListItemError::UnknownType(line.to_string()))
        }
    }
}

impl PrefixListItem {
    pub fn capacity(&self) -> u64 {
        match self {
            PrefixListItem::Prefix(prefix) => prefix.capacity(),
            PrefixListItem::IPRange(ip_range) => ip_range.capacity(),
            PrefixListItem::Hostname(hostname) => hostname.capacity(),
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            PrefixListItem::Prefix(prefix) => prefix.get_name(),
            PrefixListItem::IPRange(ip_range) => ip_range.get_name(),
            PrefixListItem::Hostname(hostname) => hostname.get_name(),
        }
    }

    pub fn start_ip(&self) -> &IPv4 {
        match self {
            PrefixListItem::Prefix(prefix) => prefix.start_ip(),
            PrefixListItem::IPRange(ip_range) => ip_range.start_ip(),
            PrefixListItem::Hostname(hostname) => hostname.start_ip(),
        }
    }

    pub fn end_ip(&self) -> &IPv4 {
        match self {
            PrefixListItem::Prefix(prefix) => prefix.end_ip(),
            PrefixListItem::IPRange(ip_range) => ip_range.end_ip(),
            PrefixListItem::Hostname(hostname) => hostname.end_ip(),
        }
    }
}

fn is_ip_range(line: impl AsRef<str>) -> bool {
    let line = line.as_ref();

    line.chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == '-')
        && line.matches('-').count() == 1
        && line.matches('.').count() == 6
}

fn is_ip_prefix(line: impl AsRef<str>) -> bool {
    let line = line.as_ref();

    let condition1 = line
        .chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == '/')
        && line.matches('.').count() == 3;

    if line.contains('/') {
        // number of characters after the '/' must be 1-2
        let condition2 = line
            .split('/')
            .nth(1)
            .map(|s| s.len() <= 2 && s.parse::<u8>().is_ok())
            .unwrap_or(false);

        return condition1 && condition2;
    }

    condition1
}

fn is_hostname(line: impl AsRef<str>) -> bool {
    let line = line.as_ref();

    if line.is_empty() {
        return false;
    }

    line.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_list_item_from_str_prefix() {
        let input = "10.0.0.0/8";
        let result = PrefixListItem::from_str(input);
        assert!(result.is_ok());
        if let PrefixListItem::Prefix(prefix) = result.unwrap() {
            assert_eq!(prefix.get_name(), input);
        } else {
            panic!("Expected Prefix variant");
        }
    }

    #[test]
    fn test_prefix_list_item_from_str_ip_range() {
        let input = "10.11.12.13-10.11.12.18";
        let result = PrefixListItem::from_str(input);
        assert!(result.is_ok());
        if let PrefixListItem::IPRange(ip_range) = result.unwrap() {
            assert_eq!(ip_range.get_name(), input);
        } else {
            panic!("Expected IPRange variant");
        }
    }

    #[test]
    fn test_prefix_list_item_from_str_invalid() {
        let input = "invalid";
        let result = PrefixListItem::from_str(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_prefix_list_item_capacity_prefix() {
        let input = "10.0.0.0/8";
        let prefix_list_item = PrefixListItem::from_str(input).unwrap();
        assert_eq!(prefix_list_item.capacity(), 1); // 2^(32-8)
    }

    #[test]
    fn test_prefix_list_item_capacity_ip_range() {
        let input = "10.11.12.13-10.11.12.18";
        let prefix_list_item = PrefixListItem::from_str(input).unwrap();
        assert_eq!(prefix_list_item.capacity(), 4); // 10.11.12.13 to 10.11.12.18 inclusive
    }

    #[test]
    fn test_is_ip_range() {
        assert!(is_ip_range("10.11.12.13-10.11.12.14"));
        assert!(!is_ip_range("10.11.12.13 - 10.11.12.14"));
        assert!(!is_ip_range("10.11.12.13-10.11.12"));
        assert!(!is_ip_range("10.11.12.13"));
        assert!(!is_ip_range("10.11.12.13 "));
        assert!(!is_ip_range("10.11.12.13/24"));
        assert!(!is_ip_range("a10.11.12.13-10.11.12.14"));
        assert!(!is_ip_range(""));
    }

    #[test]
    fn test_is_ip_prefix() {
        assert!(is_ip_prefix("10.11.12.13"));
        assert!(is_ip_prefix("10.11.12.13/1"));
        assert!(is_ip_prefix("10.11.12.13/32"));
        assert!(!is_ip_prefix("10.11.12.13 - 10.11.12.14"));
        assert!(!is_ip_prefix("10.11.12.13-10.11.12"));
        assert!(!is_ip_prefix("10.11.12.13/"));
        assert!(!is_ip_prefix("10.11.12.13 "));
        assert!(!is_ip_prefix(" 10.11.12.13 "));
        assert!(!is_ip_prefix("10.11.12.13/234"));
        assert!(!is_ip_prefix("a10.11.12.13-10.11.12.14"));
        assert!(!is_ip_prefix(""));
    }

    #[test]
    fn test_is_hostname() {
        assert!(is_hostname("hostname"));
        assert!(is_hostname("10.11.12.13"));
        assert!(is_hostname("host-name"));
        assert!(is_hostname("host.name"));
        assert!(is_hostname("host123.name.com"));
        assert!(!is_hostname("host@name"));
        assert!(!is_hostname("host name"));
        assert!(!is_hostname("host_name"));
        assert!(!is_hostname("host:name"));
        assert!(!is_hostname("host name.com"));
        assert!(!is_hostname("host name.com "));
        assert!(!is_hostname(" host name.com"));
        assert!(!is_hostname("host name.com@"));
        assert!(!is_hostname("host name.com#"));
        assert!(!is_hostname("host name.com$"));
        assert!(!is_hostname("host name.com%"));
        assert!(!is_hostname(""));
    }
}
