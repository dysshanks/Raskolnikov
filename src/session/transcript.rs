use std::fs;
use std::path::Path;

pub struct Transcript;

impl Transcript {
    pub fn write(
        path: &Path,
        session_id: &str,
        model: &str,
        provider: &str,
        entries: &[TranscriptEntry],
    ) -> Result<(), std::io::Error> {
        let mut md = String::new();
        md.push_str(&format!("# Session: {}\n", session_id));
        md.push_str(&format!(
            "**Model:** {}  **Provider:** {}\n\n",
            model, provider
        ));
        md.push_str("---\n\n");

        for entry in entries {
            match entry {
                TranscriptEntry::Operator { ts, content } => {
                    md.push_str(&format!("**[{}] you**\n{}\n\n", ts, content));
                }
                TranscriptEntry::Agent { ts, content } => {
                    md.push_str(&format!("**[{}] agent**\n{}\n\n", ts, content));
                }
                TranscriptEntry::Tool {
                    ts,
                    tool,
                    duration,
                    output,
                } => {
                    md.push_str(&format!(
                        "**[{}] tool: {}** *({}s)*\n```\n{}\n```\n\n",
                        ts, tool, duration, output
                    ));
                }
                TranscriptEntry::Finding {
                    ts,
                    source,
                    description,
                } => {
                    md.push_str(&format!(
                        "**[{}] finding** [{}] {}\n\n",
                        ts, source, description
                    ));
                }
            }
        }

        fs::write(path, md)
    }
}

pub enum TranscriptEntry {
    Operator {
        ts: String,
        content: String,
    },
    Agent {
        ts: String,
        content: String,
    },
    Tool {
        ts: String,
        tool: String,
        duration: u64,
        output: String,
    },
    Finding {
        ts: String,
        source: String,
        description: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcript_write() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("conversation.md");

        let entries = vec![
            TranscriptEntry::Operator {
                ts: "14:22:05".to_string(),
                content: "scan 10.0.0.1".to_string(),
            },
            TranscriptEntry::Agent {
                ts: "14:22:07".to_string(),
                content: "Starting with nmap.\n`nmap -sV -sC -T4 10.0.0.1`".to_string(),
            },
            TranscriptEntry::Tool {
                ts: "14:22:09".to_string(),
                tool: "nmap".to_string(),
                duration: 92,
                output: "22/tcp open ssh".to_string(),
            },
        ];

        Transcript::write(&path, "2025-06-16T14-22-01", "qwen3", "ollama", &entries).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Session: 2025-06-16T14-22-01"));
        assert!(content.contains("scan 10.0.0.1"));
        assert!(content.contains("nmap"));
        assert!(content.contains("92"));
    }
}
