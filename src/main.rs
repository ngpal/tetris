mod app;
mod game;
mod ui;
mod config;

use std::{io, time::{Duration, Instant}};
use ratatui::crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use ratatui::{TerminalOptions, Viewport};
use clap::Parser;
use crate::app::{App, GameMode};
use crate::game::BOARD_HEIGHT;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    alt_screen: bool,

    #[arg(short, long)]
    name: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut terminal = setup_terminal(args.alt_screen)?;
    let app_result = run(&mut terminal, args.name);
    restore_terminal(&mut terminal, args.alt_screen)?;
    
    if let Ok(res) = app_result {
        // Ensure we print after the inline viewport space
        if !args.alt_screen {
            for _ in 0..(BOARD_HEIGHT + 2) { println!(); }
        }
        println!("\x1b[1;36m [ LEADERBOARD ]\x1b[0m");
        for (i, (name, score)) in res.leaderboard.iter().take(10).enumerate() {
            let color = match i {
                0 => "\x1b[1;33m", // Gold
                1 => "\x1b[1;37m", // Silver
                2 => "\x1b[1;34m", // Bronze
                _ => "\x1b[0;90m", // Gray
            };
            println!("  {}{}. {:<15} {}\x1b[0m", color, i + 1, name, score);
        }
        println!("\x1b[36m -----------------------------\x1b[0m\n");
    }

    Ok(())
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, name: Option<String>) -> io::Result<App> {
    let mut app = App::new(name);

    while app.running {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        let timeout = if app.clearing_lines.is_some() || app.mode != GameMode::Playing {
            Duration::from_millis(50)
        } else {
            app.tick_rate.saturating_sub(app.last_tick.elapsed())
        };

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.mode {
                        GameMode::MainMenu => match key.code {
                            KeyCode::Up => app.menu_prev(),
                            KeyCode::Down => app.menu_next(),
                            KeyCode::Enter => app.menu_select(),
                            KeyCode::Char('q') => app.quit(),
                            _ => {}
                        },
                        GameMode::Options => match key.code {
                            KeyCode::Up => app.options_prev(),
                            KeyCode::Down => app.options_next(),
                            KeyCode::Enter => app.options_select(),
                            KeyCode::Esc => app.mode = GameMode::MainMenu,
                            _ => {}
                        },
                        GameMode::Leaderboard => match key.code {
                            KeyCode::Esc | KeyCode::Enter => app.mode = GameMode::MainMenu,
                            _ => {}
                        },
                        GameMode::Playing => match key.code {
                            KeyCode::Char('q') => app.quit(),
                            KeyCode::Char('p') | KeyCode::Esc => app.toggle_pause(),
                            KeyCode::Char('r') => app.mode = GameMode::ConfirmingRestart,
                            KeyCode::Char('m') => app.mode = GameMode::MainMenu,
                            KeyCode::Left => app.move_left(),
                            KeyCode::Right => app.move_right(),
                            KeyCode::Up => app.rotate(),
                            KeyCode::Down => app.tick(),
                            KeyCode::Char(' ') => app.hard_drop(),
                            _ => {}
                        },
                        GameMode::Paused => match key.code {
                            KeyCode::Char('p') | KeyCode::Esc => app.toggle_pause(),
                            KeyCode::Char('m') => app.mode = GameMode::MainMenu,
                            KeyCode::Char('q') => app.quit(),
                            _ => {}
                        },
                        GameMode::ConfirmingRestart => match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => app.reset_for_game(),
                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.mode = GameMode::Playing,
                            _ => {}
                        },
                        GameMode::EnteringName => match key.code {
                            KeyCode::Enter => app.submit_score(),
                            KeyCode::Char(c) => app.current_input.push(c),
                            KeyCode::Backspace => { app.current_input.pop(); }
                            _ => {}
                        },
                        GameMode::GameOver => match key.code {
                            KeyCode::Char('q') => app.quit(),
                            KeyCode::Char('r') => app.reset_for_game(),
                            KeyCode::Char('m') | KeyCode::Esc => app.mode = GameMode::MainMenu,
                            _ => {}
                        }
                    }
                }
            }
        }

        if app.mode == GameMode::Playing {
            if app.clearing_lines.is_some() {
                app.tick();
            } else if app.last_tick.elapsed() >= app.tick_rate {
                app.tick();
                app.last_tick = Instant::now();
            }
        }
    }

    Ok(app)
}

fn setup_terminal(alt: bool) -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    if alt {
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        Terminal::new(backend)
    } else {
        // Use inline viewport to stay integrated with console
        let options = TerminalOptions {
            viewport: Viewport::Inline(BOARD_HEIGHT as u16 + 8),
        };
        let backend = CrosstermBackend::new(stdout);
        Terminal::with_options(backend, options)
    }
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, alt: bool) -> io::Result<()> {
    disable_raw_mode()?;
    if alt {
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    } else {
        // For inline mode, we don't clear, we just show the cursor
        terminal.show_cursor()?;
    }
    Ok(())
}
