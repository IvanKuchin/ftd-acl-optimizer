#[derive(thiserror::Error, Debug)]
pub enum CommonError {
    #[error("Failed to parse name and protocol: {0}")]
    NameAndProtocol(String),
    #[error("Failed to parse protocol: {0}")]
    Protocol(String),
}

// Example 1
// protocol 6, port 17444

// Example 2
// HTTP (protocol 6, port 80)

// Example 3
// HTTP (protocol 6, port 80-81)

pub fn parse_name_and_protocol(s: &str) -> Result<(&str, &str), CommonError> {
    // let mut parts = s.split('(');
    // let name = parts.clone().next().unwrap().trim(); // clone() is needed to avoid consuming the iterator
    // let ports = parts.last().unwrap().trim();
    // let ports = ports.split(")").next().unwrap().trim();

    // Ok((name, ports))

    let mut parts = s.split('(');

    match parts.clone().count() {
        1 => {
            let name = parts.next().unwrap().trim();
            let ports = name;

            if name.contains(')') {
                return Err(CommonError::NameAndProtocol(format!(
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
            Err(CommonError::NameAndProtocol(format!(
                "Missing closing parenthesis in port list: {}",
                s
            )))
        }
        _ => Err(CommonError::NameAndProtocol(format!(
            "Invalid port list {}",
            s
        ))),
    }
}

pub fn parse_protocol(s: &str) -> Result<u8, CommonError> {
    let mut parts = s.split(',');

    let protocol = parts
        .next()
        .ok_or_else(|| CommonError::Protocol(format!("Missing comma in port list ({})", s)))?
        .trim();

    let protocol = protocol
        .strip_prefix("protocol")
        .ok_or_else(|| {
            CommonError::Protocol(format!("Missing 'protocol' prefix in: ({})", protocol))
        })?
        .trim();

    let protocol_val = protocol
        .parse()
        .map_err(|_| CommonError::Protocol(format!("Invalid protocol number {}", protocol)))?;

    Ok(protocol_val)
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
    fn test_get_name_and_ports_single_port() {
        let input = "protocol 6, port 17444";
        let (name, ports) = parse_name_and_protocol(input).unwrap();
        assert_eq!(name, "protocol 6, port 17444");
        assert_eq!(ports, "protocol 6, port 17444");
    }

    #[test]
    fn test_get_name_and_ports_named_port() {
        let input = "HTTP (protocol 6, port 80)";
        let (name, ports) = parse_name_and_protocol(input).unwrap();
        assert_eq!(name, "HTTP");
        assert_eq!(ports, "protocol 6, port 80");
    }

    #[test]
    fn test_get_name_and_ports_named_port_range() {
        let input = "HTTP (protocol 6, port 80-81)";
        let (name, ports) = parse_name_and_protocol(input).unwrap();
        assert_eq!(name, "HTTP");
        assert_eq!(ports, "protocol 6, port 80-81");
    }

    #[test]
    fn test_get_name_and_ports_missing_closing_parenthesis() {
        let input = "HTTP (protocol 6, port 80-81";
        let result = parse_name_and_protocol(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_name_and_ports_missing_opening_parenthesis() {
        let input = "HTTP protocol 6, port 80-81)";
        let result = parse_name_and_protocol(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_name_and_ports_empty() {
        let input = "";
        let result = parse_name_and_protocol(input);
        assert_eq!(result.unwrap(), ("", ""));
    }
}
