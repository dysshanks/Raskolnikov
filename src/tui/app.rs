use crate::agent::shell::AgentShell;
use crate::ai::{Message, Provider, ProviderKind};
use crate::session::logger::SessionLogger;
use crate::session::transcript::{Transcript, TranscriptEntry};
use crate::tools::executor::ToolRunResult;
use chrono::Utc;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::watch;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    Idle,
    AwaitingConfirm,
    ToolRunning,
    Interrupted,
    ConfirmQuit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelFocus {
    ToolOutput,
    Conversation,
}

pub struct App {
    pub state: AppState,
    pub focus: PanelFocus,
    pub input: String,
    pub queued_message: Option<String>,
    pub conversation: Vec<String>,
    pub tool_output: Vec<String>,
    pub findings: Vec<String>,
    pub scroll_offset_tool: usize,
    pub scroll_offset_conv: usize,
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
        let model_name = config.ai.model.clone();
        let provider_name = provider
            .as_ref()
            .map(|p| p.name().to_string())
            .unwrap_or_else(|| "none".to_string());
        let (interrupt_tx, _) = watch::channel(false);

        let mut app = Self {
            state: AppState::Idle,
            focus: PanelFocus::Conversation,
            input: String::new(),
            queued_message: None,
            conversation: Vec::new(),
            tool_output: Vec::new(),
            findings: Vec::new(),
            scroll_offset_tool: 0,
            scroll_offset_conv: 0,
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
        };

        if let Some(logger) = &mut app.logger {
            logger.session_start(&app.model_name, &app.provider_name);
        }

