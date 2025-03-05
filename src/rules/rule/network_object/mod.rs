use std::str::FromStr;

mod group;
use std::vec;

use group::prefix_list::PrefixList;
use group::Group;

use super::utilities;

#[derive(Debug)]
pub struct NetworkObject {
    name: String,
    items: Vec<NetworkObjectItem>,
}

#[derive(Debug)]
pub enum NetworkObjectItem {
    ObjectGroup(Group),
    PrefixList(PrefixList),
}

#[derive(thiserror::Error, Debug)]
pub enum NetworkObjectError {
    #[error("Fail to parse network object: {0}")]
    General(String),
    #[error("Fail to parse network object {0}: {1}")]
    General2(String, String),
    #[error("Fail to parse network object: {0}")]
    GroupError(#[from] group::GroupError),
    #[error("Fail to parse network object: {0}")]
    PrefixListError(#[from] group::prefix_list::PrefixListError),
    #[error("PANIC in parse network object: {0}")]
    Panic(String),
    #[error("Fail to parse network object: {0}")]
    NameExtractionError(#[from] utilities::UtilitiesError),
}

impl TryFrom<&Vec<String>> for NetworkObject {
    type Error = NetworkObjectError;

    // Example input:
    // Source Networks       : Internal (group)
    //                           OBJ-10.11.12.0_23 (10.11.12.0/23)
    //                           10.0.0.0/8
    //                           204.99.0.0/16
    //                           172.16.0.0/12
    //                         OBJ-192.168.243.0_24 (192.168.243.0/24)
    //                         OBJ-10.18.46.62-69 (10.18.46.62-10.18.46.69)

    fn try_from(lines: &Vec<String>) -> Result<Self, Self::Error> {
        if lines.is_empty() {
            return Err(NetworkObjectError::General(
                "Input lines are empty".to_string(),
            ));
        }

        let (name, merged_lines) = utilities::extract_name(lines)?;

        let mut items = vec![];
        let mut idx = 0;
        while idx < merged_lines.len() {
            let (obj, obj_lines_count) = get_object(&merged_lines[idx..])?;
            items.push(obj);
            idx += obj_lines_count;
        }

        Ok(NetworkObject { name, items })
    }
}

fn get_object(
    lines: &[String],
) -> Result<(NetworkObjectItem, usize), <NetworkObject as TryFrom<&Vec<String>>>::Error> {
    if lines.is_empty() {
        return Err(NetworkObjectError::General(
            "Input lines are empty".to_string(),
        ));
    }

    let first_line = lines[0].as_str();
    if first_line.contains("(group)") {
        let lines_in_group = get_lines_in_group(lines)?;
        let group = Group::try_from(&lines[0..lines_in_group].to_vec())?;
        Ok((NetworkObjectItem::ObjectGroup(group), lines_in_group))
    } else {
        let prefix_list = PrefixList::from_str(first_line)?;
        Ok((NetworkObjectItem::PrefixList(prefix_list), 1))
    }
}

// Example1:
// Internal (group)
//   OBJ-157.121.0.0 (157.121.0.0/16)
//   OBJ-206.213.0.0 (206.213.0.0/16)
//   OBJ-167.69.0.0 (167.69.0.0/16)
//   OBJ-198.187.64.0_18 (198.187.64.0/18)
//   10.0.0.0/8
//   204.99.0.0/16
//   172.16.0.0/12
// OBJ-192.168.243.0_24 (192.168.243.0/24)
// OBJ-10.18.46.62-69 (10.18.46.62-10.18.46.69)
// return 8

// Example2:
// Internal (group)
// Another (group)
// return 1

fn get_lines_in_group(
    lines: &[String],
) -> Result<usize, <NetworkObject as TryFrom<&Vec<String>>>::Error> {
    if lines.is_empty() {
        return Err(NetworkObjectError::General(
            "Input lines are empty".to_string(),
        ));
    }
    if lines.len() == 1 {
        return Ok(1);
    }

    let [_, first_line, ..] = lines else {
        return Err(NetworkObjectError::Panic(format!("{:?}", lines)));
    };

    let reference_padding = first_line.len() - first_line.trim_start().len();
    let mut idx = 1;
    while idx < lines.len() {
        if lines[idx].contains("(group)") {
            return Ok(idx);
        }
        let padding = lines[idx].len() - lines[idx].trim_start().len();
        if padding != reference_padding {
            return Ok(idx);
        }
        idx += 1;
    }
    Ok(idx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_lines_in_group_single_group() {
        let lines = vec![
            "Internal (group)".to_string(),
            "  OBJ-157.121.0.0 (157.121.0.0/16)".to_string(),
            "  OBJ-206.213.0.0 (206.213.0.0/16)".to_string(),
            "  OBJ-167.69.0.0 (167.69.0.0/16)".to_string(),
            "  OBJ-198.187.64.0_18 (198.187.64.0/18)".to_string(),
            "  10.0.0.0/8".to_string(),
            "  204.99.0.0/16".to_string(),
            "  172.16.0.0/12".to_string(),
            "OBJ-192.168.243.0_24 (192.168.243.0/24)".to_string(),
            "OBJ-10.18.46.62-69 (10.18.46.62-10.18.46.69)".to_string(),
        ];
        let result = get_lines_in_group(&lines).unwrap();
        assert_eq!(result, 8);
    }

    #[test]
    fn test_get_lines_in_group_multiple_groups() {
        let lines = vec![
            "Internal (group)".to_string(),
            "Another (group)".to_string(),
        ];
        let result = get_lines_in_group(&lines).unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_get_lines_in_group_empty_lines() {
        let lines: Vec<String> = vec![];
        let result = get_lines_in_group(&lines);
        assert!(result.is_err());
        if let Err(NetworkObjectError::General(msg)) = result {
            assert_eq!(msg, "Input lines are empty");
        } else {
            panic!("Expected NetworkObjectError::General");
        }
    }

    #[test]
    fn test_get_object_group() {
        let lines = vec![
            "Internal (group)".to_string(),
            "    OBJ-157.121.0.0 (157.121.0.0/16)".to_string(),
            "  OBJ-157.121.0.0 (157.121.0.0/16)".to_string(),
        ];
        let (obj, count) = get_object(&lines).unwrap();
        match obj {
            NetworkObjectItem::ObjectGroup(_) => (),
            _ => panic!("Expected NetworkObjectItem::ObjectGroup"),
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_get_object_multiple_groups1() {
        let lines = vec![
            "Internal (group)".to_string(),
            "Another (group)".to_string(),
        ];
        let (obj, count) = get_object(&lines).unwrap();
        match obj {
            NetworkObjectItem::ObjectGroup(_) => (),
            _ => panic!("Expected NetworkObjectItem::ObjectGroup"),
        }
        assert_eq!(count, 1);
    }

    #[test]
    fn test_get_object_prefix_list() {
        let lines = vec!["10.0.0.0/8".to_string()];
        let (obj, count) = get_object(&lines).unwrap();
        match obj {
            NetworkObjectItem::PrefixList(_) => (),
            _ => panic!("Expected NetworkObjectItem::PrefixList"),
        }
        assert_eq!(count, 1);
    }

    #[test]
    fn test_get_object_empty_lines() {
        let lines: Vec<String> = vec![];
        let result = get_object(&lines);
        assert!(result.is_err());
        if let Err(NetworkObjectError::General(msg)) = result {
            assert_eq!(msg, "Input lines are empty");
        } else {
            panic!("Expected NetworkObjectError::General");
        }
    }

    #[test]
    fn test_try_from1() {
        let lines = vec![
            "    Source Networks       : Internal (group)".to_string(),
            "  OBJ-157.121.0.0 (157.121.0.0/16)".to_string(),
            "  OBJ-206.213.0.0 (206.213.0.0/16)".to_string(),
            "  OBJ-167.69.0.0 (167.69.0.0/16)".to_string(),
            "  OBJ-198.187.64.0_18 (198.187.64.0/18)".to_string(),
            "  10.0.0.0/8".to_string(),
            "  204.99.0.0/16".to_string(),
            "  172.16.0.0/12".to_string(),
            "OBJ-192.168.243.0_24 (192.168.243.0/24)".to_string(),
            "OBJ-10.18.46.62-69 (10.18.46.62-10.18.46.69)".to_string(),
        ];
        let result = NetworkObject::try_from(&lines).unwrap();
        assert_eq!(result.name, "Source Networks");
        assert_eq!(result.items.len(), 3);
    }

    #[test]
    fn test_try_from2() {
        let lines = vec![
            "    Source Networks       : Internal (group)".to_string(),
            " Another (group)".to_string(),
            "  OBJ-157.121.0.0 (157.121.0.0/16)".to_string(),
            "  OBJ-206.213.0.0 (206.213.0.0/16)".to_string(),
            "  OBJ-167.69.0.0 (167.69.0.0/16)".to_string(),
            "  OBJ-198.187.64.0_18 (198.187.64.0/18)".to_string(),
            "  10.0.0.0/8".to_string(),
            "  204.99.0.0/16".to_string(),
            "  172.16.0.0/12".to_string(),
            "OBJ-192.168.243.0_24 (192.168.243.0/24)".to_string(),
            "OBJ-10.18.46.62-69 (10.18.46.62-10.18.46.69)".to_string(),
        ];
        let result = NetworkObject::try_from(&lines).unwrap();
        assert_eq!(result.name, "Source Networks");
        assert_eq!(result.items.len(), 4);
    }

    #[test]
    fn test_try_from3() {
        let lines = vec![
            "    Source Networks       : Internal (group)".to_string(),
            " Another (group)".to_string(),
        ];
        let result = NetworkObject::try_from(&lines).unwrap();
        assert_eq!(result.name, "Source Networks");
        assert_eq!(result.items.len(), 2);
    }

    #[test]
    fn test_try_from4() {
        let lines = vec![
            "    Source Networks       : Internal (group)".to_string(),
            "  OBJ-157.121.0.0 (157.121.0.0/16)".to_string(),
            "  OBJ-206.213.0.0 (206.213.0.0/16)".to_string(),
            "  OBJ-167.69.0.0 (167.69.0.0/16)".to_string(),
            " Another (group)".to_string(),
            "  OBJ-198.187.64.0_18 (198.187.64.0/18)".to_string(),
            "  10.0.0.0/8".to_string(),
            "  204.99.0.0/16".to_string(),
            "  172.16.0.0/12".to_string(),
            "OBJ-192.168.243.0_24 (192.168.243.0/24)".to_string(),
            "OBJ-10.18.46.62-69 (10.18.46.62-10.18.46.69)".to_string(),
        ];
        let result = NetworkObject::try_from(&lines).unwrap();
        assert_eq!(result.name, "Source Networks");
        assert_eq!(result.items.len(), 4);
    }

    #[test]
    fn test_try_from5() {
        let lines = vec![
            "    Source Networks       : Internal (group)".to_string(),
            "  OBJ-157.121.0.0 (157.121.0.0/16)".to_string(),
            " Another (group)".to_string(),
            "  OBJ-206.213.0.0 (206.213.0.0/16)".to_string(),
            " Another (group)".to_string(),
            "  OBJ-167.69.0.0 (167.69.0.0/16)".to_string(),
            " Another (group)".to_string(),
            "  OBJ-198.187.64.0_18 (198.187.64.0/18)".to_string(),
            " Another (group)".to_string(),
            "  10.0.0.0/8".to_string(),
            "  204.99.0.0/16".to_string(),
            "  172.16.0.0/12".to_string(),
            "OBJ-192.168.243.0_24 (192.168.243.0/24)".to_string(),
            "OBJ-10.18.46.62-69 (10.18.46.62-10.18.46.69)".to_string(),
        ];
        let result = NetworkObject::try_from(&lines).unwrap();
        assert_eq!(result.name, "Source Networks");
        assert_eq!(result.items.len(), 7);
    }
}
