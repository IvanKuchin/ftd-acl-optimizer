use super::group::protocol_list::ProtocolList;

/// Vector of PortObjectItem returned after optimization  
/// name - description of all operations performed on items  
/// items - the list of PortList objects  
/// PortList objects are flattened from the Group objects and normal PortList objects
#[derive(Debug)]
pub struct ProtocolListOptimized {
    name: String,
    items: Vec<ProtocolList>,
}

impl ProtocolListOptimized {
    pub fn from(port_list: &ProtocolList) -> Self {
        ProtocolListOptimized {
            name: port_list.get_name().to_string(),
            items: vec![port_list.clone()],
        }
    }

    pub fn append(&mut self, port_list: &ProtocolList) {
        self.items.push(port_list.clone());
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_protocol(&self) -> u8 {
        self.items
            .first()
            .map(|port_list| port_list.get_protocol())
            .unwrap_or_else(|| panic!
                (
                    "Logic error: PortObjectOptimized ({}) should have at least one PortList, if this error is triggered, parsing logic must be fixed. The only way to craft PortObjectOptimized is from-method, which creates non-empty Vec<PortList>",
                    self.name
                )
            )
    }
}