        app
    }

    pub async fn run(&mut self) {
        enable_raw_mode().expect("Failed to enable raw mode");
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .expect("Failed to enter alternate screen");

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).expect("Failed to create terminal");

        let res = self.run_loop(&mut terminal).await;

        disable_raw_mode().expect("Failed to disable raw mode");
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
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
                let chunks = crate::tui::layout::create_layout(f);
                crate::tui::layout::render(self, f, chunks);
            })?;

            self.check_tool_completion().await;

            if self.processing && self.state != AppState::ToolRunning {
                self.process_ai().await;
            }

            if self.state == AppState::ToolRunning && self.tool_rx.is_none() {
                if let Some(cmd) = self.pending_command.take() {
                    self.spawn_tool(cmd);
                }
            }

            if !self.processing && self.state == AppState::Idle {
                if let Some(msg) = self.queued_message.take() {
                    self.submit_message(msg);
                }
            }

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if self.state == AppState::ConfirmQuit {
                        match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                                return Ok(());
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                self.state = AppState::Idle;
                            }
                            _ => {}
                        }
                        continue;
                    }

                    if let Err(e) = self.handle_key(key) {
                        self.conversation.push(format!("[system] Error: {}", e));
                    }
                }
            }
        }
    }

    fn submit_message(&mut self, msg: String) {
        let trimmed = msg.trim().to_string();
        if trimmed.is_empty() {
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

    async fn process_ai(&mut self) {
        self.processing = false;

        let provider = match &self.provider {
            Some(p) => p,
            None => {
                self.conversation.push(
                    "[system] No AI provider. Configure one in config.toml or set API keys."
                        .to_string(),
                );
                return;
            }
        };

        let mut ai_messages = Vec::new();
        ai_messages.push(Message::system(self.agent_shell.build_prompt()));
        ai_messages.extend(self.messages.clone());

        match provider.chat(&ai_messages).await {
            Ok(response) => {
                let content = response.content.trim().to_string();
                if content.is_empty() {
                    return;
                }

                self.conversation.push(format!("agent: {}", content));
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
            Err(e) => {
                let err_msg = format!("AI error: {}", e);
                self.conversation.push(format!("[system] {}", err_msg));
            }
        }
    }

    fn parse_tool_suggestion(&mut self, response: &str) {
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

    fn spawn_tool(&mut self, command: String) {
        let tool_name = self
            .pending_tool
            .take()
            .unwrap_or_else(|| command.split_whitespace().next().unwrap_or("?").to_string());

        if let Some(logger) = &mut self.logger {
            logger.tool_start(&tool_name, &command);
        }

        self.conversation
            .push(format!("[system] Running: {}", command));

        let interrupt_rx = self.interrupt_tx.subscribe();
        let cmd_clone = command.clone();

        let (tx, rx) = tokio::sync::oneshot::channel();
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

        self.tool_rx = Some(rx);
        self.tool_name = Some(tool_name);
        self.state = AppState::ToolRunning;
    }

    async fn check_tool_completion(&mut self) {
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
        let tool = self.tool_name.take().unwrap_or_default();

        if !result.stdout.is_empty() {
            self.tool_output.push(result.stdout.clone());
        }
        if !result.stderr.is_empty() {
            self.tool_output.push(format!("stderr: {}", result.stderr));
        }

        if let Some(logger) = &mut self.logger {
            logger.tool_end(
                &tool,
                result.exit_code.unwrap_or(-1),
                result.duration.as_secs(),
            );
        }

        let ts = Utc::now().format("%H:%M:%S").to_string();
        let output = format!("stdout:\n{}\nstderr:\n{}", result.stdout, result.stderr);
        self.transcript_entries.push(TranscriptEntry::Tool {
            ts,
            tool: tool.clone(),
            duration: result.duration.as_secs(),
            output: output.clone(),
        });

        if result.was_interrupted {
            self.state = AppState::Interrupted;
            self.conversation
                .push("[system] Tool interrupted".to_string());
        } else {
            let code = result.exit_code.unwrap_or(-1);
            let secs = result.duration.as_secs();
            let output_msg = format!(
                "Tool `{}` finished (exit code {}, {}s):\n{}",
                tool, code, secs, output
            );
            self.messages.push(Message::tool(output_msg, &tool));
            self.state = AppState::Idle;
            self.processing = true;
        }
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
        let _ = crate::session::findings::FindingsExport::write(
            &findings_path,
            &date,
            &self.model_name,
            &self.provider_name,
            &[],
            &[],
            &[],
        );

        let elapsed = self.start_time.elapsed().as_secs();
        if let Some(logger) = &mut self.logger {
            logger.session_end(elapsed);
        }
    }

    pub fn handle_key(&mut self, key: event::KeyEvent) -> Result<(), String> {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return match key.code {
                KeyCode::Char('c') => {
                    match self.state {
                        AppState::ToolRunning => {
                            let _ = self.interrupt_tx.send(true);
                            self.state = AppState::Interrupted;
                            self.conversation
                                .push("[system] Tool interrupted by operator".to_string());
                        }
                        _ => {
                            self.state = AppState::ConfirmQuit;
                        }
                    }
                    Ok(())
                }
                KeyCode::Char('l') => {
                    self.tool_output.clear();
                    self.scroll_offset_tool = 0;
                    Ok(())
                }
                _ => Ok(()),
            };
        }

        match key.code {
            KeyCode::Enter => {
                let input = std::mem::take(&mut self.input);
                if input.trim().is_empty() {
                    return Ok(());
                }

                if input.trim() == "/quit" {
                    self.state = AppState::ConfirmQuit;
                    return Ok(());
                }

                match self.state {
                    AppState::Idle => {
                        self.submit_message(input);
                        Ok(())
                    }
                    AppState::AwaitingConfirm => {
                        let trimmed = input.trim().to_lowercase();
                        if trimmed == "yes" || trimmed == "y" || trimmed == "go ahead" {
                            self.state = AppState::ToolRunning;
                            Ok(())
                        } else {
                            self.pending_tool = None;
                            self.pending_command = None;
                            self.conversation.push(format!("you: {}", input));
                            self.state = AppState::Idle;
                            self.submit_message(input);
                            Ok(())
                        }
                    }
                    AppState::Interrupted => {
                        self.state = AppState::Idle;
                        let _ = self.interrupt_tx.send(false);
                        if !input.trim().is_empty() {
                            self.submit_message(input);
                        }
                        Ok(())
                    }
                    AppState::ToolRunning => {
                        self.queued_message = Some(input);
                        Ok(())
                    }
                    AppState::ConfirmQuit => Ok(()),
                }
            }
            KeyCode::Char(c) => {
                self.input.push(c);
                Ok(())
            }
            KeyCode::Backspace => {
                self.input.pop();
                Ok(())
            }
            KeyCode::Tab => {
                self.focus = match self.focus {
                    PanelFocus::ToolOutput => PanelFocus::Conversation,
                    PanelFocus::Conversation => PanelFocus::ToolOutput,
                };
                Ok(())
            }
            KeyCode::PageUp => {
                match self.focus {
                    PanelFocus::ToolOutput => {
                        self.scroll_offset_tool = self.scroll_offset_tool.saturating_add(10);
                    }
                    PanelFocus::Conversation => {
                        self.scroll_offset_conv = self.scroll_offset_conv.saturating_add(10);
                    }
                }
                Ok(())
            }
            KeyCode::PageDown => {
                match self.focus {
                    PanelFocus::ToolOutput => {
                        self.scroll_offset_tool = self.scroll_offset_tool.saturating_sub(10);
                    }
                    PanelFocus::Conversation => {
                        self.scroll_offset_conv = self.scroll_offset_conv.saturating_sub(10);
                    }
                }
                Ok(())
            }
            KeyCode::Esc => {
                let _input = std::mem::take(&mut self.input);
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
