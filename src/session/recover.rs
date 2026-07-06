use crate::session::findings::{FindingsExport, FlagFinding};
use crate::session::logger::SessionEvent;
use crate::session::transcript::{Transcript, TranscriptEntry};
use std::fs;
use std::io::Read;
use std::path::Path;

pub fn recover_session(session_dir: &Path) -> Result<(), String> {
    let log_path = session_dir.join("session.log");
    let conv_path = session_dir.join("conversation.md");
    let findings_path = session_dir.join("findings.md");

    if !log_path.exists() {
        return Err("session.log not found".to_string());
    }

    let events = read_events(&log_path)?;

    let mut entries: Vec<TranscriptEntry> = Vec::new();
    let ports = Vec::new();
    let web_paths = Vec::new();
    let mut flags: Vec<FlagFinding> = Vec::new();
    let mut session_id = String::new();
    let mut model = String::new();
    let mut provider = String::new();

    for event in &events {
        let ts = event.ts.trim_end_matches('Z').to_string();
        match event.event_type.as_str() {
            "session_start" => {
                session_id = ts.clone();
                if let Some(ref m) = event.model {
                    model = m.clone();
                }
                if let Some(ref p) = event.provider {
                    provider = p.clone();
                }
            }
            "operator" => {
                if let Some(ref content) = event.content {
                    entries.push(TranscriptEntry::Operator {
                        ts: ts.clone(),
                        content: content.clone(),
                    });
                }
            }
            "agent" => {
                if let Some(ref content) = event.content {
                    entries.push(TranscriptEntry::Agent {
                        ts: ts.clone(),
                        content: content.clone(),
                    });
                }
            }
            "tool_start" | "tool_end" => {}
            "finding" => {
                if let Some(ref source) = event.source {
                    if let Some(ref content) = event.content {
                        entries.push(TranscriptEntry::Finding {
                            ts: ts.clone(),
                            source: source.clone(),
                            description: content.clone(),
                        });
                        flags.push(FlagFinding {
                            description: format!("[{}] {}", source, content),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    if session_id.is_empty() {
        let stem = log_path
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "recovered".to_string());
        session_id = stem;
    }

    if !entries.is_empty() {
        Transcript::write(&conv_path, &session_id, &model, &provider, &entries)
            .map_err(|e| format!("Failed to write conversation: {}", e))?;
    } else {
        return Err("No conversational events found in session log".to_string());
    }

    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    FindingsExport::write(
        &findings_path,
        &date,
        &model,
        &provider,
        &ports,
        &web_paths,
        &flags,
    )
    .map_err(|e| format!("Failed to write findings: {}", e))?;

    Ok(())
}

fn read_events(path: &Path) -> Result<Vec<SessionEvent>, String> {
    let raw = fs::read(path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    let mut decoder = flate2::read::GzDecoder::new(&raw[..]);
    let mut content = String::new();
    decoder
        .read_to_string(&mut content)
        .map_err(|e| format!("Failed to decompress {}: {}", path.display(), e))?;

    let mut events = Vec::new();
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let event: SessionEvent =
            serde_json::from_str(line).map_err(|e| format!("Failed to parse event: {}", e))?;
        events.push(event);
    }

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::logger::SessionLogger;

    #[test]
    fn test_recover_from_log() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let mut logger = SessionLogger::new(&path).unwrap();

        logger.session_start("qwen3", "ollama");
        logger.operator_message("scan target");
        logger.agent_message("Scanning with nmap");
        logger.log(crate::session::logger::SessionEvent {
            ts: chrono::Utc::now().to_rfc3339(),
            event_type: "finding".to_string(),
            content: Some("Port 80 open".to_string()),
            tool: None,
            cmd: None,
            exit_code: None,
            duration_s: None,
            model: None,
            provider: None,
            source: Some("nmap".to_string()),
        });
        logger.flush_to_disk();

        recover_session(&path).unwrap();

        assert!(path.join("conversation.md").exists());
        assert!(path.join("findings.md").exists());
        let conv = std::fs::read_to_string(path.join("conversation.md")).unwrap();
        assert!(conv.contains("scan target"));
        assert!(conv.contains("Scanning with nmap"));
    }

    #[test]
    fn test_recover_missing_log() {
        let dir = tempfile::tempdir().unwrap();
        let result = recover_session(dir.path());
        assert!(result.is_err());
    }
}
