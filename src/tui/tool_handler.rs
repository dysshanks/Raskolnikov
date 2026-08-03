use crate::ai::Message;
use crate::session::transcript::TranscriptEntry;
use crate::tui::app::{App, AppState};
use chrono::Utc;

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

        let parts: Vec<String> = shell_words::split(&command).unwrap_or_else(|e| {
            self.conversation
                .push(format!("[system] Could not parse command: {}", e));
            Vec::new()
        });

        if parts.is_empty() {
            self.conversation
                .push("[system] Empty command, nothing to run.".to_string());
            self.state = AppState::Idle;
            return;
        }

        let interrupt_rx = self.interrupt_tx.subscribe();
        let cmd = parts[0].clone();
        let args: Vec<String> = parts[1..].to_vec();
        let (tx, rx) = tokio::sync::oneshot::channel();

        let streaming = self.config.ui.stream_output;
        if streaming {
            let (output_tx, output_rx) = tokio::sync::mpsc::unbounded_channel();
            self.tool_output_rx = Some(output_rx);

            tokio::spawn(async move {
                let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                let result = crate::tools::executor::run_tool_streaming(
                    &cmd,
                    &arg_refs,
                    interrupt_rx,
                    output_tx,
                )
                .await;
                let _ = tx.send(result);
            });
        } else {
            tokio::spawn(async move {
                let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                let result = crate::tools::executor::run_tool(&cmd, &arg_refs, interrupt_rx).await;
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
                    let clean = crate::tools::sanitize::sanitize_output(line);
                    self.conversation.push(format!("│ {}", clean));
                }
            }
            if !result.stderr.is_empty() {
                for line in result.stderr.lines() {
                    let clean = crate::tools::sanitize::sanitize_output(line);
                    self.conversation.push(format!("│ [stderr] {}", clean));
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

            let tags = crate::tools::parse::parse_tool_output(
                &tool,
                &output,
                &mut self.agent_shell.context,
            );
            if !tags.is_empty() {
                let mut added = 0;
                for tag in &tags {
                    if !self.findings.iter().any(|f| f == tag) {
                        self.findings.push(tag.clone());
                        self.conversation.push(format!("[finding] {}", tag));
                        added += 1;
                    }
                }
                if added > 0 {
                    self.toast = Some((
                        format!("{} finding(s) parsed", added),
                        std::time::Instant::now(),
                    ));
                }
            }

            self.state = AppState::Idle;
            self.processing = true;
        }
        self.scroll_offset_conv = usize::MAX;
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
    async fn test_spawn_empty_command_noop() {
        let mut app = make_test_app();
        app.pending_tool = Some("custom".to_string());
        app.spawn_tool("".to_string());
        assert_eq!(app.state, AppState::Idle);
        assert!(app.tool_rx.is_none());
    }

    #[tokio::test]
    async fn test_spawn_tool_parses_quoted_args() {
        let mut app = make_test_app();
        app.pending_tool = Some("gobuster".to_string());
        app.spawn_tool("gobuster dir -u http://host -w \"/path with spaces/list.txt\"".to_string());
        assert_eq!(app.state, AppState::ToolRunning);
        assert!(app.tool_rx.is_some());
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
