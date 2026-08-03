use crate::agent::context::{Credential, EngagementContext};
use crate::tools::gobuster::{parse_ffuf_json, parse_gobuster_output};
use crate::tools::nmap::parse_nmap_xml;

/// Returns a short human-readable summary line for the findings bar.
/// The dispatch by tool name runs the matching parser and folds results into
/// `context`. Parsers are best-effort: unrecognised output yields nothing.
pub fn parse_tool_output(tool: &str, output: &str, context: &mut EngagementContext) -> Vec<String> {
    let mut tags = Vec::new();

    match tool {
        "nmap" => {
            for port in parse_nmap_xml(output) {
                if port.state == "open" {
                    let exists = context
                        .ports
                        .iter()
                        .any(|p| p.port == port.port && p.protocol == port.protocol);
                    if !exists {
                        context.ports.push(crate::agent::context::Port {
                            port: port.port,
                            protocol: port.protocol.clone(),
                            state: port.state.clone(),
                            service: port.service.clone(),
                            version: port.version.clone(),
                        });
                        tags.push(format!(
                            "{}/tcp {} {}",
                            port.port, port.protocol, port.service
                        ));
                    }
                }
            }
            if !tags.is_empty() {
                context.add_finding(
                    format!("nmap found {} open port(s)", tags.len()),
                    "nmap".to_string(),
                );
            }
        }
        "gobuster" => {
            for path in parse_gobuster_output(output) {
                let exists = context
                    .web_paths
                    .iter()
                    .any(|p| p.path == path.path && p.status_code == path.status_code);
                if !exists {
                    context.web_paths.push(crate::agent::context::WebPath {
                        path: path.path.clone(),
                        status_code: path.status_code,
                        notes: String::new(),
                    });
                    tags.push(format!("{} ({})", path.path, path.status_code));
                }
            }
        }
        "ffuf" => {
            for path in parse_ffuf_json(output) {
                let exists = context
                    .web_paths
                    .iter()
                    .any(|p| p.path == path.path && p.status_code == path.status_code);
                if !exists {
                    context.web_paths.push(crate::agent::context::WebPath {
                        path: path.path.clone(),
                        status_code: path.status_code,
                        notes: String::new(),
                    });
                    tags.push(format!("{} ({})", path.path, path.status_code));
                }
            }
        }
        "feroxbuster" => {
            for path in crate::tools::feroxbuster::parse_feroxbuster_output(output) {
                let exists = context
                    .web_paths
                    .iter()
                    .any(|p| p.path == path.path && p.status_code == path.status_code);
                if !exists {
                    context.web_paths.push(crate::agent::context::WebPath {
                        path: path.path.clone(),
                        status_code: path.status_code,
                        notes: String::new(),
                    });
                    tags.push(format!("{} ({})", path.path, path.status_code));
                }
            }
        }
        "httpx" | "httpx-toolkit" => {
            for path in crate::tools::httpx::parse_httpx_output(output) {
                let exists = context
                    .web_paths
                    .iter()
                    .any(|p| p.path == path.path && p.status_code == path.status_code);
                if !exists {
                    context.web_paths.push(crate::agent::context::WebPath {
                        path: path.path.clone(),
                        status_code: path.status_code,
                        notes: String::new(),
                    });
                    tags.push(format!("{} ({})", path.path, path.status_code));
                }
            }
        }
        "nikto" => {
            for finding in crate::tools::nikto::parse_nikto_output(output) {
                context.add_finding(finding.description.clone(), "nikto".to_string());
                tags.push(finding.description.clone());
            }
        }
        "sqlmap" => {
            for finding in crate::tools::sqlmap::parse_sqlmap_output(output) {
                let desc = format!(
                    "SQLi in {} ({} via {})",
                    finding.parameter, finding.injection_type, finding.technique
                );
                context.add_finding(desc.clone(), "sqlmap".to_string());
                tags.push(desc);
            }
        }
        "hydra" => {
            for cred in crate::tools::hydra::parse_hydra_output(output) {
                context.add_credential(Credential {
                    service: cred.service.clone(),
                    host: cred.host.clone(),
                    user: cred.user.clone(),
                    password: cred.password.clone(),
                });
                tags.push(format!(
                    "{} {}:{}:{}",
                    cred.service, cred.host, cred.user, cred.password
                ));
            }
        }
        "nxc" | "netexec" => {
            for cred in crate::tools::netexec::parse_netexec_output(output) {
                context.add_credential(Credential {
                    service: format!("smb/{}", cred.port),
                    host: cred.host.clone(),
                    user: cred.user.clone(),
                    password: cred.password.clone(),
                });
                context.add_finding(
                    format!(
                        "{} valid {}:{} on {} {}",
                        if cred.pwned { "Pwn3d" } else { "valid" },
                        cred.user,
                        cred.password,
                        cred.host,
                        if cred.pwned { "(admin)" } else { "" }
                    ),
                    "netexec".to_string(),
                );
                tags.push(format!("{}:{}@{}", cred.user, cred.password, cred.host));
            }
        }
        "john" => {
            for crack in crate::tools::john::parse_john_output(output) {
                let desc = format!("cracked hash {} = {}", crack.hash, crack.password);
                context.add_finding(desc.clone(), "john".to_string());
                tags.push(desc);
            }
        }
        "hashcat" => {
            for crack in crate::tools::hashcat::parse_hashcat_output(output) {
                let desc = format!("cracked hash {} = {}", crack.hash, crack.password);
                context.add_finding(desc.clone(), "hashcat".to_string());
                tags.push(desc);
            }
        }
        "nuclei" => {
            for finding in crate::tools::nuclei::parse_nuclei_output(output) {
                let mut desc = format!("{} {} {}", finding.severity, finding.url, finding.protocol);
                if let Some(cve) = &finding.cve {
                    desc.push_str(&format!(" [{}]", cve));
                }
                context.add_finding(desc.clone(), "nuclei".to_string());
                tags.push(desc);
            }
        }
        "enum4linux-ng" | "enum4linux" => {
            for finding in crate::tools::enum4linux::parse_enum4linux_output(output) {
                let desc = format!("{} {}", finding.kind, finding.value);
                context.add_finding(desc.clone(), "enum4linux-ng".to_string());
                tags.push(desc);
            }
        }
        "whatweb" => {
            for finding in crate::tools::whatweb::parse_whatweb_output(output) {
                let desc = match &finding.version {
                    Some(v) => format!("{}[{}] on {}", finding.tech, v, finding.url),
                    None => format!("{} on {}", finding.tech, finding.url),
                };
                context.add_finding(desc.clone(), "whatweb".to_string());
                tags.push(desc);
            }
        }
        _ => {}
    }

    tags
}

