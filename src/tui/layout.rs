use crate::ai::Role;
use crate::tui::app::{App, AppState};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const SPINNER: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
const BAR_CHARS: &[char] = &['░', '▒', '▓', '█'];

pub fn create_layout(app: &App, frame: &Frame) -> Vec<Rect> {
    let size = frame.size();
    let confirming = app.state == AppState::ConfirmQuit;
    if app.show_island {
        if confirming {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
                .split(size)
                .to_vec()
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
                .split(size)
                .to_vec()
        }
    } else if confirming {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(size)
            .to_vec()
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(size)
            .to_vec()
    }
}

pub fn render(app: &App, frame: &mut Frame) {
    let chunks = crate::tui::layout::create_layout(app, frame);
    let confirming = app.state == AppState::ConfirmQuit;
    if app.show_island {
        render_conversation(app, frame, chunks[0]);
        render_island(app, frame, chunks[1]);
        if confirming {
            render_confirm_quit(frame, chunks[2]);
            render_bar(app, frame, chunks[3]);
            render_input(app, frame, chunks[4]);
        } else {
            render_bar(app, frame, chunks[2]);
            render_input(app, frame, chunks[3]);
        }
    } else if confirming {
        render_conversation(app, frame, chunks[0]);
        render_confirm_quit(frame, chunks[1]);
        render_bar(app, frame, chunks[2]);
        render_input(app, frame, chunks[3]);
    } else {
        render_conversation(app, frame, chunks[0]);
        render_bar(app, frame, chunks[1]);
        render_input(app, frame, chunks[2]);
    }

    if !app.filtered_commands.is_empty() {
        render_command_list(app, frame);
    }

    if !confirming && app.state == AppState::AwaitingConfirm {
        render_confirm_tool(app, frame);
    }

    if app.toast.is_some() {
        render_toast(app, frame, chunks[0]);
    }
}

fn is_streaming(app: &App) -> bool {
    app.streaming_rx.is_some()
}

fn spinner_char(frame_count: u64) -> char {
    SPINNER[frame_count as usize % SPINNER.len()]
}

fn bar_char(frame_count: u64) -> char {
    let idx = (frame_count as usize / 3) % BAR_CHARS.len();
    BAR_CHARS[idx]
}

fn decorate_conversation(slice: &[String], streaming: bool, frame_count: u64) -> Vec<String> {
    let mut v: Vec<String> = slice.to_vec();
    if streaming {
        let sp = spinner_char(frame_count);
        if let Some(last) = v.last_mut() {
            if last.starts_with("agent:") {
                let trimmed = last
                    .trim_end_matches(|c: char| SPINNER.contains(&c) || c == ' ')
                    .to_string();
                let idx = v.len() - 1;
                v[idx] = format!("{} {}", trimmed, sp);
            }
        }
    }
    v
}

fn render_conversation(app: &App, frame: &mut Frame, area: Rect) {
    let content_height = area.height as usize;
    let streaming = is_streaming(app);

    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(app.colors.accent));
    let inner = block.inner(area);

    let raw = if app.conversation.is_empty() {
        vec![" Ready. Type anything to start.".to_string()]
    } else if app.auto_scroll || content_height == 0 {
        let start = app.conversation.len().saturating_sub(inner.height as usize);
        self::decorate_conversation(&app.conversation[start..], streaming, app.frame_count)
    } else {
        let start = app
            .scroll_offset_conv
            .min(app.conversation.len().saturating_sub(1));
        self::decorate_conversation(&app.conversation[start..], streaming, app.frame_count)
    };

    let content = raw.join("\n");

    let paragraph = Paragraph::new(content)
        .style(Style::default())
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, inner);
    frame.render_widget(block, area);
}

fn render_bar(app: &App, frame: &mut Frame, area: Rect) {
    let elapsed = app.start_time.elapsed();
    let h = elapsed.as_secs() / 3600;
    let m = elapsed.as_secs() / 60 % 60;
    let s = elapsed.as_secs() % 60;
    let elapsed_str = format!("{:02}:{:02}:{:02}", h, m, s);

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(app.colors.accent));

    let status = if app.state == AppState::Updating {
        let sp = spinner_char(app.frame_count);
        format!(
            " {}  │  {}  │  {} updating {}",
            app.model_name, elapsed_str, VERSION, sp
        )
    } else if app.state == AppState::ToolRunning {
        let bar = bar_char(app.frame_count);
        format!(
            " {}  │  {}  │  {} {}{}",
            app.model_name, elapsed_str, VERSION, bar, bar
        )
    } else if app.processing || is_streaming(app) {
        let sp = spinner_char(app.frame_count);
        format!(
            " {}  │  {}  │  {} {}",
            app.model_name, elapsed_str, VERSION, sp
        )
    } else {
        format!(" {}  │  {}  │  v{}", app.model_name, elapsed_str, VERSION)
    };

    let findings = if app.findings.is_empty() {
        "none yet".to_string()
    } else {
        app.findings.join("  ·  ")
    };

    let full = format!("{}  │  FINDINGS  ·  {}", status, findings);

    let paragraph = Paragraph::new(full)
        .style(Style::default().fg(app.colors.surface))
        .block(block);

    frame.render_widget(paragraph, area);
}

