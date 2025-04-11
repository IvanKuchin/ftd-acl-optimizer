use super::{
    group::prefix_list::prefix_list_item::PrefixListItem, network_object_item::NetworkObjectItem,
};

use crate::rules::rule::network_object::group::prefix_list::prefix_list_item::ip_range::IPRange;

pub struct PrefixListItemOptimized {
    name: String,
    items: Vec<PrefixListItem>,
}

impl From<&PrefixListItem> for PrefixListItemOptimized {
    fn from(item: &PrefixListItem) -> Self {
        PrefixListItemOptimized {
            name: item.get_name().to_string(),
            items: vec![item.clone()],
        }
    }
}

impl PrefixListItemOptimized {
    pub fn append(&mut self, network_object: &PrefixListItem) {
        self.items.push(network_object.clone());
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn capacity(&self) -> u64 {
        let start_ip = self.items.iter().map(|item| item.start_ip()).min().unwrap_or_else(|| panic!("Logic error: PrefixListItemOptimized ({}) should have at least one PrefixListItem, if this error is triggered, parsing logic must be fixed. Currentlt the only way to craft obj is from-trait which accepts correct object", self.name));
        let end_ip = self.items.iter().map(|item| item.end_ip()).max().unwrap_or_else(|| panic!("Logic error: PrefixListItemOptimized ({}) should have at least one PrefixListItem, if this error is triggered, parsing logic must be fixed. Currentlt the only way to craft obj is from-trait which accepts correct object", self.name));

        let ip_range = IPRange::new(self.name.clone(), start_ip.clone(), end_ip.clone());

        ip_range.capacity()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_from_trait() {
        let prefix_list_item = PrefixListItem::from_str("192.168.0.1/24").unwrap();
        let optimized_item: PrefixListItemOptimized = (&prefix_list_item).into();

        assert_eq!(optimized_item.name, "192.168.0.1/24");
        assert_eq!(optimized_item.items.len(), 1);
    }

    #[test]
    fn test_append() {
        let prefix_list_item1 = PrefixListItem::from_str("192.168.0.0/24").unwrap();
        let prefix_list_item2 = PrefixListItem::from_str("192.168.1.0/24").unwrap();

        let mut optimized_item: PrefixListItemOptimized = (&prefix_list_item1).into();
        optimized_item.append(&prefix_list_item2);

        assert_eq!(optimized_item.items.len(), 2);
    }

    #[test]
    fn test_set_name() {
        let prefix_list_item = PrefixListItem::from_str("192.168.0.1/32").unwrap();
        let mut optimized_item: PrefixListItemOptimized = (&prefix_list_item).into();

        optimized_item.set_name("new_name".to_string());
        assert_eq!(optimized_item.name, "new_name");
    }

    #[test]
    fn test_capacity() {
        let prefix_list_item = PrefixListItem::from_str("192.168.0.1-192.168.0.255").unwrap();
        let optimized_item: PrefixListItemOptimized = (&prefix_list_item).into();

        let capacity = optimized_item.capacity();
        assert_eq!(capacity, 8); // Assuming IPRange::capacity calculates the range correctly
    }

    #[test]
    #[should_panic(expected = "Logic error: PrefixListItemOptimized")]
    fn test_capacity_panic_on_empty_items() {
        let optimized_item = PrefixListItemOptimized {
            name: "empty".to_string(),
            items: vec![],
        };

        optimized_item.capacity(); // This should panic
    }

    #[test]
    fn test_capacity_marge_1() {
        let prefix_list_items = [
            PrefixListItem::from_str("192.168.0.0/24").unwrap(),
            PrefixListItem::from_str("192.168.1.0/24").unwrap(),
        ];

        let mut optimized_item: PrefixListItemOptimized = (&prefix_list_items[0]).into();

        prefix_list_items.iter().skip(1).for_each(|item| {
            optimized_item.append(item);
        });

        assert_eq!(optimized_item.capacity(), 1);
    }

    #[test]
    fn test_capacity_shadow_1() {
        let prefix_list_items = [
            PrefixListItem::from_str("192.168.0.0/24").unwrap(),
            PrefixListItem::from_str("192.168.0.64/26").unwrap(),
        ];

        let mut optimized_item: PrefixListItemOptimized = (&prefix_list_items[0]).into();

        prefix_list_items.iter().skip(1).for_each(|item| {
            optimized_item.append(item);
        });

        assert_eq!(optimized_item.capacity(), 1);
    }

    #[test]
    fn test_capacity_overlap_1() {
        let prefix_list_items = [
            PrefixListItem::from_str("192.168.0.0-192.168.0.192").unwrap(),
            PrefixListItem::from_str("192.168.0.64-192.168.0.255").unwrap(),
        ];

        let mut optimized_item: PrefixListItemOptimized = (&prefix_list_items[0]).into();

        prefix_list_items.iter().skip(1).for_each(|item| {
            optimized_item.append(item);
        });

        assert_eq!(optimized_item.capacity(), 1);
    }
}
