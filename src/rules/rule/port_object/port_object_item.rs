use super::*;
use group::Group;

/// PortObjectItem is either a PortList or a Group
#[derive(Debug)]
pub enum PortObjectItem {
    PortList(PortList),
    Group(Group),
}

impl PortObjectItem {
    /// Builds a flattened list of PortList objects from Groups and PortLists
    pub fn collect_objects(&self) -> Vec<&PortList> {
        let port_lists: Vec<&PortList> = match self {
            PortObjectItem::PortList(port_list) => vec![port_list],
            PortObjectItem::Group(group) => group.port_lists.iter().collect(),
        };

        port_lists
    }
}
