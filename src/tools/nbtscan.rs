#[derive(Debug, Clone)]
pub struct NetbiosName {
    pub ip: String,
    pub name: String,
}

/// Parses `nbtscan` output into IP → NetBIOS hostname pairs:
///
/// ```text
/// 10.0.0.1       MACHINE1
/// 10.0.0.2       WORKGROUP\\SERVER1 <server>
/// ```
pub fn parse_nbtscan_output(output: &str) -> Vec<NetbiosName> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty()
                || line.starts_with("Doing NBT scan")
                || line.starts_with("IP address")
            {
                return None;
            }
            let mut parts = line.split_whitespace();
            let ip = parts.next()?;
            if ip.parse::<std::net::IpAddr>().is_err() {
                return None;
            }
            let name = parts.next()?.split('\\').next().unwrap_or("").to_string();
            if name.is_empty() {
                return None;
            }
            Some(NetbiosName {
                ip: ip.to_string(),
                name,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nbtscan_output() {
        let output = "Doing NBT name scan for addresses from 10.0.0.0/24\nIP address       NetBIOS Name     Server    User             MAC address\n10.0.0.1         MACHINE1\n10.0.0.2         WORKGROUP\\SERVER1 <server>\n";
        let names = parse_nbtscan_output(output);
        assert_eq!(names.len(), 2);
        assert_eq!(names[0].ip, "10.0.0.1");
        assert_eq!(names[0].name, "MACHINE1");
        assert_eq!(names[1].name, "WORKGROUP");
    }

    #[test]
    fn test_parse_nbtscan_empty() {
        assert!(parse_nbtscan_output("").is_empty());
    }

    #[test]
    fn test_parse_nbtscan_noise() {
        let noise = "Searching for NetBIOS hosts...\n";
        assert!(parse_nbtscan_output(noise).is_empty());
    }
}
