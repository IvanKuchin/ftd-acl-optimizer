use std::str::FromStr;

pub mod prefix_list_item;
use prefix_list_item::PrefixListItem;

#[derive(Debug, Clone)]
pub struct PrefixList {
    _name: String,
    items: Vec<PrefixListItem>,
}

#[derive(thiserror::Error, Debug)]
pub enum PrefixListError {
    #[error("Fail to parse prefix list: {0}")]
    General(String),
    #[error("Fail to parse prefix list {0}")]
    GeneralNoColon(String),
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

            let prefix_str = line
                .split("(")
                .nth(1)
                .ok_or(PrefixListError::General(format!(
                    "Invalid prefix list format ({}), open parenthesis doesn't split prefix in two pieces.",
                    line
                )))?
                .split(")")
                .next()
                .ok_or(PrefixListError::General(format!(
                    "Invalid prefix list format ({}), close parenthesis doesn't split prefix in two pieces.",
                    line
                )))?
                .trim()
                .to_string();

            let items = prefix_str
                .split(",")
                .map(|s| {
                    s.trim()
                        .parse::<PrefixListItem>()
                        .map_err(|e| PrefixListError::GeneralNoColon(format!("({}) :{}", line, e)))
                })
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Self { _name: name, items })
        } else if !line.contains("(") && !line.contains(")") {
            let name = line.to_string();
            let items = vec![line
                .trim()
                .parse::<PrefixListItem>()
                .map_err(|e| PrefixListError::General(e.to_string()))?];

            if items.is_empty() {
                return Err(PrefixListError::General("Empty prefix list.".to_string()));
            }

            Ok(Self { _name: name, items })
        } else {
            return Err(PrefixListError::General(format!(
                "Invalid prefix list format {}",
                line
            )));
        }
    }
}

impl PrefixList {
    pub fn get_items(&self) -> &Vec<PrefixListItem> {
        &self.items
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
        assert_eq!(prefix_list._name, "RFC1918");
    }

    #[test]
    fn test_valid_prefix_list2() {
        let line = "10.0.0.0/8";
        let prefix_list = PrefixList::from_str(line);
        assert!(prefix_list.is_ok());
        let prefix_list = prefix_list.unwrap();
        assert!(prefix_list.items.len() == 1);
        assert_eq!(prefix_list._name, "10.0.0.0/8");
    }

    #[test]
    fn test_valid_prefix_list3() {
        let line = "10.11.12.13";
        let prefix_list = PrefixList::from_str(line);
        assert!(prefix_list.is_ok());
        let prefix_list = prefix_list.unwrap();
        assert!(prefix_list.items.len() == 1);
        assert_eq!(prefix_list._name, "10.11.12.13");
    }

    #[test]
    fn test_valid_prefix_list4() {
        let line =
            "RFC1918 (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, 192.168.168.168-192.168.168.169, ipv4.net)";
        let prefix_list = PrefixList::from_str(line);
        assert!(prefix_list.is_ok());
        let prefix_list = prefix_list.unwrap();
        assert!(prefix_list.items.len() == 5);
        assert_eq!(prefix_list._name, "RFC1918");
    }

    #[test]
    fn test_valid_prefix_list5() {
        let line =
            "RFC1918 (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, 192.168.168.168-192.168.168.169, ipv4.net, connme.ru)";
        let prefix_list = PrefixList::from_str(line);
        assert!(prefix_list.is_ok());
        let prefix_list = prefix_list.unwrap();
        assert!(prefix_list.items.len() == 6);
        assert_eq!(prefix_list._name, "RFC1918");
    }

    #[test]
    fn test_invalid_prefix() {
        let line = "Invalid (10.0.0.0/8, invalid_prefix)";
        let result = PrefixList::from_str(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_prefix_list_format_duplicate() {
        let line = "RFC1918 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16";
        let result = PrefixList::from_str(line);
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()), 
            "Fail to parse prefix list: Unknown type of prefix list item: RFC1918 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16"
        );
    }

    #[test]
    fn test_invalid_prefix_list_open_parenthesis() {
        let line = "RFC1918 (";
        let result = PrefixList::from_str(line);
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "Fail to parse prefix list: Invalid prefix list format RFC1918 ("
        );
    }

    #[test]
    fn test_invalid_prefix_list_close_parenthesis() {
        let line = "RFC1918 (  )10.0.0.0/32";
        let result = PrefixList::from_str(line);
        assert!(result.is_err());
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
        assert_eq!(prefix_list._name, "RFC1918");
        assert_eq!(prefix_list.items.len(), 3);
    }

    #[test]
    fn test_prefix_list_with_extra_comma() {
        let line = "RFC1918 (10.0.0.0/8,, 172.16.0.0/12, 192.168.0.0/16)";
        let result = PrefixList::from_str(line);
        dbg!(&result);
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
