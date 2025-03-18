use super::group::port_list::PortList;

/// Vector of PortObjectItem returned after optimization  
/// name - all opertations performed on items  
/// items - the list of PortList objects  
/// PortList objects are flattened from the Group objects and normal PortList objects
#[derive(Debug)]
pub struct PortObjectOptimized {
    name: String,
    items: Vec<PortList>,
}

impl PortObjectOptimized {
    pub fn from(port_list: &PortList) -> Self {
        PortObjectOptimized {
            name: port_list.get_name().to_string(),
            items: vec![port_list.clone()],
        }
    }

    pub fn append(&mut self, port_list: &PortList) {
        self.items.push(port_list.clone());
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}
