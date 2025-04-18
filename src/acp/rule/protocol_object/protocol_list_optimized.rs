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

    pub fn get_name(&self) -> &str {
        &self.name
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

    pub fn get_ports(&self) -> (u16, u16) {
        let start = self
            .items
            .iter()
            .map(|port_list| port_list.get_ports().0)
            .min();

        let end = self
            .items
            .iter()
            .map(|port_list| port_list.get_ports().1)
            .max();

        (start.unwrap_or(0), end.unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn from() {
        let protocol_list = ProtocolList::from_str("HTTP (protocol 6, port 80)").unwrap();
        let optimized = ProtocolListOptimized::from(&protocol_list);

        assert_eq!(optimized.name, "HTTP");
        assert_eq!(optimized.items.len(), 1);
        assert_eq!(optimized.items[0], protocol_list);
    }

    #[test]
    fn append() {
        let protocol_list1 = ProtocolList::from_str("HTTP (protocol 6, port 80)").unwrap();
        let protocol_list2 = ProtocolList::from_str("HTTP (protocol 6, port 443)").unwrap();
        let mut optimized = ProtocolListOptimized::from(&protocol_list1);

        optimized.append(&protocol_list2);

        assert_eq!(optimized.items.len(), 2);
        assert_eq!(optimized.items[1], protocol_list2);
    }

    #[test]
    fn set_name() {
        let protocol_list = ProtocolList::from_str("HTTP (protocol 6, port 80)").unwrap();
        let mut optimized = ProtocolListOptimized::from(&protocol_list);

        optimized.set_name("NewName".to_string());

        assert_eq!(optimized.name, "NewName");
    }

    #[test]
    fn get_protocol() {
        let protocol_list = ProtocolList::from_str("HTTP (protocol 6, port 80)").unwrap();
        let optimized = ProtocolListOptimized::from(&protocol_list);

        assert_eq!(optimized.get_protocol(), 6);
    }

    #[test]
    fn get_ports_1() {
        let protocol_list1 = ProtocolList::from_str("Test1 (protocol 6, port 80-100)").unwrap();
        let protocol_list2 = ProtocolList::from_str("Test2 (protocol 6, port 50-150)").unwrap();
        let mut optimized = ProtocolListOptimized::from(&protocol_list1);

        optimized.append(&protocol_list2);

        let (start, end) = optimized.get_ports();
        assert_eq!(start, 50);
        assert_eq!(end, 150);
    }

    #[test]
    fn get_ports_2() {
        let protocol_list1 = ProtocolList::from_str("Test1 (protocol 6, port 80-110)").unwrap();
        let protocol_list2 = ProtocolList::from_str("Test2 (protocol 6, port 90-150)").unwrap();
        let mut optimized = ProtocolListOptimized::from(&protocol_list1);

        optimized.append(&protocol_list2);

        let (start, end) = optimized.get_ports();
        assert_eq!(start, 80);
        assert_eq!(end, 150);
    }

    #[test]
    fn get_ports_3() {
        let protocol_list1 = ProtocolList::from_str("Test2 (protocol 6, port 90-150)").unwrap();
        let protocol_list2 = ProtocolList::from_str("Test1 (protocol 6, port 80-110)").unwrap();
        let mut optimized = ProtocolListOptimized::from(&protocol_list1);

        optimized.append(&protocol_list2);

        let (start, end) = optimized.get_ports();
        assert_eq!(start, 80);
        assert_eq!(end, 150);
    }

    #[test]
    fn get_ports_4() {
        let protocol_list1 = ProtocolList::from_str("HTTP (protocol 6, port 80)").unwrap();
        let protocol_list2 = ProtocolList::from_str("HTTPS (protocol 6, port 443)").unwrap();
        let mut optimized = ProtocolListOptimized::from(&protocol_list1);

        optimized.append(&protocol_list2);

        let (start, end) = optimized.get_ports();
        assert_eq!(start, 80);
        assert_eq!(end, 443);
    }

    #[test]
    fn get_ports_5() {
        let protocol_list1 = ProtocolList::from_str("HTTPS (protocol 6, port 443)").unwrap();
        let protocol_list2 = ProtocolList::from_str("HTTP (protocol 6, port 80)").unwrap();
        let mut optimized = ProtocolListOptimized::from(&protocol_list1);

        optimized.append(&protocol_list2);

        let (start, end) = optimized.get_ports();
        assert_eq!(start, 80);
        assert_eq!(end, 443);
    }

    #[test]
    fn get_ports_6() {
        let protocol_list1 = ProtocolList::from_str("HTTPS (protocol 6, port 443-8443)").unwrap();
        let optimized = ProtocolListOptimized::from(&protocol_list1);

        let (start, end) = optimized.get_ports();
        assert_eq!(start, 443);
        assert_eq!(end, 8443);
    }
}
