use super::prefix_list_item_optimized::PrefixListItemOptimized;

#[derive(Debug)]
pub struct NetworkObjectOptimized {
    pub name: String,
    pub items: Vec<PrefixListItemOptimized>,
}

pub struct Builder {
    pub name: Option<String>,
    pub items: Vec<PrefixListItemOptimized>,
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
    pub fn capacity(&self) -> u64 {
        self.items.iter().map(|item| item.capacity()).sum()
    }
}
