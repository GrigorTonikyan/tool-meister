use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::action::Action;

#[derive(Default)]
pub struct Home {
    command_tx: Option<UnboundedSender<Action>>,
}

impl Home {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for Home {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                // add any logic here that should run on every tick
            }
            Action::Render => {
                // add any logic here that should run on every render
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        // Create the main layout with header, content, and footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Header
        let header = Paragraph::new("Arch Tool Meister - Rust TUI")
            .block(Block::default().borders(Borders::ALL).title("Main Menu"))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(header, chunks[0]);

        // Content area - for now show a placeholder
        // TODO: This will be replaced with actual menu rendering logic
        let content_lines = vec![
            Line::from("Welcome to Arch Tool Meister!"),
            Line::from(""),
            Line::from("This is a modular TUI for managing Arch Linux tools."),
            Line::from(""),
            Line::from("Loading menu..."),
        ];

        let content = Paragraph::new(content_lines)
            .block(Block::default().borders(Borders::ALL).title("Content"))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(content, chunks[1]);

        // Footer with navigation help
        let footer = Paragraph::new(
            "Navigation: ↑/↓ or numbers to select, Enter to confirm, 0 to go back, q to quit",
        )
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
        frame.render_widget(footer, chunks[2]);

        Ok(())
    }
}
