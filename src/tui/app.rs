use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;

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
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Idle,
            focus: PanelFocus::Conversation,
            input: String::new(),
            queued_message: None,
            conversation: Vec::new(),
            tool_output: Vec::new(),
            findings: Vec::new(),
            scroll_offset_tool: 0,
            scroll_offset_conv: 0,
        }
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

            if self.queued_message.is_some() && self.state != AppState::ToolRunning {
                // Tool finished — inject queued message
            }
        }
    }

    pub fn handle_key(&mut self, key: event::KeyEvent) -> Result<(), String> {
        if key.modifiers.contains(event::KeyModifiers::CONTROL) {
            return match key.code {
                KeyCode::Char('c') => {
                    if self.state == AppState::ToolRunning {
                        self.state = AppState::Interrupted;
                        self.conversation
                            .push("[system] Tool interrupted by operator".to_string());
                    } else {
                        self.state = AppState::ConfirmQuit;
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

                if self.state == AppState::ToolRunning {
                    self.queued_message = Some(input);
                    return Ok(());
                }

                self.conversation.push(format!("you: {}", input));
                self.input.clear();
                self.scroll_offset_conv = 0;
                Ok(())
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