#[cfg(test)]
mod tests {
    use super::*;

    fn xml_ports() -> &'static str {
        r#"<?xml version="1.0"?>
<nmaprun>
  <host>
    <ports>
      <port protocol="tcp" portid="80">
        <state state="open" reason="syn-ack"/>
        <service name="http" product="Apache httpd" version="2.4.52"/>
      </port>
    </ports>
  </host>
</nmaprun>"#
    }

    #[test]
    fn test_parse_nmap_populates_context() {
        let mut ctx = EngagementContext::new();
        let tags = parse_tool_output("nmap", xml_ports(), &mut ctx);
        assert_eq!(ctx.ports.len(), 1);
        assert_eq!(ctx.ports[0].port, 80);
        assert_eq!(ctx.ports[0].service, "http");
        assert_eq!(ctx.findings.len(), 1);
        assert!(!tags.is_empty());
    }

    #[test]
    fn test_parse_hydra_populates_credentials() {
        let mut ctx = EngagementContext::new();
        let tags = parse_tool_output(
            "hydra",
            "[22][ssh] host login: root password: toor",
            &mut ctx,
        );
        assert_eq!(ctx.credentials.len(), 1);
        assert_eq!(ctx.credentials[0].user, "root");
        assert_eq!(ctx.credentials[0].password, "toor");
        assert!(!tags.is_empty());
    }

    #[test]
    fn test_parse_gobuster_populates_web_paths() {
        let mut ctx = EngagementContext::new();
        let tags = parse_tool_output(
            "gobuster",
            "/admin (Status: 302)\n/x (Status: 200)",
            &mut ctx,
        );
        assert_eq!(ctx.web_paths.len(), 2);
        assert!(!tags.is_empty());
    }

    #[test]
    fn test_unknown_tool_noop() {
        let mut ctx = EngagementContext::new();
        let tags = parse_tool_output("nonexistent", "whatever", &mut ctx);
        assert!(tags.is_empty());
        assert!(ctx.findings.is_empty());
        assert!(ctx.ports.is_empty());
    }

    #[test]
    fn test_garbage_output_does_not_panic() {
        let mut ctx = EngagementContext::new();
        let tags = parse_tool_output("sqlmap", "!!! not sqlmap output !!!", &mut ctx);
        assert!(tags.is_empty());
    }
}
