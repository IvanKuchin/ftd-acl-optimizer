pub mod network_object;
use std::collections::HashMap;

use network_object::NetworkObject;

mod protocol_object;
use protocol_object::ProtocolObject;

use network_object::network_object_optimized::NetworkObjectOptimized;
use protocol_object::protocol_list_optimized::ProtocolListOptimized;

#[derive(Debug)]
pub struct Rule {
    name: String,
    src_networks: Option<NetworkObject>,
    dst_networks: Option<NetworkObject>,
    src_protocols: Option<ProtocolObject>,
    dst_protocols: Option<ProtocolObject>,
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
    PortObjectError(#[from] protocol_object::PortObjectError),
    #[error("Fail to parse rule name: {0}")]
    RuleNameParsingError(String),
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

        let src_networks = match source_networks.is_empty() {
            true => None,
            false => Some(NetworkObject::try_from(&source_networks).map_err(|e| {
                RuleError::General2(
                    format!("source networks ({:?})", source_networks).to_owned(),
                    e.to_string(),
                )
            })?),
        };
        let dst_networks = match destination_networks.is_empty() {
            true => None,
            false => Some(NetworkObject::try_from(&destination_networks).map_err(|e| {
                RuleError::General2(
                    format!("destination networks ({:?})", destination_networks).to_owned(),
                    e.to_string(),
                )
            })?),
        };

        let src_protocols = match source_ports.is_empty() {
            true => None,
            false => Some(ProtocolObject::try_from(&source_ports)?),
        };
        let dst_protocols = match destination_ports.is_empty() {
            true => None,
            false => Some(ProtocolObject::try_from(&destination_ports)?),
        };

        Ok(Self {
            name,
            src_networks,
            dst_networks,
            src_protocols,
            dst_protocols,
        })
    }
}

impl Rule {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn capacity(&self) -> u64 {
        let src_protocols_opt = self.src_protocols.as_ref().map(|p| p.optimize());
        let dst_protocols_opt = self.dst_protocols.as_ref().map(|p| p.optimize());
        let protocol_factor = get_protocol_factor(&src_protocols_opt, &dst_protocols_opt);

        let src_networks_capacity = self.src_networks.as_ref().map_or(1, |n| n.capacity());
        let dst_networks_capacity = self.dst_networks.as_ref().map_or(1, |n| n.capacity());

        src_networks_capacity * dst_networks_capacity * protocol_factor
    }

    pub fn optimized_capacity(&self) -> u64 {
        let src_protocols_opt = self.src_protocols.as_ref().map(|p| p.optimize());
        let dst_protocols_opt = self.dst_protocols.as_ref().map(|p| p.optimize());
        let protocol_factor = get_protocol_factor(&src_protocols_opt, &dst_protocols_opt);

        let (src_networks_opt, dst_networks_opt) = self.get_optimized_networks();

        let src_networks_capacity = src_networks_opt.as_ref().map_or(1, |n| n.capacity());
        let dst_networks_capacity = dst_networks_opt.as_ref().map_or(1, |n| n.capacity());

        src_networks_capacity * dst_networks_capacity * protocol_factor
    }

    pub fn get_optimized_networks(
        &self,
    ) -> (
        Option<NetworkObjectOptimized>,
        Option<NetworkObjectOptimized>,
    ) {
        (
            self.src_networks.as_ref().map(|n| n.optimize()),
            self.dst_networks.as_ref().map(|n| n.optimize()),
        )
    }
}

/// Calculate the protocol factor based on the src and dst protocols
/// For example:  
/// src_protocols = [TCP, UDP, TCP] -> (TCP, 2 times), (UDP, 1 time)  
/// dst_protocols = [TCP, UDP, UDP] -> (TCP, 1 time),  (UDP, 2 times)  
/// protocol_factor =  TCP (2 * 1) + UDP (1 * 2) = 2 + 2 = 4
fn get_protocol_factor(
    src_ports: &Option<Vec<ProtocolListOptimized>>,
    dst_ports: &Option<Vec<ProtocolListOptimized>>,
) -> u64 {
    let src_protocols = src_ports
        .as_ref()
        .map_or(HashMap::new(), |p| protocol_freq_distribution(p));
    let dst_protocols = dst_ports
        .as_ref()
        .map_or(HashMap::new(), |p| protocol_freq_distribution(p));

    if src_protocols.is_empty() && dst_protocols.is_empty() {
        return 1;
    }

    let (longest, shortest) = if src_protocols.len() > dst_protocols.len() {
        (src_protocols, dst_protocols)
    } else {
        (dst_protocols, src_protocols)
    };

    longest.iter().fold(0, |acc, (protocol, count1)| {
        let count2 = shortest.get(protocol).unwrap_or(&1);
        acc + (*count1 * *count2)
    })
}

