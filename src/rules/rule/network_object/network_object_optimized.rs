use super::{
    group::prefix_list::prefix_list_item::PrefixListItem, network_object_item::NetworkObjectItem,
};

pub struct PrefixListItemOptimized {
    name: String,
    items: Vec<PrefixListItem>,
}

impl PrefixListItemOptimized {
    pub fn from(network_object: &PrefixListItem) -> Self {
        PrefixListItemOptimized {
            name: network_object.get_name().to_string(),
            items: vec![network_object.clone()],
        }
    }

    pub fn append(&mut self, network_object: &PrefixListItem) {
        self.items.push(network_object.clone());
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}
