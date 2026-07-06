use crate::ai::Message;
use crate::session::transcript::TranscriptEntry;
use crate::tools::executor::ToolRunResult;
use crate::tui::app::{App, AppState};
use chrono::Utc;
use std::time::Duration;

impl App {
    pub fn parse_tool_suggestion(&mut self, response: &str) {
        let trimmed = response.trim();
        if !trimmed.ends_with("— run this?") && !trimmed.ends_with("-- run this?") {
            return;
        }

        let re = regex::Regex::new(r"```(?:\w+\n)?([^`]+)```").unwrap();
        let caps: Vec<_> = re.captures_iter(response).collect();

        if let Some(cap) = caps.last() {
            let command = cap[1].trim().to_string();
            if command.is_empty() {
                return;
            }

            let tool_name = command.split_whitespace().next().unwrap_or("").to_string();
            if tool_name.is_empty() {
                return;
            }

            self.pending_tool = Some(tool_name);
            self.pending_command = Some(command);
            self.state = AppState::AwaitingConfirm;
        }
    }

    pub fn spawn_tool(&mut self, command: String) {
        let tool_name = self
            .pending_tool
            .take()
            .unwrap_or_else(|| command.split_whitespace().next().unwrap_or("?").to_string());

        if let Some(logger) = &mut self.logger {
            logger.tool_start(&tool_name, &command);
        }

        self.conversation
            .push(format!("── Running: {} ──", command));

        let interrupt_rx = self.interrupt_tx.subscribe();
        let cmd_clone = command.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();

        let streaming = self.config.ui.stream_output;
        if streaming {
            let (output_tx, output_rx) = tokio::sync::mpsc::unbounded_channel();
            self.tool_output_rx = Some(output_rx);

            tokio::spawn(async move {
                let parts: Vec<&str> = cmd_clone.split_whitespace().collect();
                let result = if parts.is_empty() {
                    ToolRunResult {
                        exit_code: Some(-1),
                        stdout: String::new(),
                        stderr: "Empty command".to_string(),
                        duration: Duration::default(),
                        was_interrupted: false,
                    }
                } else {
                    crate::tools::executor::run_tool_streaming(
                        parts[0],
                        &parts[1..],
                        interrupt_rx,
                        output_tx,
                    )
                    .await
                };
                let _ = tx.send(result);
            });
        } else {
            tokio::spawn(async move {
                let parts: Vec<&str> = cmd_clone.split_whitespace().collect();
                let result = if parts.is_empty() {
                    ToolRunResult {
                        exit_code: Some(-1),
                        stdout: String::new(),
                        stderr: "Empty command".to_string(),
                        duration: Duration::default(),
                        was_interrupted: false,
                    }
                } else {
                    crate::tools::executor::run_tool(parts[0], &parts[1..], interrupt_rx).await
                };
                let _ = tx.send(result);
            });
        }

        self.tool_rx = Some(rx);
        self.tool_name = Some(tool_name);
        self.tool_count += 1;
        self.state = AppState::ToolRunning;
    }

