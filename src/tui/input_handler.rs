use crate::tui::app::{App, AppState, COMMANDS};
use crossterm::event::{self, KeyCode, KeyModifiers};

impl App {
    fn command_query(&self) -> &str {
        if self.input.starts_with('/') {
            let rest = self.input[1..].trim();
            if rest.is_empty() {
                ""
            } else {
                rest
            }
        } else {
            ""
        }
    }

    fn fuzzy_score(query: &str, name: &str) -> usize {
        if query.is_empty() {
            return 1;
        }
        let qb = query.as_bytes();
        let nb = name.as_bytes();
        if query.eq_ignore_ascii_case(name) {
            return 100;
        }
        if name.len() >= query.len()
            && nb[..query.len()]
                .iter()
                .zip(qb)
                .all(|(a, b)| a.eq_ignore_ascii_case(b))
        {
            return 80;
        }
        let q_lower: Vec<u8> = qb.iter().map(|b| b.to_ascii_lowercase()).collect();
        let n_lower: Vec<u8> = nb.iter().map(|b| b.to_ascii_lowercase()).collect();
        if n_lower.windows(q_lower.len()).any(|w| w == q_lower) {
            return 60;
        }
        let mut qi = q_lower.iter();
        let mut qc = qi.next();
        for nc in &n_lower {
            if Some(nc) == qc {
                qc = qi.next();
            }
        }
        if qc.is_none() {
            return 40;
        }
        0
    }

