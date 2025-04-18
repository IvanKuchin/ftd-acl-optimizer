use super::prefix_list_item_optimized::PrefixListItemOptimized;

#[derive(Debug)]
pub struct NetworkObjectOptimized {
    name: String,
    items: Vec<PrefixListItemOptimized>,
}

pub struct Builder {
    name: Option<String>,
    items: Vec<PrefixListItemOptimized>,
}

impl Builder {
    pub fn new(items: Vec<PrefixListItemOptimized>) -> Self {
        Builder { name: None, items }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn build(self) -> NetworkObjectOptimized {
        NetworkObjectOptimized {
            name: self.name.unwrap_or_default(),
            items: self.items,
        }
    }
}

impl NetworkObjectOptimized {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn items(&self) -> &[PrefixListItemOptimized] {
        &self.items
    }

    pub fn capacity(&self) -> u64 {
        self.items.iter().map(|item| item.capacity()).sum()
    }
}
