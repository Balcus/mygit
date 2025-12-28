use std::io;
use ratatui::DefaultTerminal;
use crate::app::app_state::AppState;

pub fn start() -> io::Result<()> {
    let mut terminal: DefaultTerminal = ratatui::init();
    let result = AppState::default().run(&mut terminal);
    ratatui::restore();
    result
}