    fn update_command_filter(&mut self) {
        let query = self.command_query();
        if query.is_empty() && self.input.starts_with('/') {
            self.filtered_commands = (0..COMMANDS.len()).collect();
        } else if query.is_empty() {
            self.filtered_commands.clear();
        } else {
            let mut scored: Vec<(usize, usize)> = COMMANDS
                .iter()
                .enumerate()
                .map(|(i, cmd)| (i, Self::fuzzy_score(query, cmd.name)))
                .filter(|&(_, s)| s > 0)
                .collect();
            scored.sort_by_key(|k| std::cmp::Reverse(k.1));
            self.filtered_commands = scored.into_iter().map(|(i, _)| i).collect();
        }
        self.selected_command = 0;
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
                    self.conversation.clear();
                    self.scroll_offset_conv = 0;
                    Ok(())
                }
                KeyCode::Char('o') => {
                    self.show_island = !self.show_island;
                    Ok(())
                }
                _ => Ok(()),
            };
        }

        match key.code {
            KeyCode::Tab => {
                if self.input.starts_with('/') && !self.filtered_commands.is_empty() {
                    if let Some(&idx) = self.filtered_commands.get(self.selected_command) {
                        let cmd = &COMMANDS[idx];
                        let base = cmd.name.split_whitespace().next().unwrap_or(cmd.name);
                        self.input = format!("{} ", base);
                        self.filtered_commands.clear();
                    }
                }
                Ok(())
            }
            KeyCode::Enter => {
                if self.input.starts_with('/') && !self.filtered_commands.is_empty() {
                    if let Some(&idx) = self.filtered_commands.get(self.selected_command) {
                        let cmd = &COMMANDS[idx];
                        let base = cmd.name.split_whitespace().next().unwrap_or(cmd.name);
                        self.input = format!("{} ", base);
                        self.filtered_commands.clear();
                    }
                    return Ok(());
                }

                let input = std::mem::take(&mut self.input);
                if input.trim().is_empty() {
                    return Ok(());
                }

                match self.state {
                    AppState::Idle => {
                        self.input_history.insert(0, input.clone());
                        if self.input_history.len() > 100 {
                            self.input_history.pop();
                        }
                        self.history_index = None;
                        self.auto_scroll = true;
                        self.submit_message(input);
                        Ok(())
                    }
                    AppState::AwaitingConfirm => Ok(()),
                    AppState::Interrupted => {
                        self.state = AppState::Idle;
                        let _ = self.interrupt_tx.send(false);
                        self.auto_scroll = true;
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
                if self.input.starts_with('/') {
                    self.update_command_filter();
                } else {
                    self.filtered_commands.clear();
                }
                Ok(())
            }
            KeyCode::Backspace => {
                self.input.pop();
                if self.input.starts_with('/') {
                    self.update_command_filter();
                } else {
                    self.filtered_commands.clear();
                }
                Ok(())
            }
            KeyCode::Up => {
                if self.input.starts_with('/') && !self.filtered_commands.is_empty() {
                    if self.selected_command > 0 {
                        self.selected_command -= 1;
                    } else {
                        self.selected_command = self.filtered_commands.len() - 1;
                    }
                } else if self.state == AppState::Idle && !self.input_history.is_empty() {
                    if self.history_index.is_none() {
                        self.history_saved = self.input.clone();
                        self.history_index = Some(0);
                    } else if let Some(idx) = self.history_index {
                        let next = idx + 1;
                        if next < self.input_history.len() {
                            self.history_index = Some(next);
                        }
                    }
                    if let Some(idx) = self.history_index {
                        if idx < self.input_history.len() {
                            self.input = self.input_history[idx].clone();
                        }
                    }
                }
                Ok(())
            }
            KeyCode::Down => {
                if self.input.starts_with('/') && !self.filtered_commands.is_empty() {
                    if self.selected_command + 1 < self.filtered_commands.len() {
                        self.selected_command += 1;
                    } else {
                        self.selected_command = 0;
                    }
                } else if self.state == AppState::Idle {
                    if let Some(idx) = self.history_index {
                        if idx == 0 {
                            self.history_index = None;
                            self.input = std::mem::take(&mut self.history_saved);
                        } else {
                            self.history_index = Some(idx - 1);
                            self.input = self.input_history[idx - 1].clone();
                        }
                    }
                }
                Ok(())
            }
            KeyCode::PageUp => {
                self.auto_scroll = false;
                self.scroll_offset_conv = self.scroll_offset_conv.saturating_sub(10);
                self.scroll_offset_conv = self
                    .scroll_offset_conv
                    .min(self.conversation.len().saturating_sub(1));
                Ok(())
            }
            KeyCode::PageDown => {
                if self.scroll_offset_conv >= self.conversation.len().saturating_sub(10) {
                    self.auto_scroll = true;
                }
                self.scroll_offset_conv = self.scroll_offset_conv.saturating_add(10);
                self.scroll_offset_conv = self
                    .scroll_offset_conv
                    .min(self.conversation.len().saturating_sub(1));
                Ok(())
            }
            KeyCode::Esc => {
                if !self.filtered_commands.is_empty() {
                    self.filtered_commands.clear();
                    let _ = std::mem::take(&mut self.input);
                } else {
                    let _input = std::mem::take(&mut self.input);
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::shell::AgentShell;
    use crate::config;
    use crate::tui::app::App;
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

    fn key(code: KeyCode) -> event::KeyEvent {
        event::KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl(code: KeyCode) -> event::KeyEvent {
        event::KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    #[test]
    fn test_handle_key_ctrl_c_idle_triggers_confirm_quit() {
        let mut app = make_app();
        app.state = AppState::Idle;
        app.handle_key(ctrl(KeyCode::Char('c'))).unwrap();
        assert_eq!(app.state, AppState::ConfirmQuit);
    }

    #[test]
    fn test_handle_key_ctrl_c_tool_running_interrupts() {
        let mut app = make_app();
        app.state = AppState::ToolRunning;
        app.handle_key(ctrl(KeyCode::Char('c'))).unwrap();
        assert_eq!(app.state, AppState::Interrupted);
    }

    #[test]
    fn test_handle_key_ctrl_l_clears_conversation() {
        let mut app = make_app();
        app.conversation.push("hello".to_string());
        app.scroll_offset_conv = 5;
        app.handle_key(ctrl(KeyCode::Char('l'))).unwrap();
        assert!(app.conversation.is_empty());
        assert_eq!(app.scroll_offset_conv, 0);
    }

    #[test]
    fn test_handle_key_ctrl_o_toggles_island() {
        let mut app = make_app();
        app.show_island = true;
        app.handle_key(ctrl(KeyCode::Char('o'))).unwrap();
        assert!(!app.show_island);
        app.handle_key(ctrl(KeyCode::Char('o'))).unwrap();
        assert!(app.show_island);
    }

    #[test]
    fn test_handle_key_enter_submits_message() {
        let mut app = make_app();
        app.state = AppState::Idle;
        app.input = "hello".to_string();
        app.handle_key(key(KeyCode::Enter)).unwrap();
        assert!(app.input.is_empty());
        assert!(app.processing);
    }

    #[test]
    fn test_handle_key_enter_empty_input_noop() {
        let mut app = make_app();
        app.state = AppState::Idle;
        app.input = "   ".to_string();
        app.handle_key(key(KeyCode::Enter)).unwrap();
        assert!(!app.processing);
    }

    #[test]
    fn test_handle_key_enter_awaiting_confirm_noop() {
        let mut app = make_app();
        app.state = AppState::AwaitingConfirm;
        app.input = "yes".to_string();
        app.handle_key(key(KeyCode::Enter)).unwrap();
        assert_eq!(app.state, AppState::AwaitingConfirm);
    }

    #[test]
    fn test_handle_key_confirm_quit_handled_in_run_loop() {
        let mut app = make_app();
        app.state = AppState::ConfirmQuit;
        app.input = "test".to_string();
        app.handle_key(key(KeyCode::Enter)).unwrap();
        assert_eq!(app.state, AppState::ConfirmQuit);
    }

    #[test]
    fn test_handle_key_char_appends_to_input() {
        let mut app = make_app();
        app.handle_key(key(KeyCode::Char('a'))).unwrap();
        assert_eq!(app.input, "a");
        app.handle_key(key(KeyCode::Char('b'))).unwrap();
        assert_eq!(app.input, "ab");
    }

    #[test]
    fn test_handle_key_backspace_removes_char() {
        let mut app = make_app();
        app.input = "abc".to_string();
        app.handle_key(key(KeyCode::Backspace)).unwrap();
        assert_eq!(app.input, "ab");
    }

    #[test]
    fn test_handle_key_page_up_scrolls() {
        let mut app = make_app();
        app.conversation = vec!["a".to_string(); 50];
        app.scroll_offset_conv = 30;
        app.auto_scroll = true;
        app.handle_key(key(KeyCode::PageUp)).unwrap();
        assert_eq!(app.scroll_offset_conv, 20);
        assert!(!app.auto_scroll);
    }

    #[test]
    fn test_handle_key_page_down_scrolls() {
        let mut app = make_app();
        app.conversation = vec!["a".to_string(); 50];
        app.scroll_offset_conv = 10;
        app.handle_key(key(KeyCode::PageDown)).unwrap();
        assert_eq!(app.scroll_offset_conv, 20);
    }

    #[test]
    fn test_handle_key_esc_clears_input() {
        let mut app = make_app();
        app.input = "/findings test".to_string();
        app.handle_key(key(KeyCode::Esc)).unwrap();
        assert!(app.input.is_empty());
    }

    #[test]
    fn test_handle_key_up_down_history() {
        let mut app = make_app();
        app.state = AppState::Idle;
        app.input_history = vec!["nmap".to_string(), "gobuster".to_string()];
        app.input = String::new();

        app.handle_key(key(KeyCode::Up)).unwrap();
        assert_eq!(app.input, "nmap");
        app.handle_key(key(KeyCode::Up)).unwrap();
        assert_eq!(app.input, "gobuster");
        app.handle_key(key(KeyCode::Down)).unwrap();
        assert_eq!(app.input, "nmap");
    }

    #[test]
    fn test_handle_key_up_empty_history_noop() {
        let mut app = make_app();
        app.state = AppState::Idle;
        app.input_history.clear();
        let before = app.input.clone();
        app.handle_key(key(KeyCode::Up)).unwrap();
        assert_eq!(app.input, before);
    }

    #[test]
    fn test_fuzzy_score_exact() {
        assert_eq!(App::fuzzy_score("/quit", "/quit"), 100);
    }

    #[test]
    fn test_fuzzy_score_prefix() {
        assert_eq!(App::fuzzy_score("/qu", "/quit"), 80);
    }

    #[test]
    fn test_fuzzy_score_substring() {
        assert_eq!(App::fuzzy_score("uit", "/quit"), 60);
    }

    #[test]
    fn test_fuzzy_score_subsequence() {
        assert_eq!(App::fuzzy_score("qt", "/quit"), 40);
    }

    #[test]
    fn test_fuzzy_score_no_match() {
        assert_eq!(App::fuzzy_score("xyz", "/quit"), 0);
    }

    #[test]
    fn test_fuzzy_score_empty_query() {
        assert_eq!(App::fuzzy_score("", "/quit"), 1);
    }

    #[test]
    fn test_fuzzy_score_case_insensitive() {
        assert_eq!(App::fuzzy_score("/QUIT", "/quit"), 100);
        assert_eq!(App::fuzzy_score("/QU", "/quit"), 80);
    }

    #[test]
    fn test_command_query_slash() {
        let mut app = make_app();
        app.input = "/findings test".to_string();
        assert_eq!(app.command_query(), "findings test");
    }

    #[test]
    fn test_command_query_bare_slash() {
        let mut app = make_app();
        app.input = "/".to_string();
        assert_eq!(app.command_query(), "");
    }

    #[test]
    fn test_command_query_no_slash() {
        let mut app = make_app();
        app.input = "hello".to_string();
        assert_eq!(app.command_query(), "");
    }

    #[test]
    fn test_update_command_filter_shows_all_on_bare_slash() {
        let mut app = make_app();
        app.input = "/".to_string();
        app.update_command_filter();
        assert_eq!(app.filtered_commands.len(), COMMANDS.len());
    }

    #[test]
    fn test_update_command_filter_filters() {
        let mut app = make_app();
        app.input = "/find".to_string();
        app.update_command_filter();
        assert!(app.filtered_commands.len() > 0);
        let idx = app.filtered_commands[0];
        assert_eq!(COMMANDS[idx].name, "/findings <tag>");
    }

    #[test]
    fn test_update_command_filter_empty_no_slash_clears() {
        let mut app = make_app();
        app.input = "noslash".to_string();
        app.update_command_filter();
        assert!(app.filtered_commands.is_empty());
    }

    #[test]
    fn test_handle_key_tab_completes_command() {
        let mut app = make_app();
        app.input = "/".to_string();
        app.update_command_filter();
        app.selected_command = 0;
        app.handle_key(key(KeyCode::Tab)).unwrap();
        assert!(app.input.starts_with('/'));
    }

    #[test]
    fn test_handle_key_slash_up_down_navigates_commands() {
        let mut app = make_app();
        app.input = "/".to_string();
        app.update_command_filter();
        let initial = app.selected_command;
        app.handle_key(key(KeyCode::Down)).unwrap();
        assert_eq!(app.selected_command, (initial + 1) % COMMANDS.len());
        app.handle_key(key(KeyCode::Up)).unwrap();
        assert_eq!(app.selected_command, initial);
    }

    #[test]
    fn test_handle_key_char_slash_triggers_filter() {
        let mut app = make_app();
        app.handle_key(key(KeyCode::Char('/'))).unwrap();
        assert!(!app.filtered_commands.is_empty());
    }
}
