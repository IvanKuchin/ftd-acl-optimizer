mod group;
use group::port_list;
use group::port_list::PortList;
use group::Group;

#[derive(Debug)]
pub struct PortObject {
    name: String,
    items: Vec<PortObjectItem>,
}

#[derive(Debug)]
pub enum PortObjectItem {
    PortList(PortList),
}

#[derive(thiserror::Error, Debug)]
pub enum PortObjectError {
    #[error("Failed to parse port object: {0}")]
    General(String),
    #[error("Failed to parse port object: {0}")]
    PortListError(#[from] port_list::PortListError),
}

impl TryFrom<&Vec<String>> for PortObject {
    type Error = PortObjectError;

    fn try_from(lines: &Vec<String>) -> Result<Self, Self::Error> {
        todo!("Implement PortObject::try_from");

        let name = get_name(lines)?;

        // it doesn't NOT work, refactoring required
        let port_list: Vec<_> = lines
            .iter()
            .map(|line| line.parse::<PortList>())
            .map(|result| result.map(PortObjectItem::PortList))
            .collect::<Result<_, port_list::PortListError>>()?;

        Ok(Self {
            name,
            items: port_list,
        })
    }
}

fn get_name(lines: &[String]) -> Result<String, PortObjectError> {
    Ok("".to_string())
}
