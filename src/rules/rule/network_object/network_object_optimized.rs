use super::network_object_item::NetworkObjectItem;

pub struct NetworkObjectOptimized {
    name: String,
    items: Vec<NetworkObjectItem>,
}

impl NetworkObjectOptimized {
    pub fn from(network_object: &NetworkObjectItem) -> Self {
        NetworkObjectOptimized {
            name: network_object.get_name().to_string(),
            items: vec![network_object.clone()],
        }
    }

    pub fn append(&mut self, network_object: &NetworkObjectItem) {
        self.items.push(network_object.clone());
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}
