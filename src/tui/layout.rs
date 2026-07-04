use crate::tui::app::{App, AppState, PanelFocus};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn create_layout(frame: &Frame) -> Vec<Rect> {
    let size = frame.size();
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(size)
        .to_vec()
}

pub fn render(app: &App, frame: &mut Frame, chunks: Vec<Rect>) {
    render_header(app, frame, chunks[0]);
    render_main_panels(app, frame, chunks[1]);
    render_findings(app, frame, chunks[2]);
    render_input(app, frame, chunks[3]);
}

fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let header = format!(" RASKOLNIKOV  alpha {}  model: {}", VERSION, app.model_name);
    let header_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    let block = Block::default().style(Style::default().bg(Color::Black));

    let paragraph = Paragraph::new(header).style(header_style).block(block);

    frame.render_widget(paragraph, area);
}

fn render_main_panels(app: &App, frame: &mut Frame, area: Rect) {
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_tool_output(app, frame, panels[0]);
    render_conversation(app, frame, panels[1]);
}

fn render_tool_output(app: &App, frame: &mut Frame, area: Rect) {
    let is_focused = app.focus == PanelFocus::ToolOutput;
    let is_running = app.state == AppState::ToolRunning;
    let is_interrupted = app.state == AppState::Interrupted;

    let border_color = if is_focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .title("TOOL OUTPUT")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let content: String = if app.tool_output.is_empty() {
        if is_running {
            " [running...]".to_string()
        } else if is_interrupted {
            " [interrupted]".to_string()
        } else {
            String::new()
        }
    } else {
        let start = app
            .scroll_offset_tool
            .min(app.tool_output.len().saturating_sub(1));
        app.tool_output[start..].join("\n")
    };

    let style = if !is_running && app.tool_output.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default()
    };

    let paragraph = Paragraph::new(content)
        .style(style)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_conversation(app: &App, frame: &mut Frame, area: Rect) {
    let is_focused = app.focus == PanelFocus::Conversation;

    let border_color = if is_focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .title("CONVERSATION")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let content = if app.conversation.is_empty() {
        " Ready. Type anything.".to_string()
    } else {
        let start = app
            .scroll_offset_conv
            .min(app.conversation.len().saturating_sub(1));
        app.conversation[start..].join("\n")
    };

    let paragraph = Paragraph::new(content)
        .style(Style::default())
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_findings(app: &App, frame: &mut Frame, area: Rect) {
    let content = if app.findings.is_empty() {
        " FINDINGS  ·  none yet".to_string()
    } else {
        let mut s = " FINDINGS".to_string();
        for f in &app.findings {
            s.push_str(&format!("  ·  {}", f));
        }
        s
    };

    let block = Block::default().style(Style::default().bg(Color::Black));

    let paragraph = Paragraph::new(content)
        .style(Style::default().fg(Color::Yellow))
        .block(block);

    frame.render_widget(paragraph, area);
}

fn render_input(app: &App, frame: &mut Frame, area: Rect) {
    let prompt = match app.state {
        AppState::ConfirmQuit => {
            " End session? Conversation and findings will be saved. [Y/n] ".to_string()
        }
        AppState::AwaitingConfirm => " Confirm tool? (yes/no/modify) ".to_string(),
        AppState::Interrupted => " [interrupted] Continue? ".to_string(),
        _ => {
            let mut p = format!("> {}", app.input);
            if app.queued_message.is_some() {
                p.push_str(" [1 queued]");
            }
            p
        }
    };

    let style = match app.state {
        AppState::ConfirmQuit => Style::default().fg(Color::Red),
        AppState::ToolRunning => Style::default().fg(Color::DarkGray),
        _ => Style::default(),
    };

    let block = Block::default().style(Style::default().bg(Color::Black));

    let paragraph = Paragraph::new(prompt).style(style).block(block);

    frame.render_widget(paragraph, area);
}
