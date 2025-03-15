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

/// Get the next object from input lines (either Group or PortList) and the number of lines to consume.
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

impl PortObject {
    // used strictly for testing
    // capacity calculation does not work on a port object level, it should be done on a rule level
    fn _capacity(&self) -> u64 {
        let port_lists: Vec<&PortList> = self
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let unmergeable_items: Vec<&PortList> = port_lists
            .iter()
            .filter(|port_list| !port_list.is_mergable())
            .copied()
            .collect();
        let unique_unmergeable_items = unique_unmergeable_items(unmergeable_items);

        let mergable_items: Vec<&PortList> = port_lists
            .iter()
            .filter(|port_list| port_list.is_mergable())
            .copied()
            .collect();
        let merged_ranges = merge_and_count_items(mergable_items);

        unique_unmergeable_items.len() as u64 + merged_ranges.len() as u64
    }
}

fn unique_unmergeable_items(port_lists: Vec<&PortList>) -> Vec<&PortList> {
    let unique_items = port_lists
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .copied()
        .collect();

    unique_items
}

fn merge_and_count_items(port_lists: Vec<&PortList>) -> Vec<(u32, u32)> {
    let port_ranges = port_lists
        .iter()
        .map(|item| {
            let proto = item.get_protocol();
            let (start, end) = item.get_ports();
            (
                ((proto as u32) << 16) + start as u32,
                ((proto as u32) << 16) + end as u32,
            )
        })
        .collect::<Vec<_>>();

    merge_ranges(port_ranges)
}

/// Merge overlapping port ranges.
/// The input is a list of tuples where each tuple represents a port range.
/// The output is a list of tuples where each tuple represents a merged port range.
/// Example:
///  [(80, 80), (443, 443), (80, 81), (33434, 33434)]
/// -> [(80, 81), (443, 443), (33434, 33434)]
fn merge_ranges(port_ranges: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
    use std::cmp::max;
    if port_ranges.is_empty() {
        return vec![];
    }

    let mut port_ranges = port_ranges;
    port_ranges.sort_unstable();
    let mut merged_ranges = vec![];
    let mut current_range = port_ranges[0];
    for range in port_ranges.iter().skip(1) {
        if range.0 <= current_range.1 + 1 {
            current_range = (current_range.0, max(current_range.1, range.1));
        } else {
            merged_ranges.push(current_range);
            current_range = *range;
        }
    }
    merged_ranges.push(current_range);

    merged_ranges
}

