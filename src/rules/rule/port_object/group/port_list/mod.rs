use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub struct PortList {
    name: String,
    protocol: u8,
    start: u16,
    end: u16,
}

#[derive(thiserror::Error, Debug)]
pub enum PortListError {
    #[error("Failed to parse port list: {0}")]
    General(String),
}

impl fmt::Display for PortList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start == self.end {
            write!(
                f,
                "{} (protocol {}, port {})",
                self.name, self.protocol, self.start
            )
        } else {
            write!(
                f,
                "{} (protocol {}, port {}-{})",
                self.name, self.protocol, self.start, self.end
            )
        }
    }
}

impl FromStr for PortList {
    type Err = PortListError;

    // Example 1
    // protocol 6, port 17444

    // Example 2
    // HTTP (protocol 6, port 80)

    // Example 3
    // HTTP (protocol 6, port 80-81)

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, ports) = get_name_and_ports(s)?;

        let protocol = parse_protocol(ports)?;

        let (start, end) = parse_ports(ports)?;

        Ok(Self {
            name: name.to_string(),
            protocol,
            start,
            end,
        })
    }
}

// Example 1
// protocol 6, port 17444

// Example 2
// HTTP (protocol 6, port 80)

// Example 3
// HTTP (protocol 6, port 80-81)

fn get_name_and_ports(s: &str) -> Result<(&str, &str), PortListError> {
    let mut parts = s.split('(');

    match parts.clone().count() {
        1 => {
            let name = parts.next().unwrap().trim();
            let ports = name;

            if name.contains(')') {
                return Err(PortListError::General(format!(
                    "Missing opening parenthesis in port list: {}",
                    s
                )));
            }

            Ok((name, ports))
        }
        2 => {
            let name = parts.next().unwrap().trim();
            let ports = parts.next().unwrap().trim();

            if let Some(ports) = ports.strip_suffix(')') {
                return Ok((name, ports));
            }
            Err(PortListError::General(format!(
                "Missing closing parenthesis in port list: {}",
                s
            )))
        }
        _ => Err(PortListError::General("Invalid port list".to_string())),
    }
}

fn parse_protocol(s: &str) -> Result<u8, PortListError> {
    let mut parts = s.split(',');

    let protocol = parts
        .next()
        .ok_or_else(|| PortListError::General(format!("Missing comma in port list ({})", s)))?
        .trim();

    let protocol = protocol
        .strip_prefix("protocol")
        .ok_or_else(|| {
            PortListError::General(format!("Missing 'protocol' prefix in: ({})", protocol))
        })?
        .trim();

    let protocol_val = protocol
        .parse()
        .map_err(|_| PortListError::General(format!("Invalid protocol number {}", protocol)))?;

    Ok(protocol_val)
}

fn parse_ports(s: &str) -> Result<(u16, u16), PortListError> {
    let mut parts = s.split("port");

    let ports = parts
        .nth(1)
        .ok_or_else(|| PortListError::General(format!("Missing port ({})", s)))?
        .trim();

    let mut split = ports.split('-');

    let start = split
        .next()
        .ok_or_else(|| PortListError::General(format!("Missing start port ({})", ports)))?
        .trim();

    let start = start
        .parse::<u16>()
        .map_err(|_| PortListError::General(format!("Invalid start port number {}", start)))?;

    let end = split.next();
    let end = match end {
        Some(end) => end
            .trim()
            .parse::<u16>()
            .map_err(|_| PortListError::General(format!("Invalid end port number {}", end)))?,
        None => start,
    };

    Ok((start, end))
}