fn render_input(app: &App, frame: &mut Frame, area: Rect) {
    let pulse_dim = (app.frame_count as usize / 4).is_multiple_of(2);

    let (prefix, text, show_cursor) = match app.state {
        AppState::ConfirmQuit | AppState::AwaitingConfirm => ("> ", app.input.clone(), false),
        AppState::Interrupted => ("> ", "[interrupted] Press Enter".to_string(), false),
        AppState::Updating => ("> ", "[updating...]".to_string(), false),
        _ => {
            let t = if app.queued_message.is_some() {
                format!("{} [queued]", app.input)
            } else {
                app.input.clone()
            };
            ("> ", t, true)
        }
    };
    let text = text.as_str();

    let prefix_style = if pulse_dim {
        Style::default()
            .fg(app.colors.accent)
            .add_modifier(Modifier::DIM)
    } else {
        Style::default().fg(app.colors.accent)
    };

    let text_style = match app.state {
        AppState::Interrupted => Style::default().fg(Color::Red),
        _ => Style::default().fg(Color::White),
    };

    let cursor_offset = prefix.len() + text.len();
    let cursor_x = if show_cursor && area.width > 0 {
        Some(area.x + (cursor_offset as u16).min(area.width.saturating_sub(1)))
    } else {
        None
    };

    let spans = ratatui::text::Line::from(vec![
        ratatui::text::Span::styled(prefix, prefix_style),
        ratatui::text::Span::styled(text, text_style),
    ]);

    let paragraph = Paragraph::new(spans);
    frame.render_widget(paragraph, area);

    if let Some(cx) = cursor_x {
        frame.set_cursor(cx, area.y);
    }
}

fn render_island(app: &App, frame: &mut Frame, area: Rect) {
    let elapsed = app.start_time.elapsed();
    let h = elapsed.as_secs() / 3600;
    let m = elapsed.as_secs() / 60 % 60;
    let s = elapsed.as_secs() % 60;
    let elapsed_str = format!("{:02}:{:02}:{:02}", h, m, s);

    let state_str = match app.state {
        AppState::Idle => "idle",
        AppState::AwaitingConfirm => "await confirm",
        AppState::ToolRunning => "tool",
        AppState::Interrupted => "interrupted",
        AppState::ConfirmQuit => "quit?",
        AppState::Updating => "updating",
    };

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(app.colors.accent));

    let content = format!(
        " {} v{}  │  {}  │  {}  │ {} msgs  {} tools  │ session {}  │ {}",
        app.model_name,
        VERSION,
        app.provider_name,
        state_str,
        app.messages
            .iter()
            .filter(|m| matches!(m.role, Role::User))
            .count(),
        app.tool_count,
        &app.session_id[..app.session_id.len().min(8)],
        elapsed_str,
    );

    let paragraph = Paragraph::new(content)
        .style(Style::default().fg(app.colors.accent))
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn render_command_list(app: &App, frame: &mut Frame) {
    let area = frame.size();
    let max_visible = 10;
    let count = app.filtered_commands.len().min(max_visible);

    let popup_width = area.width.min(60);
    let popup_height = if count > 0 { (count * 2 + 1) as u16 } else { 2 };
    let x = area.x + area.width.saturating_sub(popup_width) / 2;
    let y = area.height.saturating_sub(4 + popup_height);

    let popup_area = Rect::new(x, y, popup_width, popup_height);
    if popup_area.width < 10 || popup_area.height < 3 {
        return;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" commands ")
        .border_style(Style::default().fg(app.colors.accent))
        .style(Style::default().bg(Color::Black));

    let start = app
        .selected_command
        .saturating_sub(max_visible - 1)
        .min(app.filtered_commands.len().saturating_sub(max_visible));

    let mut lines = Vec::new();
    for i in 0..app.filtered_commands.len().min(max_visible) {
        let idx = start + i;
        if idx >= app.filtered_commands.len() {
            break;
        }
        let cmd_idx = app.filtered_commands[idx];
        let cmd = &super::app::COMMANDS[cmd_idx];
        let selected = idx == app.selected_command;

        if i > 0 {
            lines.push(ratatui::text::Line::from(""));
        }

        let prefix = if selected { "▸ " } else { "  " };

        if selected {
            let full = format!("{}{}  —  {}", prefix, cmd.name, cmd.description);
            lines.push(ratatui::text::Line::from(ratatui::text::Span::styled(
                full,
                Style::default().fg(Color::White).bg(app.colors.highlight),
            )));
        } else {
            let name_span = ratatui::text::Span::styled(
                format!("{}{}", prefix, cmd.name),
                Style::default().fg(Color::White),
            );
            let sep_span =
                ratatui::text::Span::styled("  —  ", Style::default().fg(app.colors.surface));
            let desc_span = ratatui::text::Span::styled(
                cmd.description,
                Style::default().fg(app.colors.surface),
            );
            lines.push(ratatui::text::Line::from(vec![
                name_span, sep_span, desc_span,
            ]));
        }
    }

    let paragraph = Paragraph::new(ratatui::text::Text::from(lines)).block(block);

    frame.render_widget(paragraph, popup_area);
}

