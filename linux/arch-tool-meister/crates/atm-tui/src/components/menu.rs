use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::action::Action;
use crate::app::{App, MenuState};

pub struct Menu {
    command_tx: Option<UnboundedSender<Action>>,
}

impl Menu {
    pub fn new() -> Self {
        Self { command_tx: None }
    }

    pub fn render_menu(&self, frame: &mut Frame, area: Rect, app: &App) -> Result<()> {
        let status_state = app.get_status_state();

        // Check if we're awaiting confirmation
        if app.is_awaiting_confirmation() {
            self.render_confirmation_prompt(frame, area, app)?;
            return Ok(());
        }

        let chunks = if status_state.visible {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Min(0),    // Content
                    Constraint::Length(3), // Status bar
                    Constraint::Length(3), // Footer
                ])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Min(0),    // Content
                    Constraint::Length(3), // Footer
                ])
                .split(area)
        };

        // Header with title and context
        let title = match app.get_current_menu_state() {
            Some(MenuState::Main) => "Main Menu",
            Some(MenuState::Module(module_name)) => &format!("Module: {}", module_name),
            None => "Main Menu",
        };

        let header = Paragraph::new(format!("🔧 Arch Tool Meister - {}", title))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Navigation")
                    .title_alignment(Alignment::Center)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_widget(header, chunks[0]);

        // Check if we're in loading state
        let loading_state = app.get_loading_state();
        if loading_state.is_loading {
            self.render_loading_screen(frame, chunks[1], app)?;
        } else {
            self.render_menu_content(frame, chunks[1], app)?;
        }

        // Render status bar if visible
        let footer_area = if status_state.visible {
            self.render_status_bar(frame, chunks[2], app)?;
            chunks[3]
        } else {
            chunks[2]
        };

        // Footer with navigation help
        let scroll_state = app.get_scroll_state();
        let footer_text = if loading_state.is_loading {
            "Please wait..."
        } else if scroll_state.content_length > 10 {
            // Show scroll help for longer menus
            if app.get_current_menu_state() == Some(&MenuState::Main) {
                "Navigation: ↑/↓ or 1-9 to select • PgUp/PgDn to scroll • Enter to confirm • q to quit"
            } else {
                "Navigation: ↑/↓ or 1-9 to select • PgUp/PgDn to scroll • Enter to confirm • 0 to go back • q to quit"
            }
        } else if app.get_current_menu_state() == Some(&MenuState::Main) {
            "Navigation: ↑/↓ or 1-9 to select • Enter to confirm • q to quit"
        } else {
            "Navigation: ↑/↓ or 1-9 to select • Enter to confirm • 0 to go back • q to quit"
        };

        let footer = Paragraph::new(footer_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Help")
                    .title_alignment(Alignment::Center)
                    .border_style(Style::default().fg(Color::Green)),
            )
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            );
        frame.render_widget(footer, footer_area);

        Ok(())
    }

    fn render_confirmation_prompt(&self, frame: &mut Frame, area: Rect, app: &App) -> Result<()> {
        // Create a centered confirmation area
        let centered_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .split(area);

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(centered_chunks[1]);

        let confirmation_message = app.get_confirmation_message();
        let result_type = app.get_confirmation_result_type();

        let color = match result_type {
            crate::action::StatusType::Success => Color::Green,
            crate::action::StatusType::Error => Color::Red,
            crate::action::StatusType::Warning => Color::Yellow,
            crate::action::StatusType::Info => Color::Blue,
        };

        let confirmation_text = format!("{}\n\nPress Enter to continue...", confirmation_message);

        let confirmation_paragraph = Paragraph::new(confirmation_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Command Completed")
                    .title_alignment(Alignment::Center)
                    .border_style(Style::default().fg(color)),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .wrap(Wrap { trim: true });

        frame.render_widget(confirmation_paragraph, horizontal_chunks[1]);

        Ok(())
    }

    fn render_status_bar(&self, frame: &mut Frame, area: Rect, app: &App) -> Result<()> {
        let status_state = app.get_status_state();

        let (color, symbol) = match status_state.status_type {
            crate::action::StatusType::Success => (Color::Green, "✓"),
            crate::action::StatusType::Error => (Color::Red, "✗"),
            crate::action::StatusType::Warning => (Color::Yellow, "⚠"),
            crate::action::StatusType::Info => (Color::Blue, "ℹ"),
        };

        let status_text = format!("{} {}", symbol, status_state.message);
        let status_paragraph = Paragraph::new(status_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Status")
                    .title_alignment(Alignment::Center)
                    .border_style(Style::default().fg(color)),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(color).add_modifier(Modifier::BOLD));

        frame.render_widget(status_paragraph, area);
        Ok(())
    }

    fn render_loading_screen(&self, frame: &mut Frame, area: Rect, app: &App) -> Result<()> {
        let loading_state = app.get_loading_state();

        // Create a centered loading area
        let centered_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .split(area);

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .split(centered_chunks[1]);

        // Create loading content
        let spinner = app.get_loading_animation();
        let elapsed = if let Some(start_time) = loading_state.start_time {
            let elapsed = start_time.elapsed().as_secs();
            format!(" ({}s)", elapsed)
        } else {
            String::new()
        };

        let loading_text = if let Some(progress) = loading_state.progress {
            format!(
                "{} Loading... {}%{}\n\n{}",
                spinner, progress, elapsed, loading_state.message
            )
        } else {
            format!(
                "{} Loading{}\n\n{}",
                spinner, elapsed, loading_state.message
            )
        };

        let loading_paragraph = Paragraph::new(loading_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Executing Command")
                    .title_alignment(Alignment::Center)
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan))
            .wrap(Wrap { trim: true });

        frame.render_widget(loading_paragraph, horizontal_chunks[1]);

        // Render progress bar if progress is available
        if let Some(progress) = loading_state.progress {
            let progress_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Min(0),
                ])
                .split(horizontal_chunks[1]);

            let progress_bar = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("Progress"))
                .gauge_style(Style::default().fg(Color::Green))
                .percent(progress as u16);

            frame.render_widget(progress_bar, progress_chunks[1]);
        }

        Ok(())
    }

    fn render_menu_content(&self, frame: &mut Frame, area: Rect, app: &App) -> Result<()> {
        // Check if we're in VSCode module and have version info
        let is_vscode_module = matches!(app.get_current_menu_state(),
            Some(MenuState::Module(module_name)) if module_name == "vscode");

        let version_info = if is_vscode_module {
            app.get_version_info("vscode")
        } else {
            None
        };

        let chunks = if version_info.is_some() {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(6), // Version info
                    Constraint::Min(0),    // Menu options
                ])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0)])
                .split(area)
        };

        // Render version info if available
        if let Some(version_info) = version_info {
            let mut version_text = String::new();

            if version_info.stable_installed {
                if let Some(version) = &version_info.stable_version {
                    version_text.push_str(&format!("✓ VS Code Stable: {}\n", version));
                } else {
                    version_text.push_str("✓ VS Code Stable: Installed\n");
                }
            } else {
                version_text.push_str("✗ VS Code Stable: Not installed\n");
            }

            if version_info.insiders_installed {
                if let Some(version) = &version_info.insiders_version {
                    version_text.push_str(&format!("✓ VS Code Insiders: {}\n", version));
                } else {
                    version_text.push_str("✓ VS Code Insiders: Installed\n");
                }
            } else {
                version_text.push_str("✗ VS Code Insiders: Not installed\n");
            }

            let version_paragraph = Paragraph::new(version_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Installation Status")
                        .title_alignment(Alignment::Center)
                        .border_style(Style::default().fg(Color::Magenta)),
                )
                .style(Style::default().fg(Color::White))
                .wrap(Wrap { trim: true });

            frame.render_widget(version_paragraph, chunks[0]);
        }

        // Get current menu options and scroll state
        let options = app.get_current_menu_options();
        let selected_index = app.get_selected_index();
        let scroll_state = app.get_scroll_state();
        let menu_area = if version_info.is_some() {
            chunks[1]
        } else {
            chunks[0]
        };

        // Calculate visible range based on scroll state
        let menu_height = menu_area.height.saturating_sub(2) as usize; // Account for borders
        let start_index = scroll_state.offset;
        let end_index = (start_index + menu_height).min(options.len());
        let visible_options = &options[start_index..end_index];

        // Create menu items with selection highlighting
        let menu_items: Vec<ListItem> = visible_options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                let actual_index = start_index + i;
                let number = format!("{}. ", actual_index + 1);
                let content = format!("{}{}", number, option);

                if actual_index == selected_index {
                    ListItem::new(content).style(
                        Style::default()
                            .bg(Color::Blue)
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    ListItem::new(content).style(Style::default().fg(Color::White))
                }
            })
            .collect();

        // Create scroll indicators
        let title = if scroll_state.content_length > menu_height {
            format!(
                "Options ({}/{}) {}",
                start_index + 1,
                options.len(),
                if scroll_state.can_scroll_up() || scroll_state.can_scroll_down() {
                    "↑↓"
                } else {
                    ""
                }
            )
        } else {
            "Options".to_string()
        };

        // Render the menu list with enhanced styling
        let menu_list = List::new(menu_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .title_alignment(Alignment::Center)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("→ ")
            .direction(ListDirection::TopToBottom);

        frame.render_widget(menu_list, menu_area);

        Ok(())
    }
}

impl Component for Menu {
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
        // This is a placeholder - the actual rendering will be done via render_menu
        // which has access to the app state
        frame.render_widget(
            Paragraph::new("Menu component - use render_menu() with app state")
                .block(Block::default().borders(Borders::ALL).title("Menu"))
                .alignment(Alignment::Center),
            area,
        );
        Ok(())
    }
}
