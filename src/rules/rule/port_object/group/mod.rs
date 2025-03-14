use std::str::FromStr;

pub mod port_list;
use port_list::PortList;

#[derive(Debug)]
pub struct Group {
    pub name: String,
    pub port_lists: Vec<PortList>,
}

#[derive(thiserror::Error, Debug)]
pub enum GroupError {
    #[error("Fail to parse port group: {0}")]
    General(String),
    #[error("Failed to parse port group: {0}")]
    PortListError(#[from] port_list::PortListError),
}

impl TryFrom<&Vec<String>> for Group {
    type Error = GroupError;

    // Example:
    // HTTP-HTTPS_1 (group)
    //   HTTP (protocol 6, port 80)
    //   HTTPS (protocol 6, port 443)

    fn try_from(lines: &Vec<String>) -> Result<Self, Self::Error> {
        if let [title, ..] = lines.as_slice() {
            if !title.contains(" (group)") {
                return Err(GroupError::General(format!(
                    "Invalid group format, should contain (group) {}",
                    title
                )));
            }
            let name = title.split('(').next().unwrap().trim().to_string();
            let mut port_lists = vec![];

            for line in &lines[1..] {
                let port = line.trim();
                if !port.is_empty() {
                    port_lists.push(PortList::from_str(port)?);
                }
            }

            Ok(Self { name, port_lists })
        } else {
            Err(GroupError::General("Invalid group format.".to_string()))
        }
    }
}

impl Group {
    pub fn capacity(&self) -> u64 {
        todo!("Implement Group::capacity");
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::rule::port_object::group::port_list::PortListError;

    use super::*;

    #[test]
    fn test_valid_group() {
        let lines = vec![
            "HTTP-HTTPS_1 (group)".to_string(),
            "  HTTP (protocol 6, port 80)".to_string(),
            "  HTTPS (protocol 6, port 443)".to_string(),
        ];
        let group = Group::try_from(&lines).unwrap();
        assert_eq!(group.name, "HTTP-HTTPS_1");
        assert_eq!(group.port_lists.len(), 2);
    }

    #[test]
    fn test_invalid_group_format() {
        let lines = vec![
            "HTTP-HTTPS_1".to_string(),
            "  HTTP (protocol 6, port 80)".to_string(),
            "  HTTPS (protocol 6, port 443)".to_string(),
        ];
        let result = Group::try_from(&lines);
        assert!(result.is_err());
        if let Err(GroupError::General(msg)) = result {
            assert_eq!(
                msg,
                "Invalid group format, should contain (group) HTTP-HTTPS_1"
            );
        } else {
            panic!("Expected GroupError::General");
        }
    }

    #[test]
    fn test_empty_group() {
        let lines = vec!["HTTP-HTTPS_1 (group)".to_string()];
        let group = Group::try_from(&lines).unwrap();
        assert_eq!(group.name, "HTTP-HTTPS_1");
        assert!(group.port_lists.is_empty());
    }

    #[test]
    fn test_invalid_port_list() {
        let lines = vec![
            "HTTP-HTTPS_1 (group)".to_string(),
            "  INVALID_PORT".to_string(),
        ];
        let result = Group::try_from(&lines);
        dbg!(&result);
        assert!(result.is_err());
        if let Err(GroupError::PortListError(port_list::PortListError::CommonError(
            port_list::tcp_udp::common::CommonError::Protocol(msg),
        ))) = result
        {
            assert_eq!(msg, "Missing 'protocol' prefix in: (INVALID_PORT)");
        } else {
            panic!("Expected GroupError::PortListError");
        }
    }

    #[test]
    fn test_empty_lines() {
        let lines: Vec<String> = vec![];
        let result = Group::try_from(&lines);
        assert!(result.is_err());
        if let Err(GroupError::General(msg)) = result {
            assert_eq!(msg, "Invalid group format.");
        } else {
            panic!("Expected GroupError::General");
        }
    }

    #[test]
    fn test_group_capacity_single_port_list() {
        let lines = vec![
            "HTTP-HTTPS_1 (group)".to_string(),
            "  HTTP (protocol 6, port 80)".to_string(),
        ];
        let group = Group::try_from(&lines).unwrap();
        assert_eq!(group.capacity(), 1); // Single port
    }

    #[test]
    fn test_group_capacity_multiple_port_lists() {
        let lines = vec![
            "HTTP-HTTPS_1 (group)".to_string(),
            "  HTTP (protocol 6, port 80)".to_string(),
            "  HTTPS (protocol 6, port 443)".to_string(),
        ];
        let group = Group::try_from(&lines).unwrap();
        assert_eq!(group.capacity(), 2); // Two ports
    }

    #[test]
    fn test_group_capacity_port_range() {
        let lines = vec![
            "HTTP-HTTPS_1 (group)".to_string(),
            "  HTTP (protocol 6, port 80-81)".to_string(),
        ];
        let group = Group::try_from(&lines).unwrap();
        assert_eq!(group.capacity(), 1); // Port range 80-81
    }

    #[test]
    fn test_group_capacity_empty_group() {
        let lines = vec!["HTTP-HTTPS_1 (group)".to_string()];
        let group = Group::try_from(&lines).unwrap();
        assert_eq!(group.capacity(), 0); // No ports
    }

    #[test]
    fn test_group_capacity_mixed_ports_and_ranges() {
        let lines = vec![
            "HTTP-HTTPS_1 (group)".to_string(),
            "  HTTP (protocol 6, port 80)".to_string(),
            "  HTTPS (protocol 6, port 443-445)".to_string(),
        ];
        let group = Group::try_from(&lines).unwrap();
        assert_eq!(group.capacity(), 2); // 1 port + 3 ports in range
    }
}
