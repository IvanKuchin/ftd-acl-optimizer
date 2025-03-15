mod network_object;
use network_object::NetworkObject;

mod port_object;
use port_object::PortObject;

#[derive(Debug)]
pub struct Rule {
    name: String,
    source_networks: NetworkObject,
    destination_networks: NetworkObject,
    source_ports: Option<PortObject>,
    destination_ports: Option<PortObject>,
}

#[derive(thiserror::Error, Debug)]
pub enum RuleError {
    #[error("Fail to parse rule: {0}")]
    General(String),
    #[error("Fail to parse rule {0}: {1}")]
    General2(String, String),
    #[error("Fail to parse rule: {0}")]
    NetworkObjectError(#[from] network_object::NetworkObjectError),
    #[error("Fail to parse rule: {0}")]
    PortObjectError(#[from] port_object::PortObjectError),
}

impl TryFrom<Vec<String>> for Rule {
    type Error = RuleError;

    // Example
    // ----------[ Rule: Custom_rule2 | FM-15046 ]-----------
    // Source Networks       : Internal (group)
    //     OBJ-192.168.0.0 (192.168.0.0/16)
    //     OBJ-172.17.0.0 (172.17.0.0/16)
    //     OBJ-10.11.0.0 (10.11.0.0/16)
    //   OBJ-198.187.64.0_18 (198.187.64.0/18)
    // Destination Networks  : OBJ-10.138.0.0_16 (10.138.0.0/16)
    //     10.0.0.0/8
    //     204.99.0.0/16
    //     172.16.0.0/12
    //   OBJ-192.168.243.0_24 (192.168.243.0/24)
    //   OBJ-10.18.46.62-69 (10.18.46.62-10.18.46.69)
    // Source Ports     : ephemeral (protocol 6, port 1024)
    // Destination Ports  : HTTPS (protocol 6, port 443)
    // Logging Configuration

    fn try_from(lines: Vec<String>) -> Result<Self, Self::Error> {
        // let mut reader = Reader::from(lines);

        let name = get_name(&lines)?;

        let source_networks: Vec<_> = lines_from_till(
            &lines,
            "Source Networks",
            &[
                "Destination Networks",
                "Source Ports",
                "Destination Ports",
                "Logging",
                "Users",
                "URLs",
                "Safe Search",
                "Logging Configuration",
            ],
        )?;
        let destination_networks: Vec<_> = lines_from_till(
            &lines,
            "Destination Networks",
            &[
                "Source Networks",
                "Source Ports",
                "Destination Ports",
                "Logging",
                "Users",
                "URLs",
                "Safe Search",
                "Logging Configuration",
            ],
        )?;

        let source_ports: Vec<_> = lines_from_till(
            &lines,
            "Source Ports",
            &[
                "Source Networks",
                "Destination Networks",
                "Destination Ports",
                "Logging",
                "Users",
                "URLs",
                "Safe Search",
                "Logging Configuration",
            ],
        )?;
        let destination_ports: Vec<_> = lines_from_till(
            &lines,
            "Destination Ports",
            &[
                "Source Networks",
                "Destination Networks",
                "Source Ports",
                "Logging",
                "Users",
                "URLs",
                "Safe Search",
                "Logging Configuration",
            ],
        )?;

        let source_networks = NetworkObject::try_from(&source_networks)?;
        let destination_networks = NetworkObject::try_from(&destination_networks)?;
        let source_ports = match source_ports.is_empty() {
            true => None,
            false => Some(PortObject::try_from(&source_ports)?),
        };
        let destination_ports = match destination_ports.is_empty() {
            true => None,
            false => Some(PortObject::try_from(&destination_ports)?),
        };

        Ok(Self {
            name,
            source_networks,
            destination_networks,
            source_ports,
            destination_ports,
        })
    }
}

impl Rule {
    pub fn capacity(&self) -> u64 {
        self.source_networks.capacity() * self.destination_networks.capacity()
        // * self.source_ports.as_ref().map_or(1, |p| p.capacity())
        // * self.destination_ports.as_ref().map_or(1, |p| p.capacity())
    }
}

fn get_name(lines: &[String]) -> Result<String, RuleError> {
    let line = lines
        .iter()
        .find(|line| line.contains("Rule: "))
        .ok_or(RuleError::General(format!(
            "Line with rule name not found ({:?})",
            lines
        )))?;
    let name = line
        .split_whitespace()
        .nth(2)
        .ok_or(RuleError::General(format!(
            "Rule name not found in line: {:?}",
            line
        )))?;
    Ok(name.to_string())
}

fn lines_from_till(lines: &[String], start: &str, end: &[&str]) -> Result<Vec<String>, RuleError> {
    let lines: Vec<_> = lines
        .iter()
        .skip_while(|line| !line.contains(start))
        .take_while(|line| !end.iter().any(|&e| line.contains(e)))
        .map(|line| line.to_string())
        .collect();

    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use network_object::NetworkObject;
    use port_object::PortObject;

    #[test]
    fn test_lines_from_till1() {
        let lines = vec![
            "----------[ Rule: Custom_rule2 | FM-15046 ]-----------".to_string(),
            "Source Networks       : Internal (group)".to_string(),
            "OBJ-192.168.0.0 (192.168.0.0/16)".to_string(),
            "Destination Networks  : OBJ-10.138.0.0_16 (10.138.0.0/16)".to_string(),
            "10.0.0.0/8".to_string(),
        ];
        let result = lines_from_till(&lines, "Source Networks", &["Destination Networks"]).unwrap();
        assert_eq!(
            result,
            vec![
                "Source Networks       : Internal (group)".to_string(),
                "OBJ-192.168.0.0 (192.168.0.0/16)".to_string(),
            ]
        );
    }

    #[test]
    fn test_lines_from_till2() {
        let lines = vec![
            "----------[ Rule: Custom_rule2 | FM-15046 ]-----------".to_string(),
            "Source Networks       : Internal (group)".to_string(),
            "OBJ-192.168.0.0 (192.168.0.0/16)".to_string(),
            "Destination Networks  : OBJ-10.138.0.0_16 (10.138.0.0/16)".to_string(),
            "10.0.0.0/8".to_string(),
        ];
        let result = lines_from_till(&lines, "Destination Networks", &["Source Networks"]).unwrap();
        assert_eq!(
            result,
            vec![
                "Destination Networks  : OBJ-10.138.0.0_16 (10.138.0.0/16)".to_string(),
                "10.0.0.0/8".to_string(),
            ]
        );
    }

    #[test]
    fn test_lines_from_till_no_match() {
        let lines = vec!["Some other line".to_string(), "Another line".to_string()];
        let result = lines_from_till(&lines, "Source Networks", &["Destination Networks"]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_lines_from_till_with_multiple_end_markers() {
        let lines = vec![
            "----------[ Rule: Custom_rule2 | FM-15046 ]-----------".to_string(),
            "Source Networks       : Internal (group)".to_string(),
            "OBJ-192.168.0.0 (192.168.0.0/16)".to_string(),
            "Destination Networks  : OBJ-10.138.0.0_16 (10.138.0.0/16)".to_string(),
            "10.0.0.0/8".to_string(),
            "Source Ports     : ephemeral (protocol 6, port 1024)".to_string(),
            "Destination Ports  : HTTPS (protocol 6, port 443)".to_string(),
        ];
        let result = lines_from_till(
            &lines,
            "Source Networks",
            &["Destination Networks", "Source Ports"],
        )
        .unwrap();
        assert_eq!(
            result,
            vec![
                "Source Networks       : Internal (group)".to_string(),
                "OBJ-192.168.0.0 (192.168.0.0/16)".to_string(),
            ]
        );
    }

    #[test]
    fn test_lines_from_till_with_no_end_marker() {
        let lines = vec![
            "----------[ Rule: Custom_rule2 | FM-15046 ]-----------".to_string(),
            "Source Networks       : Internal (group)".to_string(),
            "OBJ-192.168.0.0 (192.168.0.0/16)".to_string(),
            "OBJ-172.17.0.0 (172.17.0.0/16)".to_string(),
            "OBJ-10.11.0.0 (10.11.0.0/16)".to_string(),
        ];
        let result = lines_from_till(&lines, "Source Networks", &["Nonexistent Marker"]).unwrap();
        assert_eq!(
            result,
            vec![
                "Source Networks       : Internal (group)".to_string(),
                "OBJ-192.168.0.0 (192.168.0.0/16)".to_string(),
                "OBJ-172.17.0.0 (172.17.0.0/16)".to_string(),
                "OBJ-10.11.0.0 (10.11.0.0/16)".to_string(),
            ]
        );
    }

    #[test]
    fn test_get_name_with_valid_data() {
        let lines = vec![
            "----------[ Rule: Custom_rule2 | FM-15046 ]-----------".to_string(),
            "Source Networks       : Internal (group)".to_string(),
            "OBJ-192.168.0.0 (192.168.0.0/16)".to_string(),
            "OBJ-172.17.0.0 (172.17.0.0/16)".to_string(),
            "OBJ-10.11.0.0 (10.11.0.0/16)".to_string(),
        ];
        let name = get_name(&lines).unwrap();
        assert_eq!(name, "Custom_rule2");
    }

    #[test]
    fn test_get_name_with_invalid_data() {
        let lines = vec!["Some random line".to_string()];
        let name = get_name(&lines);
        assert!(name.is_err());
    }

    #[test]
    fn test_lines_from_till_with_no_start_marker() {
        let lines = vec![
            "Some random line".to_string(),
            "Another random line".to_string(),
        ];
        let result = lines_from_till(&lines, "Nonexistent Marker", &["Another Marker"]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_rule_capacity_with_all_components() {
        let source_networks = NetworkObject::try_from(&vec![
            "Source Networks       : Internal (group)".to_string(),
            "OBJ-192.168.0.0 (192.168.0.0/16)".to_string(),
            "OBJ-172.17.0.0 (172.17.0.0/16)".to_string(),
        ])
        .unwrap();
        let destination_networks = NetworkObject::try_from(&vec![
            "Destination Networks       : OBJ-10.138.0.0_16 (10.138.0.0/16)".to_string(),
            "10.0.0.0/8".to_string(),
        ])
        .unwrap();
        let source_ports = Some(
            PortObject::try_from(&vec![
                "Source Ports       : ephemeral (protocol 6, port 1024)".to_string(),
            ])
            .unwrap(),
        );
        let destination_ports = Some(
            PortObject::try_from(&vec![
                "Destination Ports: HTTPS (protocol 6, port 443)".to_string()
            ])
            .unwrap(),
        );

        let rule = Rule {
            name: "Custom_rule2".to_string(),
            source_networks,
            destination_networks,
            source_ports,
            destination_ports,
        };

        assert_eq!(rule.capacity(), 2 * 2);
    }

    #[test]
    fn test_rule_capacity_without_ports() {
        let source_networks = NetworkObject::try_from(&vec![
            "Source Networks       : Internal (group)".to_string(),
            "OBJ-192.168.0.0 (192.168.0.0/16)".to_string(),
            "OBJ-172.17.0.0 (172.17.0.0/16)".to_string(),
        ])
        .unwrap();
        let destination_networks = NetworkObject::try_from(&vec![
            "Destination Networks       : OBJ-10.138.0.0_16 (10.138.0.0/16)".to_string(),
            "10.0.0.0/8".to_string(),
        ])
        .unwrap();

        let rule = Rule {
            name: "Custom_rule2".to_string(),
            source_networks,
            destination_networks,
            source_ports: None,
            destination_ports: None,
        };

        assert_eq!(rule.capacity(), 2 * 2);
    }

    #[test]
    fn test_rule_capacity_with_one_port() {
        let source_networks = NetworkObject::try_from(&vec![
            "Source Networks       : Internal (group)".to_string(),
            "OBJ-192.168.0.0 (192.168.0.0/16)".to_string(),
            "OBJ-172.17.0.0 (172.17.0.0/16)".to_string(),
        ])
        .unwrap();
        let destination_networks = NetworkObject::try_from(&vec![
            "Destination Networks       : OBJ-10.138.0.0_16 (10.138.0.0/16)".to_string(),
            "10.0.0.0/8".to_string(),
        ])
        .unwrap();
        let source_ports = Some(
            PortObject::try_from(&vec![
                "Source Ports       : ephemeral (protocol 6, port 1024)".to_string(),
            ])
            .unwrap(),
        );

        let rule = Rule {
            name: "Custom_rule2".to_string(),
            source_networks,
            destination_networks,
            source_ports,
            destination_ports: None,
        };

        assert_eq!(rule.capacity(), 2 * 2);
    }

    #[test]
    fn test_rule_capacity_with_port_ranges() {
        let source_networks = NetworkObject::try_from(&vec![
            "Source Networks       : Internal (group)".to_string(),
            "OBJ-192.168.0.0 (192.168.0.0/16)".to_string(),
            "OBJ-172.17.0.0 (172.17.0.0/16)".to_string(),
        ])
        .unwrap();
        let destination_networks = NetworkObject::try_from(&vec![
            "Destination Networks       : OBJ-10.138.0.0_16 (10.138.0.0/16)".to_string(),
            "10.0.0.0/8".to_string(),
        ])
        .unwrap();
        let source_ports = Some(
            PortObject::try_from(&vec![
                "Source Ports       : ephemeral (protocol 6, port 1024-1025)".to_string(),
            ])
            .unwrap(),
        );
        let destination_ports = Some(
            PortObject::try_from(&vec![
                "Destination Ports       : HTTPS (protocol 6, port 443-445)".to_string(),
            ])
            .unwrap(),
        );

        let rule = Rule {
            name: "Custom_rule2".to_string(),
            source_networks,
            destination_networks,
            source_ports,
            destination_ports,
        };

        assert_eq!(rule.capacity(), 2 * 2);
    }
}
