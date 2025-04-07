use std::str::FromStr;

mod prefix;
use prefix::Prefix;

mod ip_range;
use ip_range::IPRange;

mod ipv4;

#[derive(Debug, Clone)]
pub enum PrefixListItem {
    Prefix(Prefix),
    IPRange(IPRange),
}

#[derive(thiserror::Error, Debug)]
pub enum PrefixListItemError {
    // #[error("Failed to parse prefix list item: {0}")]
    // General(String),
    #[error("Failed to parse prefix list item: {0}")]
    IPRangeError(#[from] ip_range::IPRangeError),

    #[error("Failed to parse prefix list item: {0}")]
    PrefixError(#[from] prefix::PrefixError),
}

impl FromStr for PrefixListItem {
    type Err = PrefixListItemError;

    // Example line:
    // 10.0.0.0/8
    // or
    // 10.11.12.13-10.11.12.18
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        if line.contains("-") {
            let ip_range = line.parse::<IPRange>()?;
            Ok(PrefixListItem::IPRange(ip_range))
        } else {
            let prefix = line.parse::<Prefix>()?;
            Ok(PrefixListItem::Prefix(prefix))
        }
    }
}

impl PrefixListItem {
    pub fn capacity(&self) -> u64 {
        match self {
            PrefixListItem::Prefix(prefix) => prefix.capacity(),
            PrefixListItem::IPRange(ip_range) => ip_range.capacity(),
        }
    }
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
}