    pub async fn check_tool_completion(&mut self) {
        let rx = match &mut self.tool_rx {
            Some(rx) => rx,
            None => return,
        };

        let result = match rx.try_recv() {
            Ok(result) => result,
            Err(tokio::sync::oneshot::error::TryRecvError::Empty) => return,
            Err(_) => {
                self.tool_rx = None;
                self.state = AppState::Idle;
                return;
            }
        };

        self.tool_rx = None;
        self.tool_output_rx = None;
        let tool = self.tool_name.take().unwrap_or_default();

        if !self.config.ui.stream_output {
            if !result.stdout.is_empty() {
                for line in result.stdout.lines() {
                    self.conversation.push(format!("│ {}", line));
                }
            }
            if !result.stderr.is_empty() {
                for line in result.stderr.lines() {
                    self.conversation.push(format!("│ [stderr] {}", line));
                }
            }
        }

        if let Some(logger) = &mut self.logger {
            logger.tool_end(
                &tool,
                result.exit_code.unwrap_or(-1),
                result.duration.as_secs(),
            );
        }

        let ts = Utc::now().format("%H:%M:%S").to_string();
        let output = format!(
            "{}{}",
            result.stdout,
            if result.stdout.is_empty() || result.stderr.is_empty() {
                ""
            } else {
                "\n"
            }
        ) + &result.stderr;
        self.tool_outputs.push((tool.clone(), output.clone()));
        self.transcript_entries.push(TranscriptEntry::Tool {
            ts,
            tool: tool.clone(),
            duration: result.duration.as_secs(),
            output: output.clone(),
        });

        if result.was_interrupted {
            self.state = AppState::Interrupted;
            self.conversation.push("── Tool interrupted ──".to_string());
        } else {
            let code = result.exit_code.unwrap_or(-1);
            let secs = result.duration.as_secs();
            self.conversation.push(format!(
                "── Tool `{}` finished (exit {}, {}s) ──",
                tool, code, secs
            ));
            let output_msg = format!(
                "Tool `{}` finished (exit code {}, {}s):\n{}",
                tool, code, secs, output
            );
            self.messages.push(Message::tool(output_msg, &tool));
            self.state = AppState::Idle;
            self.processing = true;
        }
        self.scroll_offset_conv = self.conversation.len();
        self.auto_scroll = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_suggestion_no_run_this() {
        let mut app = make_test_app();
        app.parse_tool_suggestion("Here is some output");
        assert_eq!(app.state, AppState::Idle);
        assert!(app.pending_tool.is_none());
    }

    #[test]
    fn test_parse_tool_suggestion_with_run_this() {
        let mut app = make_test_app();
        app.parse_tool_suggestion("Let's scan.\n```\nnmap -sV 10.0.0.1\n```\n — run this?");
        assert_eq!(app.state, AppState::AwaitingConfirm);
        assert_eq!(app.pending_tool.as_deref(), Some("nmap"));
        assert_eq!(app.pending_command.as_deref(), Some("nmap -sV 10.0.0.1"));
    }

    #[test]
    fn test_parse_tool_suggestion_empty_code_block() {
        let mut app = make_test_app();
        app.parse_tool_suggestion("```\n\n```\n — run this?");
        assert_eq!(app.state, AppState::Idle);
    }

    #[test]
    fn test_parse_tool_suggestion_double_dash_variant() {
        let mut app = make_test_app();
        app.parse_tool_suggestion(
            "Run this:\n```\ngobuster dir -u http://example.com\n```\n-- run this?",
        );
        assert_eq!(app.state, AppState::AwaitingConfirm);
        assert_eq!(app.pending_tool.as_deref(), Some("gobuster"));
    }

    #[test]
    fn test_parse_tool_suggestion_language_tagged_block() {
        let mut app = make_test_app();
        app.parse_tool_suggestion("```bash\nnmap -p 80 target\n```\n — run this?");
        assert_eq!(app.state, AppState::AwaitingConfirm);
        assert_eq!(app.pending_tool.as_deref(), Some("nmap"));
    }

    #[test]
    fn test_parse_tool_suggestion_multiple_blocks_takes_last() {
        let mut app = make_test_app();
        app.parse_tool_suggestion(
            "First:\n```\necho hi\n```\n\nSecond:\n```\nnmap -v\n```\n — run this?",
        );
        assert_eq!(app.state, AppState::AwaitingConfirm);
        assert_eq!(app.pending_tool.as_deref(), Some("nmap"));
    }

    #[tokio::test]
    async fn test_check_tool_completion_no_rx_returns_early() {
        let mut app = make_test_app();
        app.tool_rx = None;
        app.check_tool_completion().await;
    }

    #[tokio::test]
    async fn test_spawn_tool_sets_tool_running_state() {
        let mut app = make_test_app();
        app.pending_tool = Some("nmap".to_string());
        app.spawn_tool("nmap -sV 10.0.0.1".to_string());
        assert_eq!(app.state, AppState::ToolRunning);
        assert!(app.tool_rx.is_some());
        assert_eq!(app.tool_name.as_deref(), Some("nmap"));
        assert!(app.tool_count > 0);
    }

    #[tokio::test]
    async fn test_spawn_empty_parts_uses_pending_tool() {
        let mut app = make_test_app();
        app.pending_tool = Some("custom".to_string());
        app.spawn_tool("".to_string());
        assert!(app.tool_rx.is_some() || app.state == AppState::ToolRunning);
    }

    fn make_test_app() -> App {
        use crate::agent::shell::AgentShell;
        use crate::config;
        use std::path::PathBuf;

        App::new_with(
            None,
            AgentShell::new(vec!["nmap".to_string()]),
            None,
            config::Config::default(),
            "test-session".to_string(),
            PathBuf::from("/tmp"),
        )
    }
}
