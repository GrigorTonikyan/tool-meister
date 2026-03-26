mod app;
mod fuzzy;
mod memory;
mod monitor;
mod process;
mod ui;

use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton,
        MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::CrosstermBackend, Terminal};

use app::{App, AppMode, InputTarget};

fn main() -> io::Result<()> {
    // ── Setup terminal ───────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // ── Run ──────────────────────────────────────────────────────────────
    let result = run(&mut terminal);

    // ── Restore terminal ─────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

/// Map a mouse Y coordinate to a row index within the main content area.
///
/// Layout:
///   rows 0..2  = header (3 lines)
///   rows 3..   = main content area (bordered block + table header)
///   last 3     = status bar
///
/// Inside the main content block:
///   +1 for the border top
///   +1 for the table/list header row (in ProcessList, ProcessDetail maps)
///
/// Returns `None` if the click is outside the content rows.
fn mouse_y_to_row(y: u16, term_height: u16, mode: &AppMode) -> Option<usize> {
    let header_height: u16 = 3;
    let status_height: u16 = 3;
    let content_start = header_height + 1; // +1 for block border

    // In ProcessDetail the top pane is 10 rows, so maps table starts at header+10
    let (data_start, data_end) = match mode {
        AppMode::ProcessList => {
            // +1 more for the table header row
            (
                content_start + 1,
                term_height.saturating_sub(status_height + 1),
            )
        }
        AppMode::ProcessDetail => {
            // Detail has 10-row info pane, then maps table with border+header
            let maps_start = header_height + 10 + 1 + 1; // info pane + border + table header
            (maps_start, term_height.saturating_sub(status_height + 1))
        }
        AppMode::StringView => {
            // List items directly inside bordered block
            (content_start, term_height.saturating_sub(status_height + 1))
        }
        AppMode::MonitorMode => {
            // Monitor has 4-row info banner, then event list with border
            let events_start = header_height + 4 + 1; // banner + border
            (events_start, term_height.saturating_sub(status_height + 1))
        }
        AppMode::MemoryInspect => {
            // Memory inspector is just a table inside bordered block
            (content_start, term_height.saturating_sub(status_height + 1))
        }
    };

    if y >= data_start && y < data_end {
        Some((y - data_start) as usize)
    } else {
        None
    }
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut app = App::new();
    app.status_msg = format!("{} processes loaded", app.processes.len());

    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();
    let mut last_click_time = Instant::now();
    let mut last_click_row: Option<usize> = None;
    let double_click_threshold = Duration::from_millis(400);

    loop {
        // ── Draw ─────────────────────────────────────────────────────────
        terminal.draw(|f| ui::draw(f, &app))?;

        // ── Events ───────────────────────────────────────────────────────
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            let ev = event::read()?;

            match ev {
                // ── Keyboard events ──────────────────────────────────────
                Event::Key(key) => {
                    // Input mode
                    if app.input_target != InputTarget::None {
                        match key.code {
                            KeyCode::Enter => app.finish_input(),
                            KeyCode::Esc => app.cancel_input(),
                            KeyCode::Backspace => {
                                app.input_buf.pop();
                            }
                            KeyCode::Char(c) => app.input_buf.push(c),
                            _ => {}
                        }
                        continue;
                    }

                    // Global keys
                    match key.code {
                        KeyCode::Char('q') => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.should_quit = true;
                        }
                        _ => {}
                    }

                    if app.should_quit {
                        break;
                    }

                    // Mode-specific keys
                    match app.mode {
                        AppMode::ProcessList => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                            KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                            KeyCode::Left | KeyCode::Char('h') => {
                                if app.col_resize_mode {
                                    app.narrow_col();
                                } else {
                                    app.scroll_left();
                                }
                            }
                            KeyCode::Right | KeyCode::Char('l') => {
                                if app.col_resize_mode {
                                    app.widen_col();
                                } else {
                                    app.scroll_right();
                                }
                            }
                            KeyCode::PageUp => app.page_up(),
                            KeyCode::PageDown => app.page_down(),
                            KeyCode::Home => app.process_scroll = 0,
                            KeyCode::End => {
                                let max = app.filtered_processes().len().saturating_sub(1);
                                app.process_scroll = max;
                            }
                            KeyCode::Enter => app.select_process(),
                            KeyCode::Char('s') => app.cycle_sort(),
                            KeyCode::Char('S') => {
                                app.toggle_sort_dir();
                            }
                            KeyCode::Char('/') => app.start_input(InputTarget::Filter),
                            KeyCode::Char('c') => app.toggle_col_resize(),
                            KeyCode::Char('+') | KeyCode::Char('=') => {
                                if app.col_resize_mode {
                                    app.widen_col();
                                }
                            }
                            KeyCode::Char('-') => {
                                if app.col_resize_mode {
                                    app.narrow_col();
                                }
                            }
                            KeyCode::Tab => {
                                if app.col_resize_mode {
                                    app.cycle_active_col();
                                }
                            }
                            KeyCode::Char('m') => app.enter_monitor(),
                            KeyCode::Char('r') => app.refresh_processes(),
                            KeyCode::Esc => {
                                if app.col_resize_mode {
                                    app.col_resize_mode = false;
                                    app.status_msg = "Column resize mode off".into();
                                } else if !app.filter_text.is_empty() {
                                    app.filter_text.clear();
                                    app.process_scroll = 0;
                                    app.status_msg = "Filter cleared".into();
                                }
                            }
                            _ => {}
                        },

                        AppMode::ProcessDetail => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                            KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                            KeyCode::Left | KeyCode::Char('h') => app.scroll_left(),
                            KeyCode::Right | KeyCode::Char('l') => app.scroll_right(),
                            KeyCode::PageUp => app.page_up(),
                            KeyCode::PageDown => app.page_down(),
                            KeyCode::Char('x') => app.extract_strings(),
                            KeyCode::Char('L') => app.start_input(InputTarget::MinLen),
                            KeyCode::Enter => {
                                if let Some(pid) = app.selected_pid {
                                    if let Some(map) = app.detail_maps.get(app.detail_scroll) {
                                        app.enter_memory_inspect(pid, map.start);
                                    }
                                }
                            }
                            KeyCode::Esc => app.go_back(),
                            _ => {}
                        },

                        AppMode::StringView => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                            KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                            KeyCode::Left | KeyCode::Char('h') => app.scroll_left(),
                            KeyCode::Right | KeyCode::Char('l') => app.scroll_right(),
                            KeyCode::PageUp => app.page_up(),
                            KeyCode::PageDown => app.page_down(),
                            KeyCode::Home => app.strings_scroll = 0,
                            KeyCode::End => {
                                let max = app.filtered_indices.len().saturating_sub(1);
                                app.strings_scroll = max;
                            }
                            KeyCode::Char('/') => app.start_input(InputTarget::Search),
                            KeyCode::Char('n') => app.next_match(),
                            KeyCode::Char('N') => app.prev_match(),
                            KeyCode::Enter => {
                                // Jump to string address
                                if let Some(pid) = app.selected_pid {
                                    // Need to find the string match and its region
                                    let idx = app.filtered_indices.get(app.strings_scroll).copied();
                                    if let Some(idx) = idx {
                                        if let Some(s) = app.strings.get(idx) {
                                            // Region start?
                                            if let Some(region) = app.detail_maps.get(s.region_idx)
                                            {
                                                let addr = region.start + s.offset_in_region;
                                                app.enter_memory_inspect(pid, addr);
                                            }
                                        }
                                    }
                                }
                            }
                            KeyCode::Esc => app.go_back(),
                            _ => {}
                        },

                        AppMode::MonitorMode => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                            KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                            KeyCode::Left | KeyCode::Char('h') => app.scroll_left(),
                            KeyCode::Right | KeyCode::Char('l') => app.scroll_right(),
                            KeyCode::Enter => app.monitor_select(),
                            KeyCode::Esc => app.go_back(),
                            _ => {}
                        },

                        AppMode::MemoryInspect => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => app.memory_scroll_up(),
                            KeyCode::Down | KeyCode::Char('j') => app.memory_scroll_down(),
                            KeyCode::PageUp => app.memory_page_up(),
                            KeyCode::PageDown => app.memory_page_down(),
                            KeyCode::Char('g') => app.start_input(InputTarget::Address),
                            KeyCode::Esc => app.go_back(),
                            _ => {}
                        },
                    }
                }

                // ── Mouse events ─────────────────────────────────────────
                Event::Mouse(mouse) => {
                    // Dismiss input overlay on any click outside
                    if app.input_target != InputTarget::None {
                        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
                            app.cancel_input();
                        }
                        continue;
                    }

                    let term_height = terminal.size()?.height;

                    match mouse.kind {
                        // ── Left click: select row ───────────────────────
                        MouseEventKind::Down(MouseButton::Left) => {
                            if let Some(row) = mouse_y_to_row(mouse.row, term_height, &app.mode) {
                                // Detect double-click
                                let is_double = last_click_row == Some(row)
                                    && last_click_time.elapsed() < double_click_threshold;

                                match app.mode {
                                    AppMode::ProcessList => {
                                        let max = app.filtered_processes().len().saturating_sub(1);
                                        let target = row.min(max);
                                        app.process_scroll = target;
                                        if is_double {
                                            app.select_process();
                                        }
                                    }
                                    AppMode::ProcessDetail => {
                                        let max = app.detail_maps.len().saturating_sub(1);
                                        app.detail_scroll = row.min(max);
                                    }
                                    AppMode::StringView => {
                                        let max = app.filtered_indices.len().saturating_sub(1);
                                        app.strings_scroll = row.min(max);
                                    }
                                    AppMode::MonitorMode => {
                                        let max = app.monitor_events.len().saturating_sub(1);
                                        let target = row.min(max);
                                        app.monitor_scroll = target;
                                        if is_double {
                                            app.monitor_select();
                                        }
                                    }
                                    AppMode::MemoryInspect => {
                                        // Scroll by row? Usually address is continuous.
                                        // Just ignore
                                    }
                                }

                                last_click_row = Some(row);
                                last_click_time = Instant::now();
                            }
                        }

                        // ── Right click: go back ─────────────────────────
                        MouseEventKind::Down(MouseButton::Right) => {
                            app.go_back();
                        }

                        // ── Scroll wheel ─────────────────────────────────
                        MouseEventKind::ScrollUp => {
                            app.scroll_up();
                            app.scroll_up();
                            app.scroll_up();
                        }
                        MouseEventKind::ScrollDown => {
                            app.scroll_down();
                            app.scroll_down();
                            app.scroll_down();
                        }

                        _ => {}
                    }
                }

                _ => {}
            }
        }

        // ── Tick ─────────────────────────────────────────────────────────
        if last_tick.elapsed() >= tick_rate {
            app.tick();
            last_tick = Instant::now();
        }
    }

    Ok(())
}
