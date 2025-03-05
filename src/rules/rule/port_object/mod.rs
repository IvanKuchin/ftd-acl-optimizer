use std::str::FromStr;

mod group;
use group::port_list;
use group::port_list::PortList;
use group::Group;

use super::network_object::utilities;

#[derive(Debug)]
pub struct PortObject {
    name: String,
    items: Vec<PortObjectItem>,
}

#[derive(Debug)]
pub enum PortObjectItem {
    PortList(PortList),
    Group(Group),
}

#[derive(thiserror::Error, Debug)]
pub enum PortObjectError {
    #[error("Failed to parse port object: {0}")]
    General(String),
    #[error("Failed to parse port object: {0}")]
    PortListError(#[from] port_list::PortListError),
    #[error("Failed to parse port object: {0}")]
    GroupError(#[from] group::GroupError),
    #[error("Fail to parse port object: {0}")]
    NameExtractionError(#[from] utilities::UtilitiesError),
}

impl TryFrom<&Vec<String>> for PortObject {
    type Error = PortObjectError;

    // Example input:
    //   Destination Ports     : HTTP-HTTPS_1 (group)
    //     HTTP (protocol 6, port 80)
    //     HTTPS (protocol 6, port 443)
    //   TCP-8080 (protocol 6, port 8080)
    //   protocol 6, port 33434
    fn try_from(lines: &Vec<String>) -> Result<Self, Self::Error> {
        if lines.is_empty() {
            return Err(PortObjectError::General(
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

        Ok(PortObject { name, items })
    }
}

fn get_object(lines: &[String]) -> Result<(PortObjectItem, usize), PortObjectError> {
    if lines.is_empty() {
        return Err(PortObjectError::General(
            "Input lines are empty".to_string(),
        ));
    }

    let first_line = lines[0].as_str();
    if first_line.contains("(group)") {
        let lines_in_group = utilities::calculate_lines_in_group(lines)?;
        let group = Group::try_from(&lines[0..lines_in_group].to_vec())?;
        Ok((PortObjectItem::Group(group), lines_in_group))
    } else {
        let port_list = PortList::from_str(first_line)?;
        Ok((PortObjectItem::PortList(port_list), 1))
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let lines: Vec<String> = vec![];
        let result = PortObject::try_from(&lines);
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "Failed to parse port object: Input lines are empty"
        );
    }

    #[test]
    fn test_single_port_list() {
        let lines = vec!["Destination Ports     : TCP-8080 (protocol 6, port 8080)".to_string()];
        let result = PortObject::try_from(&lines);
        assert!(result.is_ok());
        let port_object = result.unwrap();
        assert_eq!(port_object.name, "Destination Ports");
        assert_eq!(port_object.items.len(), 1);
        match &port_object.items[0] {
            PortObjectItem::PortList(port_list) => {
                assert_eq!(port_list.to_string(), "TCP-8080 (protocol 6, port 8080)");
            }
            _ => panic!("Expected PortList"),
        }
    }

    #[test]
    fn test_group_with_ports() {
        let lines = vec![
            "Destination Ports     : HTTP-HTTPS_1 (group)".to_string(),
            "  HTTP (protocol 6, port 80)".to_string(),
            "  HTTPS (protocol 6, port 443)".to_string(),
        ];
        let result = PortObject::try_from(&lines);
        assert!(result.is_ok());
        let port_object = result.unwrap();
        assert_eq!(port_object.name, "Destination Ports");
        assert_eq!(port_object.items.len(), 1);
        match &port_object.items[0] {
            PortObjectItem::Group(group) => {
                assert_eq!(group.name, "HTTP-HTTPS_1");
                assert_eq!(group.port_lists.len(), 2);
            }
            _ => panic!("Expected Group"),
        }
    }

    #[test]
    fn test_mixed_objects() {
        let lines = vec![
            "Destination Ports     : HTTP-HTTPS_1 (group)".to_string(),
            "  HTTP (protocol 6, port 80)".to_string(),
            "  HTTPS (protocol 6, port 443)".to_string(),
            "TCP-8080 (protocol 6, port 8080)".to_string(),
            "protocol 6, port 33434".to_string(),
        ];
        let result = PortObject::try_from(&lines);
        assert!(result.is_ok());
        let port_object = result.unwrap();
        assert_eq!(port_object.name, "Destination Ports");
        assert_eq!(port_object.items.len(), 3);
        match &port_object.items[0] {
            PortObjectItem::Group(group) => {
                assert_eq!(group.name, "HTTP-HTTPS_1");
                assert_eq!(group.port_lists.len(), 2);
            }
            _ => panic!("Expected Group"),
        }
        match &port_object.items[1] {
            PortObjectItem::PortList(port_list) => {
                assert_eq!(port_list.to_string(), "TCP-8080 (protocol 6, port 8080)");
            }
            _ => panic!("Expected PortList"),
        }
        match &port_object.items[2] {
            PortObjectItem::PortList(port_list) => {
                assert_eq!(
                    port_list.to_string(),
                    "protocol 6, port 33434 (protocol 6, port 33434)"
                );
            }
            _ => panic!("Expected PortList"),
        }
    }
}
