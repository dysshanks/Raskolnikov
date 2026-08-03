use crate::agent::context::Port;

/// Parses `masscan -oL -` list output into open ports:
///
/// ```text
/// open tcp 80 10.0.0.1 1600000000
/// open tcp 443 10.0.0.1 1600000001
/// ```
pub fn parse_masscan_output(output: &str) -> Vec<Port> {
    output
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            if parts.next()? != "open" {
                return None;
            }
            let protocol = parts.next()?.to_string();
            let port: u16 = parts.next()?.parse().ok()?;
            Some(Port {
                port,
                protocol,
                state: "open".to_string(),
                service: String::new(),
                version: String::new(),
            })
        })
        .collect()
}

/// Builds a masscan command emitting the list format (`-oL -`) to stdout.
pub fn build_masscan_command(target: &str, ports: &str, rate: u32) -> Vec<String> {
    vec![
        "--rate".to_string(),
        rate.to_string(),
        "-oL".to_string(),
        "-".to_string(),
        "-p".to_string(),
        ports.to_string(),
        target.to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_masscan_output() {
        let output = "open tcp 22 10.0.0.1 1600000000\nopen tcp 80 10.0.0.1 1600000001\n";
        let ports = parse_masscan_output(output);
        assert_eq!(ports.len(), 2);
        assert_eq!(ports[0].port, 22);
        assert_eq!(ports[0].protocol, "tcp");
        assert_eq!(ports[0].state, "open");
        assert_eq!(ports[1].port, 80);
    }

    #[test]
    fn test_parse_masscan_empty() {
        assert!(parse_masscan_output("").is_empty());
    }

    #[test]
    fn test_parse_masscan_noise() {
        let noise = "Starting masscan 1.3.2 at ...\nBanner tcp/80 ...\n";
        assert!(parse_masscan_output(noise).is_empty());
    }

    #[test]
    fn test_build_masscan_command() {
        let args = build_masscan_command("10.0.0.0/24", "1-1000", 1000);
        assert!(args.contains(&"-oL".to_string()));
        assert!(args.contains(&"1000".to_string()));
        assert!(args.contains(&"10.0.0.0/24".to_string()));
    }
}
