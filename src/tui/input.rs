use crossterm::event::{self, Event};

pub fn handle_key_input(app: &mut crate::tui::app::App) -> Result<(), String> {
    if let Event::Key(key) = event::read().map_err(|e| e.to_string())? {
        app.handle_key(key)?;
    }
    Ok(())
}
