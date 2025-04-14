use std::fmt::Display;
use std::fmt::Formatter;

pub enum DescriptionType {
    Adjoins,
    Shadows,
    PartiallyOverlaps,
}

impl Display for DescriptionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DescriptionType::Adjoins => write!(f, "ADJOINS"),
            DescriptionType::Shadows => write!(f, "SHADOWS"),
            DescriptionType::PartiallyOverlaps => write!(f, "PARTIALLY OVERLAPS"),
        }
    }
}

pub fn verb(curr_end: u32, next_start: u32, next_end: u32) -> DescriptionType {
    if curr_end as u64 + 1 == next_start as u64 {
        DescriptionType::Adjoins
    } else if next_end <= curr_end {
        DescriptionType::Shadows
    } else {
        DescriptionType::PartiallyOverlaps
    }
}