fn render_toast(app: &App, frame: &mut Frame, area: Rect) {
    let (msg, _) = app.toast.as_ref().unwrap();

    let popup_width = area.width.min(60);
    let popup_height = 3;
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + area.height.saturating_sub(popup_height + 2);

    let popup_area = Rect::new(x, y, popup_width, popup_height);
    if popup_area.width < 10 || popup_area.height < 3 {
        return;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" ✓ ")
        .border_style(Style::default().fg(app.colors.accent))
        .style(Style::default().bg(Color::Black));

    let paragraph = Paragraph::new(msg.as_str())
        .block(block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(app.colors.accent));

    frame.render_widget(paragraph, popup_area);
}

fn render_confirm_quit(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Quit ")
        .border_style(Style::default().fg(Color::Red))
        .style(Style::default().bg(Color::Black));

    let text = Paragraph::new("End session?  [Y/n]")
        .block(block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Red));

    if area.width >= 10 && area.height >= 3 {
        frame.render_widget(text, area);
    }
}

fn render_confirm_tool(app: &App, frame: &mut Frame) {
    let area = frame.size();
    let command = app
        .pending_command
        .as_deref()
        .unwrap_or("(unknown command)");
    let popup_width = area.width.clamp(30, 60);
    let inner_w = popup_width.saturating_sub(4).max(1);
    let lines = (command.len() as u16).div_ceil(inner_w);
    let popup_height = (lines + 7).min(area.height.saturating_sub(6));
    let x = (area.width - popup_width) / 2;
    let y = area.height.saturating_sub(popup_height + 4);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Run tool? ")
        .border_style(Style::default().fg(app.colors.accent))
        .style(Style::default().bg(Color::Black));

    let short = if command.len() > 80 {
        format!("{}…", &command[..77])
    } else {
        command.to_string()
    };

    let text = format!(
        " {} \n\n {} \n\n {} ",
        "Run this tool?", short, "(yes / no)",
    );

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(app.colors.accent));

    let popup_area = Rect::new(x, y, popup_width, popup_height);
    if popup_area.width >= 10 && popup_area.height >= 3 {
        frame.render_widget(paragraph, popup_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_char_cycles() {
        let c0 = spinner_char(0);
        let c1 = spinner_char(1);
        assert_ne!(c0, c1);
        assert!(SPINNER.contains(&c0));
        assert!(SPINNER.contains(&c1));
    }

    #[test]
    fn test_bar_char_cycles() {
        let c0 = bar_char(0);
        let c1 = bar_char(3);
        assert_ne!(c0, c1);
        assert!(BAR_CHARS.contains(&c0));
        assert!(BAR_CHARS.contains(&c1));
    }

    #[test]
    fn test_is_streaming_false_when_none() {
        let app = test_app();
        assert!(!is_streaming(&app));
    }

    #[test]
    fn test_decorate_conversation_not_streaming() {
        let input = vec!["hello".to_string(), "world".to_string()];
        let result = decorate_conversation(&input, false, 0);
        assert_eq!(result, input);
    }

    #[test]
    fn test_decorate_conversation_streaming_appends_spinner() {
        let input = vec!["agent: hello".to_string()];
        let result = decorate_conversation(&input, true, 1);
        assert!(result[0].starts_with("agent: hello"));
        assert!(!result[0].ends_with("hello"));
        assert!(result[0].len() > "agent: hello".len());
    }

    #[test]
    fn test_decorate_conversation_streaming_no_agent_prefix() {
        let input = vec!["not agent line".to_string()];
        let result = decorate_conversation(&input, true, 0);
        assert_eq!(result[0], "not agent line");
    }

    #[test]
    fn test_decorate_conversation_empty() {
        let result = decorate_conversation(&[], true, 0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_constants_not_empty() {
        assert!(!SPINNER.is_empty());
        assert!(!BAR_CHARS.is_empty());
    }

    fn test_app() -> App {
        App::new_with(
            None,
            crate::agent::shell::AgentShell::new(vec![]),
            None,
            crate::config::Config::default(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        )
    }
}
