use quick_xml::events::Event;
use quick_xml::Reader;

#[derive(Debug, Clone)]
pub struct NmapPort {
    pub port: u16,
    pub protocol: String,
    pub state: String,
    pub service: String,
    pub version: String,
}

pub fn parse_nmap_xml(xml: &str) -> Vec<NmapPort> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut ports = Vec::new();
    let mut buf = Vec::new();

    let mut current_port = None;
    let mut current_protocol = String::new();
    let mut current_state = String::new();
    let mut current_service = String::new();
    let mut current_version = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                b"port" => {
                    current_protocol = e
                        .attributes()
                        .filter_map(|a| a.ok())
                        .find(|a| a.key.as_ref() == b"protocol")
                        .map(|a| String::from_utf8_lossy(&a.value).to_string())
                        .unwrap_or_default();
                    current_port = e
                        .attributes()
                        .filter_map(|a| a.ok())
                        .find(|a| a.key.as_ref() == b"portid")
                        .map(|a| String::from_utf8_lossy(&a.value).to_string())
                        .and_then(|s| s.parse().ok());
                }
                b"state" => {
                    current_state = e
                        .attributes()
                        .filter_map(|a| a.ok())
                        .find(|a| a.key.as_ref() == b"state")
                        .map(|a| String::from_utf8_lossy(&a.value).to_string())
                        .unwrap_or_default();
                }
                b"service" => {
                    for attr in e.attributes().filter_map(|a| a.ok()) {
                        match attr.key.as_ref() {
                            b"name" => {
                                current_service = String::from_utf8_lossy(&attr.value).to_string()
                            }
                            b"product" => {
                                current_version = String::from_utf8_lossy(&attr.value).to_string()
                            }
                            b"version" => {
                                let v = String::from_utf8_lossy(&attr.value);
                                if !current_version.is_empty() {
                                    current_version.push(' ');
                                }
                                current_version.push_str(&v);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"port" {
                    if let Some(port) = current_port.take() {
                        ports.push(NmapPort {
                            port,
                            protocol: std::mem::take(&mut current_protocol),
                            state: std::mem::take(&mut current_state),
                            service: std::mem::take(&mut current_service),
                            version: std::mem::take(&mut current_version),
                        });
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                tracing::warn!("XML parse error: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    ports
}

pub fn build_nmap_command(target: &str, extra_flags: Option<&[&str]>) -> Vec<String> {
    let mut args = vec![
        "-sV".to_string(),
        "-sC".to_string(),
        "-T4".to_string(),
        "-oX".to_string(),
        "-".to_string(),
    ];

    if let Some(flags) = extra_flags {
        for flag in flags {
            args.push(flag.to_string());
        }
    }

    args.push(target.to_string());
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nmap_xml() {
        let xml = r#"<?xml version="1.0"?>
<nmaprun>
  <host>
    <ports>
      <port protocol="tcp" portid="22">
        <state state="open" reason="syn-ack"/>
        <service name="ssh" product="OpenSSH" version="8.9p1"/>
      </port>
      <port protocol="tcp" portid="80">
        <state state="open" reason="syn-ack"/>
        <service name="http" product="Apache httpd" version="2.4.52"/>
      </port>
      <port protocol="tcp" portid="3306">
        <state state="open" reason="syn-ack"/>
        <service name="mysql" product="MySQL" version="8.0.33"/>
      </port>
    </ports>
  </host>
</nmaprun>"#;

        let ports = parse_nmap_xml(xml);
        assert_eq!(ports.len(), 3);
        assert_eq!(ports[0].port, 22);
        assert_eq!(ports[0].service, "ssh");
        assert_eq!(ports[1].port, 80);
        assert_eq!(ports[1].service, "http");
        assert_eq!(ports[2].port, 3306);
        assert_eq!(ports[2].service, "mysql");
    }

    #[test]
    fn test_parse_nmap_xml_no_ports() {
        let xml = r#"<?xml version="1.0"?>
<nmaprun>
  <host>
    <ports>
    </ports>
  </host>
</nmaprun>"#;

        let ports = parse_nmap_xml(xml);
        assert!(ports.is_empty());
    }

    #[test]
    fn test_build_nmap_command() {
        let args = build_nmap_command("10.0.0.1", Some(&["-p-"]));
        assert!(args.contains(&"-sV".to_string()));
        assert!(args.contains(&"-p-".to_string()));
        assert!(args.contains(&"10.0.0.1".to_string()));
    }
}
