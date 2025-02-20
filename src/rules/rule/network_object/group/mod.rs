use std::str::FromStr;

pub mod prefix_list;
use prefix_list::PrefixList;

#[derive(Debug)]
pub struct Group {
    name: String,
    prefix_lists: Vec<PrefixList>,
}

#[derive(thiserror::Error, Debug)]
pub enum GroupError {
    #[error("Fail to parse group: {0}")]
    General(String),
    #[error("Fail to parse group {0}: {1}")]
    General2(String, String),
}


impl TryFrom<&Vec<String>> for Group {
    type Error = GroupError;

    // Example:
    // Internal (group)
    //                           OBJ-157.121.0.0 (157.121.0.0/16)
    //                           OBJ-206.213.0.0 (206.213.0.0/16)
    //                           OBJ-167.69.0.0 (167.69.0.0/16)
    //                           OBJ-198.187.64.0_18 (198.187.64.0/18)
    //                           10.0.0.0/8
    //                           204.99.0.0/16
    //                           172.16.0.0/12

    fn try_from(lines: &Vec<String>) -> Result<Self, Self::Error> {
        if let [title, ..] = lines.as_slice() {
            if !title.contains(" (group)") {
                return Err(GroupError::General(format!("Invalid group format {}", title)));
            }
            let name = title.split('(').next().unwrap().trim().to_string();
            let mut prefix_lists = vec![];
            

            for line in &lines[1..] {

                let prefix = line.trim().to_string();
                if !prefix.is_empty() {
                    prefix_lists.push(PrefixList::from_str(&prefix).map_err(|e| GroupError::General2(name.clone(), e.to_string()))?);
                }
            }
            
            Ok(Self {
                name,
                prefix_lists,
            })
        } else {
            Err(GroupError::General("Invalid group format.".to_string()))
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_group() {
        let lines = vec![
            "Internal (group)".to_string(),
            "      OBJ-157.121.0.0 (157.121.0.0/16)".to_string(),
            "      OBJ-206.213.0.0 (206.213.0.0/16)".to_string(),
            "      OBJ-167.69.0.0 (167.69.0.0/16)".to_string(),
            "      OBJ-198.187.64.0_18 (198.187.64.0/18)".to_string(),
            "      10.0.0.0/8".to_string(),
            "      204.99.0.0/16".to_string(),
            "      172.16.0.0/12".to_string(),
            "      172.16.17.18".to_string(),
        ];

        let group = Group::try_from(&lines).unwrap();
        assert_eq!(group.name, "Internal");
        assert_eq!(group.prefix_lists.len(), 8);
    }

    #[test]
    fn test_invalid_group_format() {
        let lines = vec!["__Invalid group format__".to_string()];
        let result = Group::try_from(&lines);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Fail to parse group: Invalid group format __Invalid group format__");
    }

    #[test]
    fn test_empty_lines() {
        let lines: Vec<String> = vec![];
        let result = Group::try_from(&lines);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Fail to parse group: Invalid group format.");
    }

    #[test]
    fn test_group_with_empty_prefixes() {
        let lines = vec![
            "Internal (group)".to_string(),
            "".to_string(),
            " ".to_string(),
        ];

        let group = Group::try_from(&lines).unwrap();
        assert_eq!(group.name, "Internal");
        assert_eq!(group.prefix_lists.len(), 0);
    }

    #[test]
    fn test_group_with_empty_group() {
        let lines = vec![
            "Internal (group)".to_string(),
        ];

        let group = Group::try_from(&lines).unwrap();
        assert_eq!(group.name, "Internal");
        assert_eq!(group.prefix_lists.len(), 0);
    }

    #[test]
    fn test_group_with_invalid_prefixes() {
        let lines = vec![
            "Internal (group)".to_string(),
            "INVALID_PREFIX".to_string(),
        ];
        
        let result = Group::try_from(&lines);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Fail to parse group Internal: Fail to parse prefix list: Failed to parse prefix list item: Failed to parse prefix: Failed to parse IPv4 address: invalid digit found in string");
    }
}