fn protocol_freq_distribution(l3_l4_proto: &[ProtocolListOptimized]) -> HashMap<u8, u64> {
    let protocol_freq = l3_l4_proto.iter().fold(HashMap::new(), |mut acc, p| {
        let protocol = p.get_protocol();
        let count = acc.entry(protocol).or_insert(0);
        *count += 1;
        acc
    });

    protocol_freq
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
        .split("-[ Rule: ")
        .nth(1)
        .ok_or(RuleError::RuleNameParsingError(line.clone()))?
        .split(" ]-")
        .next()
        .ok_or(RuleError::RuleNameParsingError(line.clone()))?;
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
    use protocol_object::ProtocolObject;

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
        assert_eq!(name, "Custom_rule2 | FM-15046".to_string());
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
        let source_networks = Some(
            NetworkObject::try_from(&vec![
                "Source Networks       : Internal (group)".to_string(),
                "OBJ-192.168.0.0 (192.168.0.0/16)".to_string(),
                "OBJ-172.17.0.0 (172.17.0.0/16)".to_string(),
            ])
            .unwrap(),
        );
        let destination_networks = Some(
            NetworkObject::try_from(&vec![
                "Destination Networks       : OBJ-10.138.0.0_16 (10.138.0.0/16)".to_string(),
                "10.0.0.0/8".to_string(),
            ])
            .unwrap(),
        );
        let source_ports = Some(
            ProtocolObject::try_from(&vec![
                "Source Ports       : ephemeral (protocol 6, port 1024)".to_string(),
            ])
            .unwrap(),
        );
        let destination_ports = Some(
            ProtocolObject::try_from(&vec![
                "Destination Ports: HTTPS (protocol 6, port 443)".to_string()
            ])
            .unwrap(),
        );

        let rule = Rule {
            name: "Custom_rule2".to_string(),
            src_networks: source_networks,
            dst_networks: destination_networks,
            src_protocols: source_ports,
            dst_protocols: destination_ports,
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
            src_networks: Some(source_networks),
            dst_networks: Some(destination_networks),
            src_protocols: None,
            dst_protocols: None,
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
            ProtocolObject::try_from(&vec![
                "Source Ports       : ephemeral (protocol 6, port 1024)".to_string(),
            ])
            .unwrap(),
        );

        let rule = Rule {
            name: "Custom_rule2".to_string(),
            src_networks: Some(source_networks),
            dst_networks: Some(destination_networks),
            src_protocols: source_ports,
            dst_protocols: None,
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
            ProtocolObject::try_from(&vec![
                "Source Ports       : ephemeral (protocol 6, port 1024-1025)".to_string(),
            ])
            .unwrap(),
        );
        let destination_ports = Some(
            ProtocolObject::try_from(&vec![
                "Destination Ports       : HTTPS (protocol 6, port 443-445)".to_string(),
            ])
            .unwrap(),
        );

        let rule = Rule {
            name: "Custom_rule2".to_string(),
            src_networks: Some(source_networks),
            dst_networks: Some(destination_networks),
            src_protocols: source_ports,
            dst_protocols: destination_ports,
        };

        assert_eq!(rule.capacity(), 2 * 2);
    }

    #[test]
    fn test_protocol_freq_distribution_single_protocol() {
        let l3_l4_proto = ProtocolObject::try_from(&vec![
            "Source Ports       : ephemeral (protocol 6, port 1024-1025)".to_string(),
        ])
        .unwrap()
        .optimize();
        let result = protocol_freq_distribution(&l3_l4_proto);
        assert_eq!(result.get(&6), Some(&1));
    }

    #[test]
    fn test_protocol_freq_distribution_two_protocols() {
        let l3_l4_proto = ProtocolObject::try_from(&vec![
            "Source Ports       : ephemeral (protocol 6, port 1024-1025)".to_string(),
            "HTTP (protocol 6, port 80)".to_string(),
        ])
        .unwrap()
        .optimize();
        let result = protocol_freq_distribution(&l3_l4_proto);
        assert_eq!(result.get(&6), Some(&2));
    }

    #[test]
    fn test_protocol_freq_distribution_three_protocols() {
        let l3_l4_proto = ProtocolObject::try_from(&vec![
            "Source Ports       : ephemeral (protocol 6, port 1024-1025)".to_string(),
            "HTTP (protocol 6, port 80)".to_string(),
            "HTTP over UDP (protocol 17, port 80)".to_string(),
        ])
        .unwrap()
        .optimize();

        let result = protocol_freq_distribution(&l3_l4_proto);
        assert_eq!(result.get(&6), Some(&2));
        assert_eq!(result.get(&17), Some(&1));
    }

    #[test]
    fn test_protocol_freq_distribution_empty() {
        let protocols: Vec<ProtocolListOptimized> = vec![];
        let result = protocol_freq_distribution(&protocols);
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_protocol_factor_empty() {
        let result = get_protocol_factor(&None, &None);
        assert_eq!(result, 1);
    }

    #[test]
    fn test_get_protocol_factor_half_empty_1() {
        let l3_l4_proto = ProtocolObject::try_from(&vec![
            "Source Ports       : ephemeral (protocol 6, port 1024-1025)".to_string(),
            "HTTP (protocol 6, port 80)".to_string(),
            "HTTP over UDP (protocol 17, port 80)".to_string(),
        ])
        .unwrap()
        .optimize();

        let result = get_protocol_factor(&Some(l3_l4_proto), &None);
        assert_eq!(result, 2 + 1);
    }

    #[test]
    fn test_get_protocol_factor_half_empty_2() {
        let l3_l4_proto = ProtocolObject::try_from(&vec![
            "Source Ports       : ephemeral (protocol 6, port 1024-1025)".to_string(),
            "HTTP (protocol 6, port 80)".to_string(),
            "HTTP over UDP (protocol 17, port 80)".to_string(),
        ])
        .unwrap()
        .optimize();

        let result = get_protocol_factor(&None, &Some(l3_l4_proto));
        assert_eq!(result, 2 + 1);
    }

    #[test]
    fn test_get_protocol_factor_1() {
        let src_proto = ProtocolObject::try_from(&vec![
            "Source Ports       : ephemeral (protocol 6, port 1024-1025)".to_string(),
            "HTTP (protocol 6, port 80)".to_string(),
            "HTTP over UDP (protocol 17, port 80)".to_string(),
        ])
        .unwrap()
        .optimize();

        let dst_proto = ProtocolObject::try_from(&vec![
            "Destination Ports       : ephemeral (protocol 6, port 1024-1025)".to_string(),
            "HTTP (protocol 6, port 80)".to_string(),
            "HTTP over UDP (protocol 17, port 80)".to_string(),
        ])
        .unwrap()
        .optimize();

        let result = get_protocol_factor(&Some(src_proto), &Some(dst_proto));
        assert_eq!(result, 2 * 2 + 1);
    }

    #[test]
    fn test_get_protocol_factor_2() {
        let src_proto = ProtocolObject::try_from(&vec![
            "Source Ports       : ephemeral (protocol 6, port 1024-1025)".to_string(),
            "HTTP (protocol 6, port 80)".to_string(),
            "HTTP over UDP (protocol 17, port 80)".to_string(),
        ])
        .unwrap()
        .optimize();

        let dst_proto = ProtocolObject::try_from(&vec![
            "Destination Ports       : ephemeral (protocol 6, port 1024-1025)".to_string(),
            "HTTP (protocol 6, port 80)".to_string(),
            "HTTPS (protocol 6, port 443)".to_string(),
            "HTTP over UDP (protocol 17, port 80)".to_string(),
        ])
        .unwrap()
        .optimize();

        let result = get_protocol_factor(&Some(src_proto), &Some(dst_proto));
        assert_eq!(result, 2 * 3 + 1);
    }

    #[test]
    fn test_get_protocol_factor_3() {
        let src_proto = ProtocolObject::try_from(&vec![
            "Source Ports       : ephemeral (protocol 6, port 1024-1025)".to_string(),
            "HTTP (protocol 6, port 80)".to_string(),
            "HTTP over UDP (protocol 17, port 80)".to_string(),
            "IGMP (protocol 2)".to_string(),
        ])
        .unwrap()
        .optimize();

        let dst_proto = ProtocolObject::try_from(&vec![
            "Destination Ports       : ephemeral (protocol 6, port 1024-1025)".to_string(),
            "HTTP (protocol 6, port 80)".to_string(),
            "HTTPS (protocol 6, port 443)".to_string(),
            "HTTP over UDP (protocol 17, port 80)".to_string(),
        ])
        .unwrap()
        .optimize();

        let result = get_protocol_factor(&Some(src_proto), &Some(dst_proto));
        assert_eq!(result, 2 * 3 + 1 + 1);
    }

    #[test]
    fn test_get_protocol_factor_4() {
        let src_proto = ProtocolObject::try_from(&vec![
            "Source Ports       : ephemeral (protocol 6, port 1024-1025)".to_string(),
            "HTTP (protocol 6, port 80)".to_string(),
            "HTTP over UDP (protocol 17, port 80)".to_string(),
            "IGMP (protocol 2)".to_string(),
        ])
        .unwrap()
        .optimize();

        let dst_proto = ProtocolObject::try_from(&vec![
            "Destination Ports       : ephemeral (protocol 6, port 1024-1025)".to_string(),
            "HTTP (protocol 6, port 80)".to_string(),
            "HTTPS (protocol 6, port 443)".to_string(),
            "FTP (protocol 6, port 21)".to_string(),
            "HTTP over UDP (protocol 17, port 80)".to_string(),
        ])
        .unwrap()
        .optimize();

        let result = get_protocol_factor(&Some(src_proto), &Some(dst_proto));
        assert_eq!(result, 2 * 4 + 1 + 1);
    }

    #[test]
    fn test_parse_rule_1() {
        let rule = "----------[ Rule: Custom_rule2 | FM-15046 ]-----------
    Source Networks       : Internal (group)
        OBJ-192.168.0.0 (192.168.0.0/16)
        OBJ-172.17.0.0 (172.17.0.0/16)
        OBJ-10.11.0.0 (10.11.0.0/16)
      OBJ-198.187.64.0_18 (198.187.64.0/18)
    Destination Networks  : OBJ-10.138.0.0_16 (10.138.0.0/16)
        10.0.0.0/8
        204.99.0.0/16
        172.16.0.0/12
      OBJ-192.168.243.0_24 (192.168.243.0/24)
      OBJ-10.18.46.62-69 (10.18.46.62-10.18.46.69)
    Source Ports     : ephemeral (protocol 6, port 1024)
    Destination Ports  : HTTPS (protocol 6, port 443)
    Logging Configuration";
        let lines: Vec<String> = rule.lines().map(|s| s.to_string()).collect();
        let rule = Rule::try_from(lines).unwrap();
        assert_eq!(rule.name, "Custom_rule2 | FM-15046".to_string());
        assert_eq!(rule.src_networks.as_ref().unwrap().capacity(), 4);
        assert_eq!(rule.dst_networks.as_ref().unwrap().capacity(), 8);
        assert!(rule.src_protocols.is_some());
        assert!(rule.dst_protocols.is_some());
        assert_eq!(rule.capacity(), 32);
    }

    #[test]
    fn test_parse_rule_missing_dst_ports() {
        let rule = "----------[ Rule: Custom_rule2 | FM-15046 ]-----------
    Source Networks       : Internal (group)
        OBJ-192.168.0.0 (192.168.0.0/16)
        OBJ-172.17.0.0 (172.17.0.0/16)
        OBJ-10.11.0.0 (10.11.0.0/16)
      OBJ-198.187.64.0_18 (198.187.64.0/18)
    Destination Networks  : OBJ-10.138.0.0_16 (10.138.0.0/16)
        10.0.0.0/8
        204.99.0.0/16
        172.16.0.0/12
      OBJ-192.168.243.0_24 (192.168.243.0/24)
      OBJ-10.18.46.62-69 (10.18.46.62-10.18.46.69)
    Source Ports     : ephemeral (protocol 6, port 1024)
    Logging Configuration";
        let lines: Vec<String> = rule.lines().map(|s| s.to_string()).collect();
        let rule = Rule::try_from(lines).unwrap();
        assert_eq!(rule.name, "Custom_rule2 | FM-15046".to_string());
        assert_eq!(rule.src_networks.as_ref().unwrap().capacity(), 4);
        assert_eq!(rule.dst_networks.as_ref().unwrap().capacity(), 8);
        assert!(rule.src_protocols.is_some());
        assert!(rule.dst_protocols.is_none());
        assert_eq!(rule.capacity(), 32);
    }

    #[test]
    fn test_parse_rule_missing_src_ports() {
        let rule = "----------[ Rule: Custom_rule2 | FM-15046 ]-----------
    Source Networks       : Internal (group)
        OBJ-192.168.0.0 (192.168.0.0/16)
        OBJ-172.17.0.0 (172.17.0.0/16)
        OBJ-10.11.0.0 (10.11.0.0/16)
      OBJ-198.187.64.0_18 (198.187.64.0/18)
    Destination Networks  : OBJ-10.138.0.0_16 (10.138.0.0/16)
        10.0.0.0/8
        204.99.0.0/16
        172.16.0.0/12
      OBJ-192.168.243.0_24 (192.168.243.0/24)
      OBJ-10.18.46.62-69 (10.18.46.62-10.18.46.69)
    Destination Ports  : HTTPS (protocol 6, port 443)
    Logging Configuration";
        let lines: Vec<String> = rule.lines().map(|s| s.to_string()).collect();
        let rule = Rule::try_from(lines).unwrap();
        assert_eq!(rule.name, "Custom_rule2 | FM-15046".to_string());
        assert_eq!(rule.src_networks.as_ref().unwrap().capacity(), 4);
        assert_eq!(rule.dst_networks.as_ref().unwrap().capacity(), 8);
        assert!(rule.src_protocols.is_none());
        assert!(rule.dst_protocols.is_some());
        assert_eq!(rule.capacity(), 32);
    }

    #[test]
    fn test_parse_rule_missing_dst_networks() {
        let rule = "----------[ Rule: Custom_rule2 | FM-15046 ]-----------
    Source Networks       : Internal (group)
        OBJ-192.168.0.0 (192.168.0.0/16)
        OBJ-172.17.0.0 (172.17.0.0/16)
        OBJ-10.11.0.0 (10.11.0.0/16)
      OBJ-198.187.64.0_18 (198.187.64.0/18)
    Source Ports     : ephemeral (protocol 6, port 1024)
    Destination Ports  : HTTPS (protocol 6, port 443)
    Logging Configuration";
        let lines: Vec<String> = rule.lines().map(|s| s.to_string()).collect();
        let rule = Rule::try_from(lines).unwrap();
        assert_eq!(rule.name, "Custom_rule2 | FM-15046".to_string());
        assert_eq!(rule.src_networks.as_ref().unwrap().capacity(), 4);
        assert!(rule.src_protocols.is_some());
        assert!(rule.dst_protocols.is_some());
        assert_eq!(rule.capacity(), 4);
    }

    #[test]
    fn test_parse_rule_missing_src_networks() {
        let rule = "----------[ Rule: Custom_rule2 | FM-15046 ]-----------
    Destination Networks  : OBJ-10.138.0.0_16 (10.138.0.0/16)
        10.0.0.0/8
        204.99.0.0/16
        172.16.0.0/12
      OBJ-192.168.243.0_24 (192.168.243.0/24)
      OBJ-10.18.46.62-69 (10.18.46.62-10.18.46.69)
    Source Ports     : ephemeral (protocol 6, port 1024)
    Destination Ports  : HTTPS (protocol 6, port 443)
    Logging Configuration";
        let lines: Vec<String> = rule.lines().map(|s| s.to_string()).collect();
        let rule = Rule::try_from(lines).unwrap();
        assert_eq!(rule.name, "Custom_rule2 | FM-15046".to_string());
        assert_eq!(rule.dst_networks.as_ref().unwrap().capacity(), 8);
        assert!(rule.src_protocols.is_some());
        assert!(rule.dst_protocols.is_some());
        assert_eq!(rule.capacity(), 8);
    }

    #[test]
    fn test_optimized_capacity_1() {
        let rule = "----------[ Rule: Custom_rule2 | FM-15046 ]-----------
    Source Networks       : Internal (group)
        OBJ-192.168.100.0 (192.168.100.0/23)
        OBJ-10.11.0.0 (10.11.0.0/16)
        OBJ-172.16.17.0-200 (172.16.17.0-172.16.17.200)
      OBJ-192.168.101.0_24 (192.168.101.0/24)
      OBJ-10.10.0.0_16 (10.10.0.0/16)
      OBJ-172.16.17.64-255 (172.16.17.64-172.16.17.255)
    Destination Networks  : OBJ-10.138.0.0_16 (10.138.0.0/16)
        10.0.0.0/8
        172.16.0.0/12
        192.168.0.0/16        
      OBJ-192.168.243.0_24 (192.168.243.0/24)
      OBJ-10.18.46.62-69 (10.18.46.62-10.18.46.69)
    Source Ports     : ephemeral (protocol 6, port 1024)
       FTP (protocol 6, port 21)
    Destination Ports  : HTTPS (protocol 6, port 443)
       FTP (protocol 6, port 21)
       SSH (protocol 6, port 22)
    Logging Configuration";
        let lines: Vec<String> = rule.lines().map(|s| s.to_string()).collect();
        let rule = Rule::try_from(lines).unwrap();
        assert_eq!(rule.name, "Custom_rule2 | FM-15046".to_string());
        assert_eq!(rule.src_networks.as_ref().unwrap().capacity(), 10);
        assert_eq!(rule.dst_networks.as_ref().unwrap().capacity(), 8);
        assert!(rule.src_protocols.is_some());
        assert!(rule.dst_protocols.is_some());
        assert_eq!(rule.capacity(), 10 * 8 * 2 * 2);
        assert_eq!(rule.optimized_capacity(), 3 * 3 * 2 * 2);
    }

    #[test]
    fn test_optimized_capacity_missing_src_network() {
        let rule = "----------[ Rule: Custom_rule2 | FM-15046 ]-----------
    Destination Networks  : OBJ-10.138.0.0_16 (10.138.0.0/16)
        10.0.0.0/8
        172.16.0.0/12
        192.168.0.0/16        
      OBJ-192.168.243.0_24 (192.168.243.0/24)
      OBJ-10.18.46.62-69 (10.18.46.62-10.18.46.69)
    Source Ports     : ephemeral (protocol 6, port 1024)
       FTP (protocol 6, port 21)
    Destination Ports  : HTTPS (protocol 6, port 443)
       FTP (protocol 6, port 21)
       SSH (protocol 6, port 22)
    Logging Configuration";
        let lines: Vec<String> = rule.lines().map(|s| s.to_string()).collect();
        let rule = Rule::try_from(lines).unwrap();
        assert_eq!(rule.name, "Custom_rule2 | FM-15046".to_string());
        assert_eq!(rule.dst_networks.as_ref().unwrap().capacity(), 8);
        assert!(rule.src_protocols.is_some());
        assert!(rule.dst_protocols.is_some());
        assert_eq!(rule.capacity(), 8 * 2 * 2);
        assert_eq!(rule.optimized_capacity(), 3 * 2 * 2);
    }

    #[test]
    fn test_optimized_capacity_missing_dst_network() {
        let rule = "----------[ Rule: Custom_rule2 | FM-15046 ]-----------
    Source Networks       : Internal (group)
        OBJ-192.168.100.0 (192.168.100.0/23)
        OBJ-10.11.0.0 (10.11.0.0/16)
        OBJ-172.16.17.0-200 (172.16.17.0-172.16.17.200)
      OBJ-192.168.101.0_24 (192.168.101.0/24)
      OBJ-10.10.0.0_16 (10.10.0.0/16)
      OBJ-172.16.17.64-255 (172.16.17.64-172.16.17.255)
    Source Ports     : ephemeral (protocol 6, port 1024)
       FTP (protocol 6, port 21)
    Destination Ports  : HTTPS (protocol 6, port 443)
       FTP (protocol 6, port 21)
       SSH (protocol 6, port 22)
    Logging Configuration";
        let lines: Vec<String> = rule.lines().map(|s| s.to_string()).collect();
        let rule = Rule::try_from(lines).unwrap();
        assert_eq!(rule.name, "Custom_rule2 | FM-15046".to_string());
        assert_eq!(rule.src_networks.as_ref().unwrap().capacity(), 10);
        assert!(rule.src_protocols.is_some());
        assert!(rule.dst_protocols.is_some());
        assert_eq!(rule.capacity(), 10 * 2 * 2);
        assert_eq!(rule.optimized_capacity(), 3 * 2 * 2);
    }

    #[test]
    fn test_optimized_capacity_missing_src_ports() {
        let rule = "----------[ Rule: Custom_rule2 | FM-15046 ]-----------
    Source Networks       : Internal (group)
        OBJ-192.168.100.0 (192.168.100.0/23)
        OBJ-10.11.0.0 (10.11.0.0/16)
        OBJ-172.16.17.0-200 (172.16.17.0-172.16.17.200)
      OBJ-192.168.101.0_24 (192.168.101.0/24)
      OBJ-10.10.0.0_16 (10.10.0.0/16)
      OBJ-172.16.17.64-255 (172.16.17.64-172.16.17.255)
    Destination Networks  : OBJ-10.138.0.0_16 (10.138.0.0/16)
        10.0.0.0/8
        172.16.0.0/12
        192.168.0.0/16        
      OBJ-192.168.243.0_24 (192.168.243.0/24)
      OBJ-10.18.46.62-69 (10.18.46.62-10.18.46.69)
    Destination Ports  : HTTPS (protocol 6, port 443)
       FTP (protocol 6, port 21)
       SSH (protocol 6, port 22)
    Logging Configuration";
        let lines: Vec<String> = rule.lines().map(|s| s.to_string()).collect();
        let rule = Rule::try_from(lines).unwrap();
        assert_eq!(rule.name, "Custom_rule2 | FM-15046".to_string());
        assert_eq!(rule.src_networks.as_ref().unwrap().capacity(), 10);
        assert_eq!(rule.dst_networks.as_ref().unwrap().capacity(), 8);
        assert!(rule.dst_protocols.is_some());
        assert_eq!(rule.capacity(), 10 * 8 * 2);
        assert_eq!(rule.optimized_capacity(), 3 * 3 * 2);
    }

    #[test]
    fn test_optimized_capacity_missing_dst_ports() {
        let rule = "----------[ Rule: Custom_rule2 | FM-15046 ]-----------
    Source Networks       : Internal (group)
        OBJ-192.168.100.0 (192.168.100.0/23)
        OBJ-10.11.0.0 (10.11.0.0/16)
        OBJ-172.16.17.0-200 (172.16.17.0-172.16.17.200)
      OBJ-192.168.101.0_24 (192.168.101.0/24)
      OBJ-10.10.0.0_16 (10.10.0.0/16)
      OBJ-172.16.17.64-255 (172.16.17.64-172.16.17.255)
    Destination Networks  : OBJ-10.138.0.0_16 (10.138.0.0/16)
        10.0.0.0/8
        172.16.0.0/12
        192.168.0.0/16        
      OBJ-192.168.243.0_24 (192.168.243.0/24)
      OBJ-10.18.46.62-69 (10.18.46.62-10.18.46.69)
    Source Ports     : ephemeral (protocol 6, port 1024)
       FTP (protocol 6, port 21)
    Logging Configuration";
        let lines: Vec<String> = rule.lines().map(|s| s.to_string()).collect();
        let rule = Rule::try_from(lines).unwrap();
        assert_eq!(rule.name, "Custom_rule2 | FM-15046".to_string());
        assert_eq!(rule.src_networks.as_ref().unwrap().capacity(), 10);
        assert_eq!(rule.dst_networks.as_ref().unwrap().capacity(), 8);
        assert!(rule.src_protocols.is_some());
        assert_eq!(rule.capacity(), 10 * 8 * 2);
        assert_eq!(rule.optimized_capacity(), 3 * 3 * 2);
    }
}
