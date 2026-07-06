use crate::agent::shell::AgentShell;
use crate::ai::{Message, Provider, ProviderKind, ProviderResponse};
use crate::config;
use crate::session::logger::SessionLogger;
use crate::session::transcript::{Transcript, TranscriptEntry};
use crate::tools::executor::ToolRunResult;

use chrono::Utc;
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::style::Color;
use ratatui::Terminal;
use std::io;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::watch;

pub struct Colors {
    pub accent: Color,
    pub surface: Color,
    pub highlight: Color,
}

impl Colors {
    pub fn from_config(cs: &config::ColorScheme) -> Self {
        Self {
            accent: config::parse_color(&cs.accent),
            surface: config::parse_color(&cs.surface),
            highlight: config::parse_color(&cs.highlight),
        }
    }
}

pub struct CommandDef {
    pub name: &'static str,
    pub description: &'static str,
}

pub(crate) const COMMANDS: &[CommandDef] = &[
    CommandDef {
        name: "/findings <tag>",
        description: "Tag a security finding",
    },
    CommandDef {
        name: "/island",
        description: "Toggle the info island",
    },
    CommandDef {
        name: "/quit",
        description: "End the session",
    },
    CommandDef {
        name: "/update",
        description: "Pull latest source and rebuild",
    },
    CommandDef {
        name: "/update tools",
        description: "Rebuild and update system tools (nmap, gobuster, nikto, sqlmap)",
    },
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    Idle,
    AwaitingConfirm,
    ToolRunning,
    Interrupted,
    ConfirmQuit,
    Updating,
}

pub struct App {
    pub state: AppState,
    pub input: String,
    pub queued_message: Option<String>,
    pub conversation: Vec<String>,
    pub findings: Vec<String>,
    pub scroll_offset_conv: usize,
    pub auto_scroll: bool,
    pub provider: Option<ProviderKind>,
    pub agent_shell: AgentShell,
    pub logger: Option<SessionLogger>,
    pub session_id: String,
    pub start_time: Instant,
    pub interrupt_tx: watch::Sender<bool>,
    pub processing: bool,
    pub messages: Vec<Message>,
    pub pending_tool: Option<String>,
    pub pending_command: Option<String>,
    pub tool_rx: Option<tokio::sync::oneshot::Receiver<ToolRunResult>>,
    pub tool_name: Option<String>,
    pub session_dir: PathBuf,
    pub config: crate::config::Config,
    pub transcript_entries: Vec<TranscriptEntry>,
    pub provider_name: String,
    pub model_name: String,
    pub tool_output_rx: Option<tokio::sync::mpsc::UnboundedReceiver<String>>,
    pub update_output_rx: Option<tokio::sync::mpsc::UnboundedReceiver<String>>,
    pub streaming_rx: Option<tokio::sync::mpsc::UnboundedReceiver<String>>,
    pub stream_done_rx: Option<tokio::sync::oneshot::Receiver<Result<ProviderResponse, String>>>,
    pub toast: Option<(String, Instant)>,
    pub frame_count: u64,
    pub input_history: Vec<String>,
    pub history_index: Option<usize>,
    pub history_saved: String,
    pub show_island: bool,
    pub tool_count: u64,
    pub tool_outputs: Vec<(String, String)>,
    pub filtered_commands: Vec<usize>,
    pub selected_command: usize,
    pub colors: Colors,
}

impl App {
    pub fn new_with(
        provider: Option<ProviderKind>,
        agent_shell: AgentShell,
        logger: Option<SessionLogger>,
        config: crate::config::Config,
        session_id: String,
        session_dir: PathBuf,
    ) -> Self {
        let colors = Colors::from_config(&config.colors);
        let model_name = config.ai.model.clone();
        let provider_name = provider
            .as_ref()
            .map(|p| p.name().to_string())
            .unwrap_or_else(|| "none".to_string());
        let (interrupt_tx, _) = watch::channel(false);

        let mut app = Self {
            state: AppState::Idle,
            input: String::new(),
            queued_message: None,
            conversation: Vec::new(),
            findings: Vec::new(),
            scroll_offset_conv: 0,
            auto_scroll: true,
            provider,
            agent_shell,
            logger,
            session_id,
            start_time: Instant::now(),
            interrupt_tx,
            processing: false,
            messages: Vec::new(),
            pending_tool: None,
            pending_command: None,
            tool_rx: None,
            tool_name: None,
            session_dir,
            config,
            transcript_entries: Vec::new(),
            provider_name,
            model_name,
            tool_output_rx: None,
            update_output_rx: None,
            streaming_rx: None,
            stream_done_rx: None,
            toast: None,
            frame_count: 0,
            input_history: Vec::new(),
            history_index: None,
            history_saved: String::new(),
            show_island: true,
            tool_count: 0,
            tool_outputs: Vec::new(),
            filtered_commands: Vec::new(),
            selected_command: 0,
            colors,
        };

        if let Some(logger) = &mut app.logger {
            logger.session_start(&app.model_name, &app.provider_name);
        }

        app
    }