impl PortList {
    pub fn capacity(&self) -> u64 {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_protocol() {
        let input = "protocol 6, port 17444";
        let protocol = parse_protocol(input).unwrap();
        assert_eq!(protocol, 6);
    }

    #[test]
    fn test_parse_protocol_missing_protocol() {
        let input = "6, port 17444";
        let result = parse_protocol(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_protocol_invalid_protocol() {
        let input = "protocol six, port 17444";
        let result = parse_protocol(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_ports_single_port() {
        let input = "protocol 6, port 17444";
        let ports = parse_ports(input).unwrap();
        assert_eq!(ports, (17444, 17444));
    }

    #[test]
    fn test_parse_ports_range() {
        let input = "protocol 6, port 17444-17445";
        let ports = parse_ports(input).unwrap();
        assert_eq!(ports, (17444, 17445));
    }

    #[test]
    fn test_parse_ports_missing_ports() {
        let input = "protocol 6";
        let result = parse_ports(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_ports_invalid_ports() {
        let input = "protocol 6, port 17444-";
        let result = parse_ports(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_name_and_ports_single_port() {
        let input = "protocol 6, port 17444";
        let (name, ports) = get_name_and_ports(input).unwrap();
        assert_eq!(name, "protocol 6, port 17444");
        assert_eq!(ports, "protocol 6, port 17444");
    }

    #[test]
    fn test_get_name_and_ports_named_port() {
        let input = "HTTP (protocol 6, port 80)";
        let (name, ports) = get_name_and_ports(input).unwrap();
        assert_eq!(name, "HTTP");
        assert_eq!(ports, "protocol 6, port 80");
    }

    #[test]
    fn test_get_name_and_ports_named_port_range() {
        let input = "HTTP (protocol 6, port 80-81)";
        let (name, ports) = get_name_and_ports(input).unwrap();
        assert_eq!(name, "HTTP");
        assert_eq!(ports, "protocol 6, port 80-81");
    }

    #[test]
    fn test_get_name_and_ports_missing_closing_parenthesis() {
        let input = "HTTP (protocol 6, port 80-81";
        let result = get_name_and_ports(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_name_and_ports_missing_opening_parenthesis() {
        let input = "HTTP protocol 6, port 80-81)";
        let result = get_name_and_ports(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_single_port() {
        let input = "protocol 6, port 17444";
        let port_list = input.parse::<PortList>().unwrap();
        assert_eq!(port_list.name, "protocol 6, port 17444");
        assert_eq!(port_list.protocol, 6);
        assert_eq!(port_list.start, 17444);
        assert_eq!(port_list.end, 17444);
    }

    #[test]
    fn test_named_single_port() {
        let input = "HTTP (protocol 6, port 80)";
        let port_list = input.parse::<PortList>().unwrap();
        assert_eq!(port_list.name, "HTTP");
        assert_eq!(port_list.protocol, 6);
        assert_eq!(port_list.start, 80);
        assert_eq!(port_list.end, 80);
    }

    #[test]
    fn test_named_port_range() {
        let input = "HTTP (protocol 6, port 80-81)";
        // let port_list = input.parse::<PortList>().unwrap();
        let port_list = PortList::from_str(input).unwrap();
        assert_eq!(port_list.name, "HTTP");
        assert_eq!(port_list.protocol, 6);
        assert_eq!(port_list.start, 80);
        assert_eq!(port_list.end, 81);
    }

    #[test]
    fn test_invalid_format() {
        let input = "Invalid format";
        let result = input.parse::<PortList>();
        assert!(result.is_err());
    }
    #[test]
    fn test_empty_string() {
        let input = "";
        let result = input.parse::<PortList>();
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_protocol() {
        let input = "HTTP (port 80)";
        let result = input.parse::<PortList>();
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_port() {
        let input = "HTTP (protocol 6)";
        let result = input.parse::<PortList>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_port_range() {
        let input = "HTTP (protocol 6, port 81-)";
        let result = input.parse::<PortList>();
        assert!(result.is_err());
    }

    #[test]
    fn test_extra_whitespace() {
        let input = "  HTTP  (  protocol 6 ,  port 80-81  )  ";
        let port_list = input.parse::<PortList>().unwrap();
        assert_eq!(port_list.name, "HTTP");
        assert_eq!(port_list.protocol, 6);
        assert_eq!(port_list.start, 80);
        assert_eq!(port_list.end, 81);
    }

    #[test]
    fn test_invalid_protocol() {
        let input = "HTTP (protocol six, port 80)";
        let result = input.parse::<PortList>();
        assert!(result.is_err());
    }
}
