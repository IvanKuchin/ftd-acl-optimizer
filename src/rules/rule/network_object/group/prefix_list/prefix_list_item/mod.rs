use std::str::FromStr;

mod prefix;
use prefix::Prefix;

mod ip_range;
use ip_range::IPRange;

mod ipv4;

#[derive(Debug)]
pub enum PrefixListItem {
    Prefix(Prefix),
    IPRange(IPRange),
}

#[derive(thiserror::Error, Debug)]
pub enum PrefixListItemError {
    #[error("Failed to parse prefix list item: {0}")]
    General(String),
    
    #[error("Failed to parse prefix list item: {0}")]
    IPRangeError(#[from] ip_range::IPRangeError),

    #[error("Failed to parse prefix list item: {0}")]
    PrefixError(#[from] prefix::PrefixError),
}

impl FromStr for PrefixListItem {
    type Err = PrefixListItemError;

    // Example line:
    // 10.0.0.0/8
    // or
    // 10.11.12.13-10.11.12.18
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        if line.contains("-") {
            let ip_range = line.parse::<IPRange>()?;
            Ok(PrefixListItem::IPRange(ip_range))
        } else {
            let prefix = line.parse::<Prefix>()?;
            Ok(PrefixListItem::Prefix(prefix))
        }
    }
}

