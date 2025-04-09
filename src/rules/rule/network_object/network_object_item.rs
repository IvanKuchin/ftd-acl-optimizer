use super::group::prefix_list::PrefixList;
use super::group::Group;

#[derive(Debug, Clone)]
pub enum NetworkObjectItem {
    ObjectGroup(Group),
    PrefixList(PrefixList),
}

impl NetworkObjectItem {
    pub fn get_name(&self) -> &str {
        match self {
            NetworkObjectItem::ObjectGroup(group) => group.get_name(),
            NetworkObjectItem::PrefixList(prefix_list) => prefix_list.get_name(),
        }
    }

    pub fn capacity(&self) -> u64 {
        match self {
            NetworkObjectItem::ObjectGroup(group) => group.capacity(),
            NetworkObjectItem::PrefixList(prefix_list) => prefix_list.capacity(),
        }
    }

    pub fn collect_objects(&self) -> Vec<&PrefixList> {
        match self {
            NetworkObjectItem::ObjectGroup(group) => group.get_prefix_lists().iter().collect(),
            NetworkObjectItem::PrefixList(prefix_list) => vec![prefix_list],
        }
    }
}
