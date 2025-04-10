use super::{
    group::prefix_list::prefix_list_item::PrefixListItem, network_object_item::NetworkObjectItem,
};

pub struct PrefixListItemOptimized {
    name: String,
    items: Vec<PrefixListItem>,
}

impl PrefixListItemOptimized {
    pub fn from(prefix_list_item: &PrefixListItem) -> Self {
        PrefixListItemOptimized {
            name: prefix_list_item.get_name().to_string(),
            items: vec![prefix_list_item.clone()],
        }
    }

    pub fn append(&mut self, network_object: &PrefixListItem) {
        self.items.push(network_object.clone());
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}