impl PortObjectItem {
    pub fn collect_objects(&self) -> Vec<&PortList> {
        let port_lists: Vec<&PortList> = match self {
            PortObjectItem::PortList(port_list) => vec![port_list],
            PortObjectItem::Group(group) => group.port_lists.iter().collect(),
        };

        port_lists
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

    #[test]
    fn test_port_object_capacity_single_port_list() {
        let lines = vec!["Destination Ports     : TCP-8080 (protocol 6, port 8080)".to_string()];
        let port_object = PortObject::try_from(&lines).unwrap();
        assert_eq!(port_object._capacity(), 1); // Single port
    }

    #[test]
    fn test_port_object_capacity_multiple_port_lists() {
        let lines = vec![
            "Destination Ports     : HTTP-HTTPS_1 (group)".to_string(),
            "  HTTP (protocol 6, port 80)".to_string(),
            "  HTTPS (protocol 6, port 443)".to_string(),
            "TCP-8080 (protocol 6, port 8080)".to_string(),
        ];
        let port_object = PortObject::try_from(&lines).unwrap();
        assert_eq!(port_object._capacity(), 3); // Three ports
    }

    #[test]
    fn test_port_object_capacity_port_range() {
        let lines = vec![
            "Destination Ports     : HTTP-HTTPS_1 (group)".to_string(),
            "  HTTP (protocol 6, port 80-81)".to_string(),
        ];
        let port_object = PortObject::try_from(&lines).unwrap();
        assert_eq!(port_object._capacity(), 1); // Port range 80-81
    }

    #[test]
    fn test_port_object_capacity_empty() {
        let lines = vec!["Destination Ports     : HTTP-HTTPS_1 (group)".to_string()];
        let port_object = PortObject::try_from(&lines).unwrap();
        assert_eq!(port_object._capacity(), 0); // No ports
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
        let port_object = PortObject::try_from(&lines).unwrap();
        assert_eq!(port_object._capacity(), 4); // 1 port + 3 ports in range + 1 port + 1 port
    }

    #[test]
    fn test_port_object_unique_unmergeable_items_1() {
        let lines = vec![
            "Destination Ports     : HTTP-HTTPS_1 (group)".to_string(),
            "  IGMP (protocol 2)".to_string(),
            "  GGMP (protocol 3)".to_string(),
            "EIGRP (protocol 88)".to_string(),
            "ESP (protocol 50)".to_string(),
            "AH (protocol 51)".to_string(),
            "protocol 10".to_string(),
        ];
        let port_object = PortObject::try_from(&lines).unwrap();
        let port_lists: Vec<&PortList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let unmergeable_items = unique_unmergeable_items(port_lists);
        assert_eq!(unmergeable_items.len(), 6);
    }

    #[test]
    fn test_port_object_unique_unmergeable_items_duplicate() {
        let lines = vec![
            "Destination Ports     : HTTP-HTTPS_2 (group)".to_string(),
            "  BGP (protocol 17)".to_string(),
            "  RIP (protocol 9)".to_string(),
            "LDP (protocol 39)".to_string(),
            "LDP (protocol 39)".to_string(),
            "PIM (protocol 103)".to_string(),
            "protocol 6".to_string(),
        ];
        let port_object = PortObject::try_from(&lines).unwrap();
        let port_lists: Vec<&PortList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let unmergeable_items = unique_unmergeable_items(port_lists);
        assert_eq!(unmergeable_items.len(), 5);
    }

    #[test]
    fn test_port_object_unique_unmergeable_items_duplicates_2() {
        let lines = vec![
            "Destination Ports     : SomeProtocols (group)".to_string(),
            "  BGP (protocol 17)".to_string(),
            "  RIP (protocol 9)".to_string(),
            "LDP (protocol 39)".to_string(),
            "ldp (protocol 39)".to_string(),
            "PIM (protocol 103)".to_string(),
            "protocol 6".to_string(),
        ];
        let port_object = PortObject::try_from(&lines).unwrap();
        let port_lists: Vec<&PortList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let unmergeable_items = unique_unmergeable_items(port_lists);
        assert_eq!(unmergeable_items.len(), 5);
    }

    #[test]
    fn test_port_object_unique_unmergeable_items_duplicates_3() {
        let lines = vec![
            "Destination Ports     : ICMP (group)".to_string(),
            "  ICMP1 (protocol 1, type 4, code 11)".to_string(),
            "ICMP1 (protocol 1, type 4, code 12)".to_string(),
        ];
        let port_object = PortObject::try_from(&lines).unwrap();
        let port_lists: Vec<&PortList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let unmergeable_items = unique_unmergeable_items(port_lists);
        assert_eq!(unmergeable_items.len(), 2);
    }

    #[test]
    fn test_port_object_unique_unmergeable_items_duplicates_4() {
        let lines = vec![
            "Destination Ports     : ICMP (group)".to_string(),
            "  ICMP1 (protocol 1, type 4, code 11)".to_string(),
            "ICMP2 (protocol 1, type 4)".to_string(),
            "ICMP3 (protocol 1, type 4, code 12)".to_string(),
        ];
        let port_object = PortObject::try_from(&lines).unwrap();
        let port_lists: Vec<&PortList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let unmergeable_items = unique_unmergeable_items(port_lists);
        assert_eq!(unmergeable_items.len(), 3);
    }

    #[test]
    fn test_port_object_unique_unmergeable_items_duplicates_in_group() {
        let lines = vec![
            "Destination Ports     : HTTP-HTTPS_2 (group)".to_string(),
            "  BGP (protocol 17)".to_string(),
            "  RIP (protocol 9)".to_string(),
            "  LDP (protocol 39)".to_string(),
            "  ldp (protocol 39)".to_string(),
            "PIM (protocol 103)".to_string(),
            "protocol 6".to_string(),
        ];
        let port_object = PortObject::try_from(&lines).unwrap();
        let port_lists: Vec<&PortList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let unmergeable_items = unique_unmergeable_items(port_lists);
        assert_eq!(unmergeable_items.len(), 5);
    }

    #[test]
    fn test_port_object_unique_unmergeable_items_duplicates_cross_groups() {
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
        let port_object = PortObject::try_from(&lines).unwrap();
        let port_lists: Vec<&PortList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let unmergeable_items = unique_unmergeable_items(port_lists);
        assert_eq!(unmergeable_items.len(), 5);
    }

    #[test]
    fn test_port_object_unique_unmergeable_items_duplicates_cross_groups_2() {
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
        let port_object = PortObject::try_from(&lines).unwrap();
        let port_lists: Vec<&PortList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let unmergeable_items = unique_unmergeable_items(port_lists);
        assert_eq!(unmergeable_items.len(), 5);
    }

    #[test]
    fn test_port_object_unique_unmergeable_items_duplicates_cross_groups_wo_name() {
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
        let port_object = PortObject::try_from(&lines).unwrap();
        let port_lists: Vec<&PortList> = port_object
            .items
            .iter()
            .flat_map(|item| item.collect_objects())
            .collect();

        let unmergeable_items = unique_unmergeable_items(port_lists);
        assert_eq!(unmergeable_items.len(), 5);
    }

    #[test]
    fn test_port_object_capacity_unmergeable_items_1() {
        let lines = vec![
            "Destination Ports     : HTTP-HTTPS_1 (group)".to_string(),
            "  IGMP (protocol 2)".to_string(),
            "  GGMP (protocol 3)".to_string(),
            "EIGRP (protocol 88)".to_string(),
            "ESP (protocol 50)".to_string(),
            "AH (protocol 51)".to_string(),
            "protocol 10".to_string(),
        ];
        assert_eq!(PortObject::try_from(&lines).unwrap()._capacity(), 6);
    }

    #[test]
    fn test_port_object_capacity_unmergeable_items_duplicate() {
        let lines = vec![
            "Destination Ports     : HTTP-HTTPS_2 (group)".to_string(),
            "  BGP (protocol 17)".to_string(),
            "  RIP (protocol 9)".to_string(),
            "LDP (protocol 39)".to_string(),
            "LDP (protocol 39)".to_string(),
            "PIM (protocol 103)".to_string(),
            "protocol 6".to_string(),
        ];
        assert_eq!(PortObject::try_from(&lines).unwrap()._capacity(), 5);
    }

    #[test]
    fn test_port_object_capacity_unmergeable_items_duplicates_2() {
        let lines = vec![
            "Destination Ports     : HTTP-HTTPS_2 (group)".to_string(),
            "  BGP (protocol 17)".to_string(),
            "  RIP (protocol 9)".to_string(),
            "LDP (protocol 39)".to_string(),
            "ldp (protocol 39)".to_string(),
            "PIM (protocol 103)".to_string(),
            "protocol 6".to_string(),
        ];
        assert_eq!(PortObject::try_from(&lines).unwrap()._capacity(), 5);
    }

    #[test]
    fn test_port_object_capacity_unmergeable_items_duplicates_in_group() {
        let lines = vec![
            "Destination Ports     : HTTP-HTTPS_2 (group)".to_string(),
            "  BGP (protocol 17)".to_string(),
            "  RIP (protocol 9)".to_string(),
            "  LDP (protocol 39)".to_string(),
            "  ldp (protocol 39)".to_string(),
            "PIM (protocol 103)".to_string(),
            "protocol 6".to_string(),
        ];
        assert_eq!(PortObject::try_from(&lines).unwrap()._capacity(), 5);
    }

    #[test]
    fn test_port_object_capacity_unmergeable_items_duplicates_cross_groups() {
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
        assert_eq!(PortObject::try_from(&lines).unwrap()._capacity(), 5);
    }

    #[test]
    fn test_port_object_capacity_unmergeable_items_duplicates_cross_groups_2() {
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
        assert_eq!(PortObject::try_from(&lines).unwrap()._capacity(), 5);
    }

    #[test]
    fn test_port_object_capacity_unmergeable_items_duplicates_cross_groups_wo_name() {
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
        assert_eq!(PortObject::try_from(&lines).unwrap()._capacity(), 5);
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
        assert_eq!(PortObject::try_from(&lines).unwrap()._capacity(), 4);
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
        assert_eq!(PortObject::try_from(&lines).unwrap()._capacity(), 4);
    }

    #[test]
    fn test_port_object_merge_ranges() {
        let port_ranges = vec![(80, 80), (443, 443), (80, 81), (33434, 33434)];

        let merged_ranges = merge_ranges(port_ranges);

        assert_eq!(merged_ranges, vec![(80, 81), (443, 443), (33434, 33434)]);
    }

    #[test]
    fn test_port_object_merge_ranges_2() {
        let port_ranges = vec![(88, 89), (443, 443), (80, 90), (33434, 33434)];

        let merged_ranges = merge_ranges(port_ranges);

        assert_eq!(merged_ranges, vec![(80, 90), (443, 443), (33434, 33434)]);
    }

    #[test]
    fn test_port_object_merge_ranges_3() {
        let port_ranges = vec![(88, 88), (443, 443), (87, 87), (1, 2), (33434, 33434)];

        let merged_ranges = merge_ranges(port_ranges);

        assert_eq!(
            merged_ranges,
            vec![(1, 2), (87, 88), (443, 443), (33434, 33434)]
        );
    }

    #[test]
    fn test_port_object_merge_ranges_4() {
        let port_ranges = vec![(80, 90), (443, 443), (85, 95), (1, 2), (33434, 33434)];

        let merged_ranges = merge_ranges(port_ranges);

        assert_eq!(
            merged_ranges,
            vec![(1, 2), (80, 95), (443, 443), (33434, 33434)]
        );
    }

    #[test]
    fn test_port_object_merge_ranges_5() {
        let port_ranges = vec![
            (80, 80),
            (443, 443),
            (81, 81),
            (1, 2),
            (82, 82),
            (33434, 33434),
        ];

        let merged_ranges = merge_ranges(port_ranges);

        assert_eq!(
            merged_ranges,
            vec![(1, 2), (80, 82), (443, 443), (33434, 33434)]
        );
    }

    #[test]
    fn test_port_object_merge_ranges_empty() {
        let port_ranges = vec![];

        let merged_ranges = merge_ranges(port_ranges);

        assert_eq!(merged_ranges, vec![]);
    }

    #[test]
    fn test_port_object_merge_ranges_single() {
        let port_ranges = vec![(80, 80)];

        let merged_ranges = merge_ranges(port_ranges);

        assert_eq!(merged_ranges, vec![(80, 80)]);
    }

    #[test]
    fn test_port_object_merge_ranges_single_2() {
        let port_ranges = vec![(80, 80), (80, 80), (80, 80)];

        let merged_ranges = merge_ranges(port_ranges);

        assert_eq!(merged_ranges, vec![(80, 80)]);
    }

    #[test]
    fn test_port_object_capacity_mergeable_items_duplicates_1() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP (protocol 6, port 80)".to_string(),
            "  HTTP2 (protocol 6, port 82)".to_string(),
        ];
        assert_eq!(PortObject::try_from(&lines).unwrap()._capacity(), 2);
    }

    #[test]
    fn test_port_object_capacity_mergeable_items_duplicates_2() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP (protocol 6, port 80-81)".to_string(),
            "  HTTP2 (protocol 6, port 82-83)".to_string(),
        ];
        assert_eq!(PortObject::try_from(&lines).unwrap()._capacity(), 1);
    }

    #[test]
    fn test_port_object_capacity_mergeable_items_duplicates_3() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP (protocol 6, port 80-81)".to_string(),
            "HTTP2 (protocol 6, port 82-83)".to_string(),
        ];
        assert_eq!(PortObject::try_from(&lines).unwrap()._capacity(), 1);
    }

    #[test]
    fn test_port_object_capacity_mergeable_items_duplicates_4() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP (protocol 6, port 80-81)".to_string(),
            "HTTP2 (protocol 6, port 82-83)".to_string(),
            "HTTP3 (protocol 6, port 84-87)".to_string(),
        ];
        assert_eq!(PortObject::try_from(&lines).unwrap()._capacity(), 1);
    }

    #[test]
    fn test_port_object_capacity_mergeable_items_overlap_1() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP (protocol 6, port 85-90)".to_string(),
            "SMTP (protocol 6, port 25)".to_string(),
            "HTTP3 (protocol 6, port 80-87)".to_string(),
        ];
        assert_eq!(PortObject::try_from(&lines).unwrap()._capacity(), 2);
    }

    #[test]
    fn test_port_object_capacity_mergeable_items_overlap_2() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP (protocol 6, port 81-82)".to_string(),
            "SMTP (protocol 6, port 25)".to_string(),
            "HTTP2 (protocol 6, port 82-82)".to_string(),
            "POP3 (protocol 6, port 110)".to_string(),
            "HTTP3 (protocol 6, port 80-80)".to_string(),
        ];
        assert_eq!(PortObject::try_from(&lines).unwrap()._capacity(), 3);
    }

    #[test]
    fn test_port_object_capacity_mergeable_items_overlap_3() {
        let lines = vec![
            "Destination Ports     : MyGroup1 (group)".to_string(),
            "  HTTP (protocol 6, port 81-82)".to_string(),
            "SMTP (protocol 6, port 25)".to_string(),
            "HTTP2 (protocol 6, port 82-82)".to_string(),
            "POP3 (protocol 6, port 110)".to_string(),
            "HTTP3 (protocol 6, port 80-80)".to_string(),
            "HTTP4 (protocol 6, port 80-80)".to_string(),
        ];
        assert_eq!(PortObject::try_from(&lines).unwrap()._capacity(), 3);
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
        assert_eq!(PortObject::try_from(&lines).unwrap()._capacity(), 4);
    }
}
