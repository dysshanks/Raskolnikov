use chrono::Utc;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

const FLUSH_THRESHOLD: usize = 5000;

#[derive(Debug, Serialize)]
pub struct SessionEvent {
    pub ts: String,
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_s: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

pub struct SessionLogger {
    events: Vec<SessionEvent>,
    path: PathBuf,
}

impl SessionLogger {
    pub fn new(session_dir: &PathBuf) -> Result<Self, std::io::Error> {
        fs::create_dir_all(session_dir)?;
        let path = session_dir.join("session.log");
        Ok(Self {
            events: Vec::with_capacity(256),
            path,
        })
    }

    pub fn log(&mut self, event: SessionEvent) {
        self.events.push(event);
        if self.events.len() >= FLUSH_THRESHOLD {
            self.flush_to_disk();
        }
    }

    pub fn flush_to_disk(&mut self) {
        if self.events.is_empty() {
            return;
        }
        let mut compressed = Vec::new();
        {
            let mut encoder = GzEncoder::new(&mut compressed, Compression::default());
            for event in &self.events {
                if let Ok(json) = serde_json::to_string(event) {
                    let _ = writeln!(encoder, "{}", json);
                }
            }
            let _ = encoder.finish();
        }
        let _ = fs::write(&self.path, compressed);
        self.events.clear();
    }

    pub fn session_start(&mut self, model: &str, provider: &str) {
        self.log(SessionEvent {
            ts: Utc::now().to_rfc3339(),
            event_type: "session_start".to_string(),
            content: None,
            tool: None,
            cmd: None,
            exit_code: None,
            duration_s: None,
            model: Some(model.to_string()),
            provider: Some(provider.to_string()),
            source: None,
        });
    }

    pub fn operator_message(&mut self, content: &str) {
        self.log(SessionEvent {
            ts: Utc::now().to_rfc3339(),
            event_type: "operator".to_string(),
            content: Some(content.to_string()),
            tool: None,
            cmd: None,
            exit_code: None,
            duration_s: None,
            model: None,
            provider: None,
            source: None,
        });
    }

    pub fn agent_message(&mut self, content: &str) {
        self.log(SessionEvent {
            ts: Utc::now().to_rfc3339(),
            event_type: "agent".to_string(),
            content: Some(content.to_string()),
            tool: None,
            cmd: None,
            exit_code: None,
            duration_s: None,
            model: None,
            provider: None,
            source: None,
        });
    }

    pub fn tool_start(&mut self, tool: &str, cmd: &str) {
        self.log(SessionEvent {
            ts: Utc::now().to_rfc3339(),
            event_type: "tool_start".to_string(),
            content: None,
            tool: Some(tool.to_string()),
            cmd: Some(cmd.to_string()),
            exit_code: None,
            duration_s: None,
            model: None,
            provider: None,
            source: None,
        });
    }

    pub fn tool_end(&mut self, tool: &str, exit_code: i32, duration_s: u64) {
        self.log(SessionEvent {
            ts: Utc::now().to_rfc3339(),
            event_type: "tool_end".to_string(),
            content: None,
            tool: Some(tool.to_string()),
            cmd: None,
            exit_code: Some(exit_code),
            duration_s: Some(duration_s),
            model: None,
            provider: None,
            source: None,
        });
    }

    pub fn session_end(&mut self, duration_s: u64) {
        self.log(SessionEvent {
            ts: Utc::now().to_rfc3339(),
            event_type: "session_end".to_string(),
            content: None,
            tool: None,
            cmd: None,
            exit_code: None,
            duration_s: Some(duration_s),
            model: None,
            provider: None,
            source: None,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_logger_buffers_and_flushes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let mut logger = SessionLogger::new(&path).unwrap();

        logger.session_start("qwen3", "ollama");
        logger.operator_message("scan 10.0.0.1");

        // File should NOT exist yet — still buffered
        let log_path = path.join("session.log");
        assert!(!log_path.exists());

        // Flush to disk (compressed)
        logger.flush_to_disk();
        assert!(log_path.exists());

        // Read and verify compressed content
        let mut raw = Vec::new();
        fs::File::open(&log_path)
            .unwrap()
            .read_to_end(&mut raw)
            .unwrap();
        let mut decoder = flate2::read::GzDecoder::new(&raw[..]);
        let mut content = String::new();
        decoder.read_to_string(&mut content).unwrap();
        assert!(content.contains("session_start"));
        assert!(content.contains("operator"));
        assert!(content.contains("scan 10.0.0.1"));
    }

    #[test]
    fn test_logger_auto_flushes_at_threshold() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let mut logger = SessionLogger::new(&path).unwrap();

        // FLUSH_THRESHOLD is 5000, push 5001 events to trigger auto-flush
        for i in 0..5001 {
            logger.operator_message(&format!("event {}", i));
        }

        let log_path = path.join("session.log");
        assert!(log_path.exists()); // auto-flushed on 5000th event

        // Events should be cleared from buffer after flush
        assert!(logger.events.len() < 100);
    }
}
