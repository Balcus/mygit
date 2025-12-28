use ratatui::Frame;
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::io;
use std::time::Duration;

pub struct AppState {
    pub menus: Vec<String>,
    pub selected_menu: usize,
    pub exit: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            menus: vec!["Staging Area".to_string(), "History".to_string()],
            selected_menu: 0,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(frame.area());

        let header = Paragraph::new("Flux VCS 🌀").block(
            Block::default()
                .borders(Borders::ALL)
                .title_alignment(ratatui::layout::Alignment::Center),
        );
        frame.render_widget(header, vertical[0]);

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(20), Constraint::Min(0)])
            .split(vertical[1]);

        let mut menu_text = String::from("\n\n");
        for (i, menu) in self.menus.iter().enumerate() {
            if i == self.selected_menu {
                menu_text.push_str(&format!("> {}\n", menu));
            } else {
                menu_text.push_str(&format!("  {}\n", menu));
            }
        }

        let side_menu =
            Paragraph::new(menu_text).block(Block::default().borders(Borders::ALL).title("Menu"));
        frame.render_widget(side_menu, horizontal[0]);

        let main_content = self.get_menu_content();
        let main = Paragraph::new(main_content).block(
            Block::default()
                .borders(Borders::ALL)
                .title(self.menus[self.selected_menu].as_str()),
        );
        frame.render_widget(main, horizontal[1]);
    }

    fn get_menu_content(&self) -> String {
        match self.selected_menu {
            0 => self.render_staging_area(),
            1 => self.render_history(),
            _ => "Unknown page".to_string(),
        }
    }

    fn render_staging_area(&self) -> String {
        "Repository Status:\n\n\
         Modified files:\n\
         • src/main.rs\n\
         • README.md\n\n\
         Untracked files:\n\
         • temp.txt"
            .to_string()
    }

    fn render_history(&self) -> String {
        "Commit History:\n\n\
         • abc123 - Initial commit\n\
         • def456 - Added new feature\n\
         • ghi789 - Fixed bug\n\
         • jkl012 - Updated documentation"
            .to_string()
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => self.exit = true,
                    KeyCode::Up | KeyCode::Char('k') => {
                        if self.selected_menu > 0 {
                            self.selected_menu -= 1;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if self.selected_menu < self.menus.len() - 1 {
                            self.selected_menu += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
