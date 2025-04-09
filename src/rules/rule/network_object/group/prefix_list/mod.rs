use std::str::FromStr;

pub mod prefix_list_item;
use prefix_list_item::PrefixListItem;

#[derive(Debug, Clone)]
pub struct PrefixList {
    name: String,
    items: Vec<PrefixListItem>,
}

#[derive(thiserror::Error, Debug)]
pub enum PrefixListError {
    #[error("Fail to parse prefix list: {0}")]
    General(String),
    #[error("Fail to parse prefix list: {0}")]
    PrefixListItemError(#[from] prefix_list_item::PrefixListItemError),
}

impl FromStr for PrefixList {
    type Err = PrefixListError;

    // Example line1:
    // RFC1918 (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, 10.11.12.13-10.11.12.18)
    // Example line2:
    // 10.0.0.0/8
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        if line.contains("()") {
            return Err(PrefixListError::General("Empty prefix list.".to_string()));
        }

        if line.contains("(") && line.contains(")") {
            let name = line.split("(").collect::<Vec<&str>>()[0].trim().to_string();

            let prefix_str = line.split("(").collect::<Vec<&str>>()[1]
                .split(")")
                .collect::<Vec<&str>>()[0]
                .trim()
                .to_string();

            let items: Vec<_> = prefix_str
                .split(",")
                .map(|s| s.trim().parse::<PrefixListItem>())
                .collect::<Result<_, prefix_list_item::PrefixListItemError>>()?;

            Ok(Self { name, items })
        } else if !line.contains("(") && !line.contains(")") {
            let name = line.to_string();
            let items = vec![line
                .trim()
                .parse::<PrefixListItem>()
                .map_err(|e| PrefixListError::General(e.to_string()))?];

            if items.is_empty() {
                return Err(PrefixListError::General("Empty prefix list.".to_string()));
            }

            Ok(Self { name, items })
        } else {
            return Err(PrefixListError::General(format!(
                "Invalid prefix list format {}",
                line
            )));
        }
    }
}

impl PrefixList {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Returns the number of subnets in the list.
    /// This function does NOT perform optimizations (overlaps, shadowing, merging).
    /// For example: Test-prefix (192.168.0.0/24, 192.168.0.0/25) will return 2.
    pub fn capacity(&self) -> u64 {
        self.items.iter().map(|p| p.capacity()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_prefix_list1() {
        let line =
            "RFC1918 (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, 192.168.168.168-192.168.168.169)";
        let prefix_list = PrefixList::from_str(line);
        assert!(prefix_list.is_ok());
        let prefix_list = prefix_list.unwrap();
        assert!(prefix_list.items.len() == 4);
        assert_eq!(prefix_list.name, "RFC1918");
    }

    #[test]
    fn test_valid_prefix_list2() {
        let line = "10.0.0.0/8";
        let prefix_list = PrefixList::from_str(line);
        assert!(prefix_list.is_ok());
        let prefix_list = prefix_list.unwrap();
        assert!(prefix_list.items.len() == 1);
        assert_eq!(prefix_list.name, "10.0.0.0/8");
    }

    #[test]
    fn test_valid_prefix_list3() {
        let line = "10.11.12.13";
        let prefix_list = PrefixList::from_str(line);
        assert!(prefix_list.is_ok());
        let prefix_list = prefix_list.unwrap();
        assert!(prefix_list.items.len() == 1);
        assert_eq!(prefix_list.name, "10.11.12.13");
    }

    #[test]
    fn test_invalid_prefix() {
        let line = "Invalid (10.0.0.0/8, invalid_prefix)";
        let result = PrefixList::from_str(line);
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()), 
            "Fail to parse prefix list: Failed to parse prefix list item: Failed to parse prefix: Failed to parse IPv4 address: invalid digit found in string");
    }

    #[test]
    fn test_invalid_prefix_list_format_duplicate() {
        let line = "RFC1918 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16";
        let result = PrefixList::from_str(line);
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()), 
            "Fail to parse prefix list: Failed to parse prefix list item: Fail to parse prefix: Invalid prefix format (expected IPv4 or Prefix/len) in RFC1918 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16."
        );
    }

    #[test]
    fn test_invalid_empty_prefix_list() {
        let line = "Empty ()";
        let result = PrefixList::from_str(line);
        assert!(result.is_err());
        let prefix_list = result.unwrap_err();
        assert_eq!(
            format!("{}", prefix_list),
            "Fail to parse prefix list: Empty prefix list."
        );
    }

    #[test]
    fn test_invalid_prefix_list_with_whitespace() {
        let line = "   RFC1918   ( 10.0.0.0/8 , 172.16.0.0/12,  192.168.0.0/16  )  ";
        let prefix_list = PrefixList::from_str(line).unwrap();
        assert_eq!(prefix_list.name, "RFC1918");
        assert_eq!(prefix_list.items.len(), 3);
    }

    #[test]
    fn test_prefix_list_with_extra_comma() {
        let line = "RFC1918 (10.0.0.0/8,, 172.16.0.0/12, 192.168.0.0/16)";
        let result = PrefixList::from_str(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_parentheses() {
        let line = "RFC1918 (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16";
        let result = PrefixList::from_str(line);
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "Fail to parse prefix list: Invalid prefix list format RFC1918 (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16"
        );
    }

    #[test]
    fn test_capacity_single_prefix() {
        let line = "10.0.0.0/8";
        let prefix_list = PrefixList::from_str(line).unwrap();
        assert_eq!(prefix_list.capacity(), 1); // 2^24
    }

    #[test]
    fn test_capacity_multiple_prefixes() {
        let line = "RFC1918 (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16)";
        let prefix_list = PrefixList::from_str(line).unwrap();
        assert_eq!(prefix_list.capacity(), 1 + 1 + 1); // 2^24 + 2^20 + 2^16
    }

    #[test]
    fn test_capacity_with_ip_range_1() {
        let line = "Range (192.168.1.1-192.168.1.10)";
        let prefix_list = PrefixList::from_str(line).unwrap();
        assert_eq!(prefix_list.capacity(), 5);
    }

    #[test]
    fn test_capacity_with_ip_range_2() {
        let line = "Range (192.168.1.1-192.168.1.10, 192.168.1.1-192.168.1.10)";
        let prefix_list = PrefixList::from_str(line).unwrap();
        assert_eq!(prefix_list.capacity(), 10);
    }

    #[test]
    fn test_capacity_empty_prefix_list() {
        let line = "Empty ()";
        let result = PrefixList::from_str(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_capacity_mixed_prefixes_and_ranges() {
        let line = "Mixed (10.0.0.0/8, 192.168.1.1-192.168.1.10)";
        let prefix_list = PrefixList::from_str(line).unwrap();
        assert_eq!(prefix_list.capacity(), 1 + 5); // 1 + 10
    }
}
