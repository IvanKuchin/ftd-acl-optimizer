use std::str::FromStr;

mod group;
use group::protocol_list::{self, ProtocolList};
use group::Group;

use super::network_object::utilities;

pub mod protocol_list_optimized;
use protocol_list_optimized::ProtocolListOptimized;

mod protocol_object_item;
use protocol_object_item::ProtocolObjectItem;

pub mod description;

#[derive(Debug)]
pub struct ProtocolObject {
    _name: String,
    items: Vec<ProtocolObjectItem>,
}

#[derive(thiserror::Error, Debug)]
pub enum PortObjectError {
    #[error("Failed to parse port object: {0}")]
    General(String),
    #[error("Failed to parse port object: {0}")]
    PortListError(#[from] protocol_list::PortListError),
    #[error("Failed to parse port object: {0}")]
    GroupError(#[from] group::GroupError),
    #[error("Fail to parse port object: {0}")]
    NameExtractionError(#[from] utilities::UtilitiesError),
}

impl TryFrom<&Vec<String>> for ProtocolObject {
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

        Ok(ProtocolObject { _name: name, items })
    }
}

impl ProtocolObject {
    /// Optimizes all PortLists inside the PortObject.
    /// Those optimizations automatically performed by FTD
    pub fn optimize(&self) -> Vec<ProtocolListOptimized> {
        let protocol_lists: Vec<&ProtocolList> = self
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let l3_items: Vec<&ProtocolList> = protocol_lists
            .iter()
            .filter(|port_list| !port_list.is_l4())
            .copied()
            .collect();
        let unique_l3_items = unique_l3_items(l3_items);

        let unique_l3_items: Vec<ProtocolListOptimized> = unique_l3_items
            .iter()
            .map(|port_list| ProtocolListOptimized::from(port_list))
            .collect();

        let l4_items: Vec<&ProtocolList> = protocol_lists
            .iter()
            .filter(|port_list| port_list.is_l4())
            .copied()
            .collect();
        let optimized_l4 = optimize_l4_items(l4_items);

        unique_l3_items
            .into_iter()
            .chain(optimized_l4)
            .collect::<Vec<_>>()
    }
}

/// Get the next object from input lines (either Group or PortList) and the number of lines to consume.
fn get_object(lines: &[String]) -> Result<(ProtocolObjectItem, usize), PortObjectError> {
    if lines.is_empty() {
        return Err(PortObjectError::General(
            "Input lines are empty".to_string(),
        ));
    }

    let first_line = lines[0].as_str();
    if first_line.contains("(group)") {
        let lines_in_group = utilities::calculate_lines_in_group(lines)?;
        let group = Group::try_from(&lines[0..lines_in_group].to_vec())?;
        Ok((ProtocolObjectItem::Group(group), lines_in_group))
    } else {
        let port_list = ProtocolList::from_str(first_line)?;
        Ok((ProtocolObjectItem::ProtocolList(port_list), 1))
    }
}

fn unique_l3_items(port_lists: Vec<&ProtocolList>) -> Vec<&ProtocolList> {
    let unique_items = port_lists
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .copied()
        .collect();

    unique_items
}

fn optimize_l4_items(to_optimize: Vec<&ProtocolList>) -> Vec<ProtocolListOptimized> {
    let mut to_optimize = to_optimize;
    to_optimize
        .sort_by_key(|item| ((item.get_protocol() as u32) << 16) + item.get_ports().0 as u32);

    let mut result = vec![];

    if to_optimize.is_empty() {
        return result;
    }

    let mut optimized_items = ProtocolListOptimized::from(to_optimize[0]);

    for next_item in to_optimize.into_iter().skip(1) {
        if optimized_items.get_protocol() == next_item.get_protocol() {
            let (_, curr_end) = optimized_items.get_ports();
            let (next_start, next_end) = next_item.get_ports();

            if next_start as u32 <= curr_end as u32 + 1 {
                let verb = description::verb(curr_end as u32, next_start as u32, next_end as u32);
                let new_name = format!(
                    "{} {verb} {}",
                    optimized_items.get_name(),
                    next_item.get_name()
                );

                optimized_items.append(next_item);
                optimized_items.set_name(new_name);
            } else {
                result.push(optimized_items);
                optimized_items = ProtocolListOptimized::from(next_item);
            }
        } else {
            result.push(optimized_items);
            optimized_items = ProtocolListOptimized::from(next_item);
        }
    }

    result.push(optimized_items);

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    impl ProtocolObject {
        // capacity calculation does not work on a port object level, it should be done on a rule level
        // due to capacity calculation must be done on a same L3 protocol. For example: source TCP with destination TCP
        fn capacity(&self) -> u64 {
            self.optimize().len() as u64
        }
    }

    #[test]
    fn test_empty_input() {
        let lines: Vec<String> = vec![];
        let result = ProtocolObject::try_from(&lines);
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "Failed to parse port object: Input lines are empty"
        );
    }

    #[test]
    fn test_single_port_list() {
        let lines = vec!["Destination Ports     : TCP-8080 (protocol 6, port 8080)".to_string()];
        let result = ProtocolObject::try_from(&lines);
        assert!(result.is_ok());
        let port_object = result.unwrap();
        assert_eq!(port_object._name, "Destination Ports");
        assert_eq!(port_object.items.len(), 1);
        match &port_object.items[0] {
            ProtocolObjectItem::ProtocolList(port_list) => {
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
        let result = ProtocolObject::try_from(&lines);
        assert!(result.is_ok());
        let port_object = result.unwrap();
        assert_eq!(port_object._name, "Destination Ports");
        assert_eq!(port_object.items.len(), 1);
        match &port_object.items[0] {
            ProtocolObjectItem::Group(group) => {
                assert_eq!(group._name, "HTTP-HTTPS_1");
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
        let result = ProtocolObject::try_from(&lines);
        assert!(result.is_ok());
        let port_object = result.unwrap();
        assert_eq!(port_object._name, "Destination Ports");
        assert_eq!(port_object.items.len(), 3);
        match &port_object.items[0] {
            ProtocolObjectItem::Group(group) => {
                assert_eq!(group._name, "HTTP-HTTPS_1");
                assert_eq!(group.port_lists.len(), 2);
            }
            _ => panic!("Expected Group"),
        }
        match &port_object.items[1] {
            ProtocolObjectItem::ProtocolList(port_list) => {
                assert_eq!(port_list.to_string(), "TCP-8080 (protocol 6, port 8080)");
            }
            _ => panic!("Expected PortList"),
        }
        match &port_object.items[2] {
            ProtocolObjectItem::ProtocolList(port_list) => {
                assert_eq!(
                    port_list.to_string(),
                    "protocol 6, port 33434 (protocol 6, port 33434)"
                );
            }
            _ => panic!("Expected PortList"),
        }
    }

    #[test]
    fn test_port_object_capacity_single_port_list() {
        let lines = vec!["Destination Ports     : TCP-8080 (protocol 6, port 8080)".to_string()];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        assert_eq!(port_object.capacity(), 1); // Single port
    }

    #[test]
    fn test_port_object_capacity_multiple_port_lists() {
        let lines = vec![
            "Destination Ports     : HTTP-HTTPS_1 (group)".to_string(),
            "  HTTP (protocol 6, port 80)".to_string(),
            "  HTTPS (protocol 6, port 443)".to_string(),
            "TCP-8080 (protocol 6, port 8080)".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        assert_eq!(port_object.capacity(), 3); // Three ports
    }

    #[test]
    fn test_port_object_capacity_port_range() {
        let lines = vec![
            "Destination Ports     : HTTP-HTTPS_1 (group)".to_string(),
            "  HTTP (protocol 6, port 80-81)".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        assert_eq!(port_object.capacity(), 1); // Port range 80-81
    }

    #[test]
    fn test_port_object_capacity_empty() {
        let lines = vec!["Destination Ports     : HTTP-HTTPS_1 (group)".to_string()];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        assert_eq!(port_object.capacity(), 0); // No ports
    }

    #[test]
    fn test_port_object_capacity_mixed_ports_and_ranges() {
        let lines = vec![
            "Destination Ports     : HTTP-HTTPS_1 (group)".to_string(),
            "  HTTP (protocol 6, port 80)".to_string(),
            "  HTTPS (protocol 6, port 443-445)".to_string(),
            "TCP-8080 (protocol 6, port 8080)".to_string(),
            "protocol 6, port 33434".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        assert_eq!(port_object.capacity(), 4); // 1 port + 3 ports in range + 1 port + 1 port
    }

    #[test]
    fn test_port_object_unique_l3_items_1() {
        let lines = vec![
            "Destination Ports     : HTTP-HTTPS_1 (group)".to_string(),
            "  IGMP (protocol 2)".to_string(),
            "  GGMP (protocol 3)".to_string(),
            "EIGRP (protocol 88)".to_string(),
            "ESP (protocol 50)".to_string(),
            "AH (protocol 51)".to_string(),
            "protocol 10".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let l3_items = unique_l3_items(port_lists);
        assert_eq!(l3_items.len(), 6);
    }

    #[test]
    fn test_port_object_unique_l3_items_duplicate() {
        let lines = vec![
            "Destination Ports     : HTTP-HTTPS_2 (group)".to_string(),
            "  BGP (protocol 17)".to_string(),
            "  RIP (protocol 9)".to_string(),
            "LDP (protocol 39)".to_string(),
            "LDP (protocol 39)".to_string(),
            "PIM (protocol 103)".to_string(),
            "protocol 6".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let l3_items = unique_l3_items(port_lists);
        assert_eq!(l3_items.len(), 5);
    }

    #[test]
    fn test_port_object_unique_l3_items_duplicates_2() {
        let lines = vec![
            "Destination Ports     : SomeProtocols (group)".to_string(),
            "  BGP (protocol 17)".to_string(),
            "  RIP (protocol 9)".to_string(),
            "LDP (protocol 39)".to_string(),
            "ldp (protocol 39)".to_string(),
            "PIM (protocol 103)".to_string(),
            "protocol 6".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let l3_items = unique_l3_items(port_lists);
        assert_eq!(l3_items.len(), 5);
    }

    #[test]
    fn test_port_object_unique_l3_items_duplicates_3() {
        let lines = vec![
            "Destination Ports     : SomeProtocols (group)".to_string(),
            "  BGP (protocol 17)".to_string(),
            "  RIP (protocol 9)".to_string(),
            "LDP (protocol 39)".to_string(),
            "protocol 39".to_string(),
            "PIM (protocol 103)".to_string(),
            "protocol 6".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let l3_items = unique_l3_items(port_lists);
        assert_eq!(l3_items.len(), 5);
    }

    #[test]
    fn test_port_object_unique_l3_items_duplicates_4() {
        let lines = vec![
            "Destination Ports     : ICMP (group)".to_string(),
            "  ICMP1 (protocol 1, type 4, code 11)".to_string(),
            "ICMP1 (protocol 1, type 4, code 12)".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let l3_items = unique_l3_items(port_lists);
        assert_eq!(l3_items.len(), 2);
    }

    #[test]
    fn test_port_object_unique_l3_items_duplicates_5() {
        let lines = vec![
            "Destination Ports     : ICMP (group)".to_string(),
            "  ICMP1 (protocol 1, type 4, code 11)".to_string(),
            "ICMP2 (protocol 1, type 4)".to_string(),
            "ICMP3 (protocol 1, type 4, code 12)".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let l3_items = unique_l3_items(port_lists);
        assert_eq!(l3_items.len(), 3);
    }

    #[test]
    fn test_port_object_unique_l3_items_duplicates_6() {
        let lines = vec![
            "Destination Ports     : ICMP (group)".to_string(),
            "  ICMP1 (protocol 1, type 4, code 11)".to_string(),
            "ICMP2 (protocol 1, type 4, code 11)".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let l3_items = unique_l3_items(port_lists);

        assert_eq!(l3_items.len(), 1);
    }

    #[test]
    fn test_port_object_unique_l3_items_duplicates_in_group() {
        let lines = vec![
            "Destination Ports     : HTTP-HTTPS_2 (group)".to_string(),
            "  BGP (protocol 17)".to_string(),
            "  RIP (protocol 9)".to_string(),
            "  LDP (protocol 39)".to_string(),
            "  ldp (protocol 39)".to_string(),
            "PIM (protocol 103)".to_string(),
            "protocol 6".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let l3_items = unique_l3_items(port_lists);
        assert_eq!(l3_items.len(), 5);
    }

    #[test]
    fn test_port_object_unique_l3_items_duplicates_cross_groups() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  BGP (protocol 17)".to_string(),
            "  RIP (protocol 9)".to_string(),
            "  LDP (protocol 39)".to_string(),
            "MyGroup2 (group)".to_string(),
            "  ldp (protocol 39)".to_string(),
            "PIM (protocol 103)".to_string(),
            "protocol 6".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let l3_items = unique_l3_items(port_lists);
        assert_eq!(l3_items.len(), 5);
    }

    #[test]
    fn test_port_object_unique_l3_items_duplicates_cross_groups_2() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  BGP (protocol 17)".to_string(),
            "  RIP (protocol 9)".to_string(),
            "  LDP (protocol 39)".to_string(),
            "MyGroup2 (group)".to_string(),
            "  ldp (protocol 39)".to_string(),
            "PIM (protocol 103)".to_string(),
            "  LdP (protocol 39)".to_string(),
            "protocol 6".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let l3_items = unique_l3_items(port_lists);
        assert_eq!(l3_items.len(), 5);
    }

    #[test]
    fn test_port_object_unique_l3_items_duplicates_cross_groups_wo_name() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  BGP (protocol 17)".to_string(),
            "  RIP (protocol 9)".to_string(),
            "  LDP (protocol 39)".to_string(),
            "MyGroup2 (group)".to_string(),
            "  ldp (protocol 39)".to_string(),
            "PIM (protocol 103)".to_string(),
            "protocol 39".to_string(),
            "protocol 6".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let l3_items = unique_l3_items(port_lists);
        assert_eq!(l3_items.len(), 5);
    }

    #[test]
    fn test_port_object_capacity_l3_items_1() {
        let lines = vec![
            "Destination Ports     : HTTP-HTTPS_1 (group)".to_string(),
            "  IGMP (protocol 2)".to_string(),
            "  GGMP (protocol 3)".to_string(),
            "EIGRP (protocol 88)".to_string(),
            "ESP (protocol 50)".to_string(),
            "AH (protocol 51)".to_string(),
            "protocol 10".to_string(),
        ];
        assert_eq!(ProtocolObject::try_from(&lines).unwrap().capacity(), 6);
    }

    #[test]
    fn test_port_object_capacity_l3_items_duplicate() {
        let lines = vec![
            "Destination Ports     : HTTP-HTTPS_2 (group)".to_string(),
            "  BGP (protocol 17)".to_string(),
            "  RIP (protocol 9)".to_string(),
            "LDP (protocol 39)".to_string(),
            "LDP (protocol 39)".to_string(),
            "PIM (protocol 103)".to_string(),
            "protocol 6".to_string(),
        ];
        assert_eq!(ProtocolObject::try_from(&lines).unwrap().capacity(), 5);
    }

    #[test]
    fn test_port_object_capacity_l3_items_duplicates_2() {
        let lines = vec![
            "Destination Ports     : HTTP-HTTPS_2 (group)".to_string(),
            "  BGP (protocol 17)".to_string(),
            "  RIP (protocol 9)".to_string(),
            "LDP (protocol 39)".to_string(),
            "ldp (protocol 39)".to_string(),
            "PIM (protocol 103)".to_string(),
            "protocol 6".to_string(),
        ];
        assert_eq!(ProtocolObject::try_from(&lines).unwrap().capacity(), 5);
    }

    #[test]
    fn test_port_object_capacity_l3_items_duplicates_in_group() {
        let lines = vec![
            "Destination Ports     : HTTP-HTTPS_2 (group)".to_string(),
            "  BGP (protocol 17)".to_string(),
            "  RIP (protocol 9)".to_string(),
            "  LDP (protocol 39)".to_string(),
            "  ldp (protocol 39)".to_string(),
            "PIM (protocol 103)".to_string(),
            "protocol 6".to_string(),
        ];
        assert_eq!(ProtocolObject::try_from(&lines).unwrap().capacity(), 5);
    }

    #[test]
    fn test_port_object_capacity_l3_items_duplicates_cross_groups() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  BGP (protocol 17)".to_string(),
            "  RIP (protocol 9)".to_string(),
            "  LDP (protocol 39)".to_string(),
            "MyGroup2 (group)".to_string(),
            "  ldp (protocol 39)".to_string(),
            "PIM (protocol 103)".to_string(),
            "protocol 6".to_string(),
        ];
        assert_eq!(ProtocolObject::try_from(&lines).unwrap().capacity(), 5);
    }

    #[test]
    fn test_port_object_capacity_l3_items_duplicates_cross_groups_2() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  BGP (protocol 17)".to_string(),
            "  RIP (protocol 9)".to_string(),
            "  LDP (protocol 39)".to_string(),
            "MyGroup2 (group)".to_string(),
            "  ldp (protocol 39)".to_string(),
            "PIM (protocol 103)".to_string(),
            "LdP (protocol 39)".to_string(),
            "protocol 6".to_string(),
        ];
        assert_eq!(ProtocolObject::try_from(&lines).unwrap().capacity(), 5);
    }

    #[test]
    fn test_port_object_capacity_l3_items_duplicates_cross_groups_wo_name() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  BGP (protocol 17)".to_string(),
            "  RIP (protocol 9)".to_string(),
            "  LDP (protocol 39)".to_string(),
            "MyGroup2 (group)".to_string(),
            "  ldp (protocol 39)".to_string(),
            "PIM (protocol 103)".to_string(),
            "protocol 39".to_string(),
            "protocol 6".to_string(),
        ];
        assert_eq!(ProtocolObject::try_from(&lines).unwrap().capacity(), 5);
    }

    #[test]
    fn test_port_object_capacity_no_duplicates() {
        let lines = vec![
            "Destination Ports     : NonTCP-NonUDP_1 (group)".to_string(),
            "  MUX (protocol 18)".to_string(),
            "  RIP (protocol 9)".to_string(),
            "EH (protocol 88)".to_string(),
            "protocol 6".to_string(),
        ];
        assert_eq!(ProtocolObject::try_from(&lines).unwrap().capacity(), 4);
    }

    #[test]
    fn test_port_object_capacity_with_additional_protocols() {
        let lines = vec![
            "Destination Ports     : NonTCP-NonUDP_2 (group)".to_string(),
            "  GRE (protocol 47)".to_string(),
            "  ESP (protocol 50)".to_string(),
            "  AH (protocol 51)".to_string(),
            "protocol 6".to_string(),
        ];
        assert_eq!(ProtocolObject::try_from(&lines).unwrap().capacity(), 4);
    }

    #[test]
    fn test_port_object_capacity_l4_items_duplicates_1() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP (protocol 6, port 80)".to_string(),
            "  HTTP2 (protocol 6, port 82)".to_string(),
        ];
        assert_eq!(ProtocolObject::try_from(&lines).unwrap().capacity(), 2);
    }

    #[test]
    fn test_port_object_capacity_l4_items_duplicates_2() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP (protocol 6, port 80-81)".to_string(),
            "  HTTP2 (protocol 6, port 82-83)".to_string(),
        ];
        assert_eq!(ProtocolObject::try_from(&lines).unwrap().capacity(), 1);
    }

    #[test]
    fn test_port_object_capacity_l4_items_duplicates_3() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP (protocol 6, port 80-81)".to_string(),
            "HTTP2 (protocol 6, port 82-83)".to_string(),
        ];
        assert_eq!(ProtocolObject::try_from(&lines).unwrap().capacity(), 1);
    }

    #[test]
    fn test_port_object_capacity_l4_items_duplicates_4() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP (protocol 6, port 80-81)".to_string(),
            "HTTP2 (protocol 6, port 82-83)".to_string(),
            "HTTP3 (protocol 6, port 84-87)".to_string(),
        ];
        assert_eq!(ProtocolObject::try_from(&lines).unwrap().capacity(), 1);
    }

    #[test]
    fn test_port_object_capacity_l4_items_overlap_1() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP (protocol 6, port 85-90)".to_string(),
            "SMTP (protocol 6, port 25)".to_string(),
            "HTTP3 (protocol 6, port 80-87)".to_string(),
        ];
        assert_eq!(ProtocolObject::try_from(&lines).unwrap().capacity(), 2);
    }

    #[test]
    fn test_port_object_capacity_l4_items_overlap_2() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP (protocol 6, port 81-82)".to_string(),
            "SMTP (protocol 6, port 25)".to_string(),
            "HTTP2 (protocol 6, port 82-82)".to_string(),
            "POP3 (protocol 6, port 110)".to_string(),
            "HTTP3 (protocol 6, port 80-80)".to_string(),
        ];
        assert_eq!(ProtocolObject::try_from(&lines).unwrap().capacity(), 3);
    }

    #[test]
    fn test_port_object_capacity_l4_items_overlap_3() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP (protocol 6, port 81-82)".to_string(),
            "SMTP (protocol 6, port 25)".to_string(),
            "HTTP2 (protocol 6, port 82-82)".to_string(),
            "POP3 (protocol 6, port 110)".to_string(),
            "HTTP3 (protocol 6, port 80-80)".to_string(),
            "HTTP4 (protocol 6, port 80-80)".to_string(),
        ];
        assert_eq!(ProtocolObject::try_from(&lines).unwrap().capacity(), 3);
    }

    #[test]
    fn test_port_object_capacity_1() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP (protocol 6, port 80)".to_string(),
            "  SNMP (protocol 17, port 161)".to_string(),
            "SSH (protocol 6, port 22)".to_string(),
            "HTTP2 (protocol 6, port 81-82)".to_string(),
            "FTP (protocol 6, port 21)".to_string(),
            "EIGRP (protocol 88)".to_string(),
        ];
        assert_eq!(ProtocolObject::try_from(&lines).unwrap().capacity(), 4);
    }

    #[test]
    fn test_optimize_l4_items_shadow_1() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP (protocol 6, port 80-82)".to_string(),
            "HTTP2 (protocol 6, port 81-82)".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let optimized = optimize_l4_items(port_lists);
        assert_eq!(optimized.len(), 1);
    }

    #[test]
    fn test_optimize_l4_items_shadow_2() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP (protocol 6, port 80-82)".to_string(),
            "  UDP80-82 (protocol 17, port 80-82)".to_string(),
            "HTTP2 (protocol 6, port 81-82)".to_string(),
            "UDP81-82 (protocol 17, port 81-82)".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let optimized = optimize_l4_items(port_lists);
        assert_eq!(optimized.len(), 2);
    }

    #[test]
    fn test_optimize_l4_items_shadow_3() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP2 (protocol 6, port 81-82)".to_string(),
            "  UDP81-82 (protocol 17, port 81-82)".to_string(),
            "TCP (protocol 6)".to_string(),
            "UDP (protocol 17)".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let optimized = optimize_l4_items(port_lists);
        assert_eq!(optimized.len(), 2);
    }

    #[test]
    fn test_optimize_l4_items_partial_overlap_1() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP (protocol 6, port 80-82)".to_string(),
            "HTTP2 (protocol 6, port 81-85)".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let optimized = optimize_l4_items(port_lists);
        assert_eq!(optimized.len(), 1);
    }

    #[test]
    fn test_optimize_l4_items_partial_overlap_2() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP (protocol 6, port 80-82)".to_string(),
            "  UDP (protocol 17, port 80-82)".to_string(),
            "HTTP2 (protocol 6, port 81-85)".to_string(),
            "UDP2 (protocol 17, port 81-85)".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let optimized = optimize_l4_items(port_lists);
        assert_eq!(optimized.len(), 2);
    }

    #[test]
    fn test_optimize_l4_items_merge() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP (protocol 6, port 80-82)".to_string(),
            "HTTP2 (protocol 6, port 83-85)".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let optimized = optimize_l4_items(port_lists);
        assert_eq!(optimized.len(), 1);
    }

    #[test]
    fn test_optimize_l4_items_merge_2() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP80 (protocol 6, port 80-80)".to_string(),
            "HTTP82 (protocol 6, port 82-82)".to_string(),
            "HTTP81 (protocol 6, port 81-81)".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let optimized = optimize_l4_items(port_lists);
        assert_eq!(optimized.len(), 1);
    }

    #[test]
    fn test_optimize_l4_items_merge_3() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP80 (protocol 6,  port 80-80)".to_string(),
            "  UDP80 (protocol 17, port 80-80)".to_string(),
            "HTTP82 (protocol 6, port 82-82)".to_string(),
            "UDP82 (protocol 17, port 82-82)".to_string(),
            "HTTP81 (protocol 6, port 81-81)".to_string(),
            "UDP81 (protocol 17, port 81-81)".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let optimized = optimize_l4_items(port_lists);
        assert_eq!(optimized.len(), 2);
    }

    #[test]
    fn test_optimize_l4_items_empty() {
        let lines = vec!["Destination Ports     : MyGroup1 (group)".to_string()];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let optimized = optimize_l4_items(port_lists);
        assert_eq!(optimized.len(), 0);
    }

    #[test]
    fn test_optimize_l4_items_length_1() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  GRE (protocol 47)".to_string(),
            "  HTTP (protocol 6, port 80-82)".to_string(),
            "  UDP (protocol 17, port 80-82)".to_string(),
            "HTTP2 (protocol 6, port 81-85)".to_string(),
            "UDP2 (protocol 17, port 81-85)".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let optimized = optimize_l4_items(port_lists);
        assert_eq!(optimized.len(), 3);
    }

    #[test]
    fn test_optimize_l4_items_length_2() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  GRE (protocol 47)".to_string(),
            "  HTTP (protocol 6, port 80-82)".to_string(),
            "  UDP (protocol 17, port 80-82)".to_string(),
            "HTTP2 (protocol 6, port 81-85)".to_string(),
            "AH (protocol 51)".to_string(),
            "ESP (protocol 50)".to_string(),
            "UDP2 (protocol 17, port 81-85)".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let optimized = optimize_l4_items(port_lists);
        assert_eq!(optimized.len(), 5);
    }

    #[test]
    fn test_optimize_l4_items_length_3() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  GRE (protocol 47)".to_string(),
            "  HTTP (protocol 6, port 80-81)".to_string(),
            "  UDP (protocol 17, port 80-82)".to_string(),
            "HTTP2 (protocol 6, port 81-85)".to_string(),
            "AH (protocol 51)".to_string(),
            "ESP (protocol 50)".to_string(),
            "UDP2 (protocol 17, port 81-85)".to_string(),
            "HTTP3 (protocol 6, port 81-82)".to_string(),
            "UDP3 (protocol 17, port 86-87)".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let optimized = optimize_l4_items(port_lists);
        assert_eq!(optimized.len(), 5);
    }

    #[test]
    fn test_optimize_l4_items_length_4() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  GRE (protocol 47)".to_string(),
            "  HTTP (protocol 6, port 80-81)".to_string(),
            "  UDP (protocol 17, port 80-82)".to_string(),
            "HTTP2 (protocol 6, port 81-85)".to_string(),
            "AH_1 (protocol 51)".to_string(),
            "ESP (protocol 50)".to_string(),
            "UDP2 (protocol 17, port 81-85)".to_string(),
            "HTTP3 (protocol 6, port 81-82)".to_string(),
            "AH_2 (protocol 51)".to_string(),
            "UDP3 (protocol 17, port 86-87)".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let optimized = optimize_l4_items(port_lists);
        assert_eq!(optimized.len(), 5);
    }

    #[test]
    fn test_optimize_l4_items_length_5() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP2 (protocol 6, port 82)".to_string(),
            "HTTP1 (protocol 6, port 81)".to_string(),
            "HTTP (protocol 6, port 80)".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let optimized = optimize_l4_items(port_lists);
        assert_eq!(optimized.len(), 1);
    }

    #[test]
    fn test_optimize_l4_items_length_6() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP2 (protocol 6, port 82)".to_string(),
            "  AH (protocol 51)".to_string(),
            "HTTP1 (protocol 6, port 81)".to_string(),
            "HTTP (protocol 6, port 80)".to_string(),
        ];
        let port_object = ProtocolObject::try_from(&lines).unwrap();
        let port_lists: Vec<&ProtocolList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let optimized = optimize_l4_items(port_lists);
        assert_eq!(optimized.len(), 2);
    }
}