    pub async fn run(&mut self) {
        enable_raw_mode().expect("Failed to enable raw mode");
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).expect("Failed to enter alternate screen");

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).expect("Failed to create terminal");

        let res = self.run_loop(&mut terminal).await;

        disable_raw_mode().expect("Failed to disable raw mode");
        execute!(terminal.backend_mut(), LeaveAlternateScreen)
            .expect("Failed to leave alternate screen");
        terminal.show_cursor().expect("Failed to show cursor");

        self.save_session();
        if let Err(e) = res {
            eprintln!("Error: {}", e);
        }
    }

    async fn run_loop<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|f| {
                crate::tui::layout::render(self, f);
            })?;

            if let Some(rx) = &mut self.tool_output_rx {
                while let Ok(line) = rx.try_recv() {
                    self.conversation.push(format!("│ {}", line));
                    self.scroll_offset_conv = self.conversation.len();
                    self.auto_scroll = true;
                }
            }

            if let Some(rx) = self.update_output_rx.as_mut() {
                while let Ok(line) = rx.try_recv() {
                    if line == "[update_done]" {
                        self.conversation
                            .push("[system] Update complete.".to_string());
                    } else {
                        self.conversation.push(line);
                    }
                    self.scroll_offset_conv = self.conversation.len();
                    self.auto_scroll = true;
                }
                if rx.is_closed() {
                    self.update_output_rx = None;
                    self.state = AppState::Idle;
                }
            }

            self.check_tool_completion().await;

            if self.processing && self.state != AppState::ToolRunning && self.streaming_rx.is_none()
            {
                self.start_ai_stream();
            }

            if let Some(rx) = &mut self.streaming_rx {
                while let Ok(token) = rx.try_recv() {
                    if let Some(last) = self.conversation.last_mut() {
                        last.push_str(&token);
                    }
                    self.scroll_offset_conv = self.conversation.len();
                    self.auto_scroll = true;
                }
            }

            if let Some(rx) = &mut self.stream_done_rx {
                match rx.try_recv() {
                    Ok(Ok(response)) => {
                        self.streaming_rx = None;
                        self.stream_done_rx = None;
                        self.processing = false;
                        let content = response.content.trim().to_string();
                        if !content.is_empty() {
                            self.messages.push(Message::assistant(&content));
                            if let Some(logger) = &mut self.logger {
                                logger.agent_message(&content);
                            }
                            let ts = Utc::now().format("%H:%M:%S").to_string();
                            self.transcript_entries.push(TranscriptEntry::Agent {
                                ts,
                                content: content.clone(),
                            });
                            self.parse_tool_suggestion(&content);
                        }
                    }
                    Ok(Err(e)) => {
                        self.streaming_rx = None;
                        self.stream_done_rx = None;
                        self.processing = false;
                        self.conversation.push(format!("[system] {}", e));
                    }
                    Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {}
                    Err(_) => {
                        self.streaming_rx = None;
                        self.stream_done_rx = None;
                        self.processing = false;
                    }
                }
            }

            if self.state == AppState::ToolRunning && self.tool_rx.is_none() {
                if let Some(cmd) = self.pending_command.take() {
                    self.spawn_tool(cmd);
                }
            }

            if !self.processing && self.state == AppState::Idle && self.streaming_rx.is_none() {
                if let Some(msg) = self.queued_message.take() {
                    self.submit_message(msg);
                }
            }

            if let Some((_, time)) = &self.toast {
                if time.elapsed() > Duration::from_secs(3) {
                    self.toast = None;
                }
            }

            self.frame_count += 1;

            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Mouse(_) => {}
                    Event::Key(key) => {
                        if self.state == AppState::ConfirmQuit {
                            match key.code {
                                KeyCode::Char('y')
                                | KeyCode::Char('Y')
                                | KeyCode::Enter
                                | KeyCode::Tab => {
                                    return Ok(());
                                }
                                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                    self.state = AppState::Idle;
                                }
                                _ => {}
                            }
                            continue;
                        }

                        if self.state == AppState::AwaitingConfirm {
                            match key.code {
                                KeyCode::Char('y')
                                | KeyCode::Char('Y')
                                | KeyCode::Enter
                                | KeyCode::Tab => {
                                    self.state = AppState::ToolRunning;
                                }
                                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                    self.pending_tool = None;
                                    self.pending_command = None;
                                    self.state = AppState::Idle;
                                }
                                _ => {}
                            }
                            continue;
                        }

                        if self.state == AppState::Updating {
                            continue;
                        }

                        if let Err(e) = self.handle_key(key) {
                            self.conversation.push(format!("[system] Error: {}", e));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub(crate) fn submit_message(&mut self, msg: String) {
        let trimmed = msg.trim().to_string();
        if trimmed.is_empty() {
            return;
        }

        if trimmed == "/quit" {
            self.state = AppState::ConfirmQuit;
            return;
        }

        if trimmed == "/island" {
            self.show_island = !self.show_island;
            return;
        }

        if let Some(finding) = trimmed.strip_prefix("/findings ") {
            let finding = finding.trim().to_string();
            if !finding.is_empty() {
                self.findings.push(finding.clone());
                self.conversation
                    .push(format!("[system] Finding tagged: {}", finding));
                self.toast = Some((format!("✓ Finding tagged: {}", finding), Instant::now()));
            }
            return;
        }

        if trimmed == "/update" || trimmed == "/update tools" {
            let update_tools = trimmed == "/update tools";
            self.conversation
                .push("[system] Starting update...".to_string());
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            self.update_output_rx = Some(rx);
            self.state = AppState::Updating;

            async fn run_cmd(
                tx: &tokio::sync::mpsc::UnboundedSender<String>,
                cmd: &mut Command,
            ) -> bool {
                let child = cmd
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .map_err(|e| {
                        let _ = tx.send(format!("Failed to spawn: {}", e));
                    })
                    .ok();
                let mut child = match child {
                    Some(c) => c,
                    None => return false,
                };

                let stdout = child.stdout.take().unwrap();
                let stderr = child.stderr.take().unwrap();
                let mut stdout_reader = BufReader::new(stdout).lines();
                let mut stderr_reader = BufReader::new(stderr).lines();
                let tx_stdout = tx.clone();
                let tx_stderr = tx.clone();

                let out_task = tokio::spawn(async move {
                    while let Ok(Some(line)) = stdout_reader.next_line().await {
                        if !line.is_empty() {
                            let _ = tx_stdout.send(line);
                        }
                    }
                });
                let err_task = tokio::spawn(async move {
                    while let Ok(Some(line)) = stderr_reader.next_line().await {
                        if !line.is_empty() {
                            let _ = tx_stderr.send(format!("  {}", line));
                        }
                    }
                });

                let status = child.wait().await;
                let _ = out_task.await;
                let _ = err_task.await;

                match status {
                    Ok(s) if s.success() => true,
                    Ok(s) => {
                        let code = s
                            .code()
                            .map(|c| c.to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        let _ = tx.send(format!("[system] Exited with code {}", code));
                        false
                    }
                    Err(e) => {
                        let _ = tx.send(format!("[system] Process error: {}", e));
                        false
                    }
                }
            }

            tokio::spawn(async move {
                let _ = tx.send("$ git pull --rebase".to_string());
                let ok = run_cmd(&tx, Command::new("git").args(["pull", "--rebase"])).await;
                if !ok {
                    let _ = tx.send("[system] Update failed at git pull.".to_string());
                    return;
                }

                let _ = tx.send("$ cargo build --release".to_string());
                let ok = run_cmd(&tx, Command::new("cargo").args(["build", "--release"])).await;
                if !ok {
                    let _ = tx.send("[system] Update failed at cargo build.".to_string());
                    return;
                }

                let _ = tx.send("Installing new binary...".to_string());
                let src = std::path::Path::new("target/release/raskolnikov");
                let dst = std::env::current_exe().ok();
                if let (true, Some(dst)) = (src.exists(), dst) {
                    if src.canonicalize().ok().as_deref() != Some(&dst) {
                        if tokio::fs::copy(src, &dst).await.is_ok() {
                            let _ = tx.send(format!("[system] Updated: {}", dst.display()));
                        } else {
                            let local = std::env::var("HOME")
                                .ok()
                                .map(|h| PathBuf::from(h).join(".local/bin/raskolnikov"));
                            if let Some(ref local) = local {
                                let _ = std::fs::create_dir_all(local.parent().unwrap());
                                if tokio::fs::copy(src, local).await.is_ok() {
                                    let _ = tx.send(format!(
                                        "[system] Copied to {}. Add ~/.local/bin to your PATH.",
                                        local.display()
                                    ));
                                } else {
                                    let _ = tx.send(
                                        "[system] Could not install binary. Run manually:\n  sudo cp target/release/raskolnikov <path>"
                                            .to_string(),
                                    );
                                }
                            } else {
                                let _ = tx.send(
                                    "[system] Could not install binary. The new binary is at target/release/raskolnikov"
                                        .to_string(),
                                );
                            }
                        }
                    }
                }

                if update_tools {
                    let _ = tx.send("$ apt-get update && apt-get install...".to_string());
                    #[cfg(target_os = "linux")]
                    {
                        let ok = run_cmd(
                            &tx,
                            Command::new("sh").args([
                                "-c",
                                "sudo apt-get update && sudo apt-get install -y nmap gobuster nikto sqlmap",
                            ]),
                        )
                        .await;
                        if ok {
                            let _ = tx.send("[system] Tools updated.".to_string());
                        } else {
                            let _ = tx.send("[system] Tool update failed.".to_string());
                        }
                    }
                    #[cfg(target_os = "macos")]
                    {
                        let _ = tx
                            .send("Please update tools manually via Homebrew: brew upgrade nmap gobuster nikto sqlmap"
                                .to_string());
                    }
                }

                let _ = tx.send("[update_done]".to_string());
            });
            return;
        }

        if self.processing {
            self.queued_message = Some(trimmed);
            return;
        }

        self.conversation.push(format!("you: {}", trimmed));
        self.messages.push(Message::user(&trimmed));

        if let Some(logger) = &mut self.logger {
            logger.operator_message(&trimmed);
        }

        let ts = Utc::now().format("%H:%M:%S").to_string();
        self.transcript_entries.push(TranscriptEntry::Operator {
            ts,
            content: trimmed,
        });

        self.processing = true;
    }

    fn start_ai_stream(&mut self) {
        let provider = match self.provider.clone() {
            Some(p) => p,
            None => {
                self.conversation.push(
                    "[system] No AI provider. Configure one in config.toml or set API keys."
                        .to_string(),
                );
                self.processing = false;
                return;
            }
        };

        let context_window = self.config.ai.context_window;
        let (needs_summary, total_tokens, ratio) =
            crate::ai::check_context(&self.messages, context_window);
        if needs_summary {
            let summarised = crate::ai::summarise_context(&mut self.messages);
            if summarised > 0 {
                let pct = (ratio * 100.0) as u32;
                let warning = format!(
                    "[system] Context at {}% ({} tokens). {} tool output{} summarised.",
                    pct,
                    total_tokens,
                    summarised,
                    if summarised == 1 { "" } else { "s" },
                );
                self.conversation.push(warning.clone());
                self.toast = Some((warning, Instant::now()));
            }
        }

        let system_prompt = self.agent_shell.build_prompt();
        let history = self.messages.clone();

        let (token_tx, token_rx) = tokio::sync::mpsc::unbounded_channel();
        let (done_tx, done_rx) = tokio::sync::oneshot::channel();
        self.streaming_rx = Some(token_rx);
        self.stream_done_rx = Some(done_rx);
        self.conversation.push("agent: ".to_string());
        self.scroll_offset_conv = self.conversation.len();
        self.auto_scroll = true;

        tokio::spawn(async move {
            let mut ai_messages = Vec::new();
            ai_messages.push(Message::system(system_prompt));
            ai_messages.extend(history);
            match provider.chat_stream(&ai_messages, token_tx).await {
                Ok(response) => {
                    let _ = done_tx.send(Ok(response));
                }
                Err(e) => {
                    let _ = done_tx.send(Err(format!("AI error: {}", e)));
                }
            }
        });
    }

    fn save_session(&mut self) {
        let _ = std::fs::create_dir_all(&self.session_dir);
        let conv_path = self.session_dir.join("conversation.md");
        let findings_path = self.session_dir.join("findings.md");

        let _ = Transcript::write(
            &conv_path,
            &self.session_id,
            &self.model_name,
            &self.provider_name,
            &self.transcript_entries,
        );

        let date = Utc::now().format("%Y-%m-%d").to_string();
        let flags: Vec<crate::session::findings::FlagFinding> = self
            .findings
            .iter()
            .map(|f| crate::session::findings::FlagFinding {
                description: f.clone(),
            })
            .collect();
        let _ = crate::session::findings::FindingsExport::write(
            &findings_path,
            &date,
            &self.model_name,
            &self.provider_name,
            &[],
            &[],
            &flags,
        );

        let tools_dir = self.session_dir.join("tools");
        let _ = std::fs::create_dir_all(&tools_dir);
        for (tool_name, output) in &self.tool_outputs {
            let ext = match tool_name.as_str() {
                "nmap" => "xml",
                _ => "txt",
            };
            let path = tools_dir.join(format!("{}_output.{}", tool_name, ext));
            let _ = std::fs::write(&path, output);
        }

        let log_path = self.session_dir.join("session.log");
        if log_path.exists() {
            let _ = std::fs::remove_file(&log_path);
        }

        let elapsed = self.start_time.elapsed().as_secs();
        if let Some(logger) = &mut self.logger {
            logger.session_end(elapsed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::shell::AgentShell;
    use std::path::PathBuf;

    fn make_app() -> App {
        App::new_with(
            None,
            AgentShell::new(vec!["nmap".to_string()]),
            None,
            config::Config::default(),
            "test-session".to_string(),
            PathBuf::from("/tmp"),
        )
    }

    #[test]
    fn test_new_with_sets_initial_state() {
        let app = make_app();
        assert_eq!(app.state, AppState::Idle);
        assert!(app.input.is_empty());
        assert!(app.conversation.is_empty());
        assert!(app.findings.is_empty());
        assert!(app.auto_scroll);
        assert!(app.show_island);
        assert_eq!(app.tool_count, 0);
    }

    #[test]
    fn test_new_with_sets_names() {
        let app = make_app();
        assert_eq!(app.provider_name, "none");
        assert_eq!(app.model_name, config::Config::default().ai.model);
    }

    #[test]
    fn test_colors_from_config() {
        let cs = config::ColorScheme {
            accent: "green".to_string(),
            surface: "gray".to_string(),
            highlight: "yellow".to_string(),
        };
        let colors = Colors::from_config(&cs);
        assert_eq!(colors.accent, Color::Green);
        assert_eq!(colors.highlight, Color::Yellow);
        assert_eq!(colors.surface, Color::Gray);
    }

    #[test]
    fn test_submit_message_island_toggle() {
        let mut app = make_app();
        assert!(app.show_island);
        app.submit_message("/island".to_string());
        assert!(!app.show_island);
        app.submit_message("/island".to_string());
        assert!(app.show_island);
    }

    #[test]
    fn test_submit_message_findings_tag() {
        let mut app = make_app();
        app.submit_message("/findings open-admin-port".to_string());
        assert_eq!(app.findings.len(), 1);
        assert_eq!(app.findings[0], "open-admin-port");
    }

    #[test]
    fn test_submit_message_empty_noop() {
        let mut app = make_app();
        let before = app.conversation.len();
        app.submit_message("   ".to_string());
        assert_eq!(app.conversation.len(), before);
    }

    #[test]
    fn test_submit_message_sets_processing() {
        let mut app = make_app();
        app.submit_message("scan target".to_string());
        assert!(app.processing);
        assert!(!app.conversation.is_empty());
    }

    #[test]
    fn test_new_with_with_tool_run_result_type() {
        let app = make_app();
        assert!(app.tool_rx.is_none());
        assert!(app.tool_name.is_none());
        assert!(app.tool_output_rx.is_none());
    }

    #[test]
    fn test_save_session_creates_files() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = App::new_with(
            None,
            AgentShell::new(vec![]),
            None,
            config::Config::default(),
            "save-test".to_string(),
            dir.path().to_path_buf(),
        );
        app.conversation.push("hello".to_string());
        app.save_session();
        assert!(dir.path().join("conversation.md").exists());
        assert!(dir.path().join("findings.md").exists());
    }

    #[test]
    fn test_new_with_sets_provider_name() {
        let provider = crate::ai::resolve_provider(&config::Config::default());
        if let Some(p) = provider {
            let app = App::new_with(
                Some(p),
                AgentShell::new(vec![]),
                None,
                config::Config::default(),
                "p-test".to_string(),
                PathBuf::from("/tmp"),
            );
            assert_ne!(app.provider_name, "none");
        }
    }
}
