use super::*;
use group::Group;

/// PortObjectItem is either a PortList or a Group
#[derive(Debug)]
pub enum ProtocolObjectItem {
    ProtocolList(ProtocolList),
    Group(Group),
}

impl ProtocolObjectItem {
    /// Builds a flattened list of PortList objects from Groups and PortLists
    pub fn collect_objects(&self) -> Vec<&ProtocolList> {
        let protocol_lists: Vec<&ProtocolList> = match self {
            ProtocolObjectItem::ProtocolList(port_list) => vec![port_list],
            ProtocolObjectItem::Group(group) => group.port_lists.iter().collect(),
        };

        protocol_lists
    }
}
