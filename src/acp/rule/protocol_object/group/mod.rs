use std::str::FromStr;

pub mod protocol_list;
use protocol_list::ProtocolList;

#[derive(Debug)]
pub struct Group {
    pub _name: String,
    pub port_lists: Vec<ProtocolList>,
}

#[derive(thiserror::Error, Debug)]
pub enum GroupError {
    #[error("Fail to parse port group: {0}")]
    General(String),
    #[error("Failed to parse port group: {0}")]
    PortListError(#[from] protocol_list::PortListError),
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
                    port_lists.push(ProtocolList::from_str(port)?);
                }
            }

            Ok(Self {
                _name: name,
                port_lists,
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
            "HTTP-HTTPS_1 (group)".to_string(),
            "  HTTP (protocol 6, port 80)".to_string(),
            "  HTTPS (protocol 6, port 443)".to_string(),
        ];
        let group = Group::try_from(&lines).unwrap();
        assert_eq!(group._name, "HTTP-HTTPS_1");
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
        assert_eq!(group._name, "HTTP-HTTPS_1");
        assert!(group.port_lists.is_empty());
    }

    #[test]
    fn test_invalid_port_list() {
        let lines = vec![
            "HTTP-HTTPS_1 (group)".to_string(),
            "  INVALID_PORT".to_string(),
        ];
        let result = Group::try_from(&lines);
        assert!(result.is_err());
        if let Err(GroupError::PortListError(protocol_list::PortListError::CommonError(
            protocol_list::tcp_udp::common::CommonError::Protocol(msg),
        ))) = result
        {
            assert_eq!(
                msg,
                "Missing 'protocol' prefix INVALID_PORT in INVALID_PORT"
            );
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
}
