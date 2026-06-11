use crate::app::{App, GameMode};
use crate::game::{BOARD_HEIGHT, BOARD_WIDTH};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

const ASCII_BORDER: symbols::border::Set = symbols::border::Set {
    top_left: "+",
    top_right: "+",
    bottom_left: "+",
    bottom_right: "+",
    vertical_left: "|",
    vertical_right: "|",
    horizontal_top: "-",
    horizontal_bottom: "-",
};

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let block_symbol = if app.config.use_fill { "[]" } else { "  " };

    match app.mode {
        GameMode::MainMenu => {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(6),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ])
                .split(area);

            render_ascii_title(frame, layout[0]);

            let items = vec![
                ListItem::new(" START GAME "),
                ListItem::new(" OPTIONS "),
                ListItem::new(" LEADERBOARD "),
                ListItem::new(" QUIT "),
            ];
            let list = List::new(items)
                .block(
                    Block::default()
                        .title(" MAIN MENU ")
                        .title_alignment(Alignment::Center)
                        .borders(Borders::ALL)
                        .border_set(ASCII_BORDER)
                        .border_style(Style::default().fg(Color::Cyan)),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::Indexed(236))
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");

            let menu_area = centered_rect_fixed(20, 6, area);
            frame.render_stateful_widget(list, menu_area, &mut app.menu_state.clone());
        }
        GameMode::Options => {
            let items = vec![
                ListItem::new(format!(
                    " Ghost: [{}] ",
                    if app.config.show_ghost { "On" } else { "Off" }
                )),
                ListItem::new(format!(
                    " Style: [{}] ",
                    if app.config.use_fill { "[]" } else { "  " }
                )),
                ListItem::new(" BACK "),
            ];
            let list = List::new(items)
                .block(
                    Block::default()
                        .title(" OPTIONS ")
                        .title_alignment(Alignment::Center)
                        .borders(Borders::ALL)
                        .border_set(ASCII_BORDER)
                        .border_style(Style::default().fg(Color::Yellow)),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::Indexed(236))
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");

            let options_area = centered_rect_fixed(20, 5, area);
            frame.render_stateful_widget(list, options_area, &mut app.options_state.clone());
        }
        GameMode::Leaderboard => {
            let items: Vec<ListItem> = app
                .leaderboard
                .iter()
                .take(10)
                .enumerate()
                .map(|(i, (name, score))| {
                    ListItem::new(format!("{}. {:<15} {}", i + 1, name, score))
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title(" LEADERBOARD ")
                        .title_alignment(Alignment::Center)
                        .borders(Borders::ALL)
                        .border_set(ASCII_BORDER)
                        .border_style(Style::default().fg(Color::Green)),
                )
                .style(Style::default().fg(Color::White));

            let board_area = centered_rect_fixed(30, 12, area);
            frame.render_widget(list, board_area);

            let footer = Paragraph::new("Press Esc to return")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(
                footer,
                Rect {
                    x: board_area.x,
                    y: board_area.y + board_area.height,
                    width: board_area.width,
                    height: 1,
                },
            );
        }
        _ => {
            // GAME UI
            let outer_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(6), Constraint::Min(0)])
                .split(area);

            render_ascii_title(frame, outer_layout[0]);

            let game_area = outer_layout[1];
            // Layout: Stats (16) | Board (22) | Next (12)
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(18),
                    Constraint::Length(22),
                    Constraint::Length(14),
                    Constraint::Min(0),
                ])
                .split(game_area);

            // LEFT: Stats
            let stats_area = layout[0];
            let stats_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // Score
                    Constraint::Length(1), // High Score
                    Constraint::Length(1), // Spacer
                    Constraint::Length(8), // Controls
                    Constraint::Min(0),
                ])
                .split(stats_area);

            // Subtle Score Display
            let score_line = Line::from(vec![
                Span::styled(
                    " SCORE ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(app.score.to_string(), Style::default().fg(Color::Yellow)),
            ]);
            frame.render_widget(Paragraph::new(score_line), stats_layout[0]);

            // Subtle High Score Display
            let high_score = app.leaderboard.first().map(|(_, s)| *s).unwrap_or(0);
            let high_line = Line::from(vec![
                Span::styled(
                    " HIGH  ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(high_score.to_string(), Style::default().fg(Color::Yellow)),
            ]);
            frame.render_widget(Paragraph::new(high_line), stats_layout[1]);

            // Controls (no border, compact)
            let ctrl_text = vec![
                Line::from(" ←/→: Move"),
                Line::from(" ↑  : Rotate"),
                Line::from(" ↓  : Drop"),
                Line::from(" Spc: Hard"),
                Line::from(" p  : Pause"),
                Line::from(" r  : Restart"),
                Line::from(" m  : Menu"),
                Line::from(" q  : Quit"),
            ];
            frame.render_widget(
                Paragraph::new(ctrl_text).style(Style::default().fg(Color::Gray)),
                stats_layout[3],
            );

            // CENTER: Game Board
            let game_width = (BOARD_WIDTH * 2 + 2) as u16;
            let game_height = (BOARD_HEIGHT + 2) as u16;
            let game_rect = Rect {
                x: layout[1].x,
                y: layout[1].y,
                width: game_width,
                height: game_height,
            };
            frame.render_widget(
                Block::default()
                    .borders(Borders::ALL)
                    .border_set(ASCII_BORDER)
                    .border_style(Style::default().fg(Color::White)),
                game_rect,
            );

            // RIGHT: Next Piece Box
            let next_area_rect = Rect {
                x: layout[2].x,
                y: layout[2].y,
                width: 14,
                height: 6,
            };
            frame.render_widget(
                Block::default()
                    .title(" NEXT ")
                    .borders(Borders::ALL)
                    .border_set(ASCII_BORDER)
                    .border_style(Style::default().fg(Color::White)),
                next_area_rect,
            );
            render_next_piece(frame, next_area_rect, &app.next_piece, block_symbol);

            let inner_rect = Rect {
                x: game_rect.x + 1,
                y: game_rect.y + 1,
                width: game_width - 2,
                height: game_height - 2,
            };

            for y in 0..BOARD_HEIGHT {
                for x in 0..BOARD_WIDTH {
                    if let Some(color) = app.board[y][x] {
                        frame.render_widget(
                            Paragraph::new(block_symbol)
                                .style(Style::default().bg(color).fg(Color::Black)),
                            Rect {
                                x: inner_rect.x + (x * 2) as u16,
                                y: inner_rect.y + y as u16,
                                width: 2,
                                height: 1,
                            },
                        );
                    }
                }
            }

            if app.mode != GameMode::EnteringName
                && !app.game_over()
                && app.clearing_lines.is_none()
            {
                if app.config.show_ghost {
                    for (x, y) in app.get_ghost_piece().global_blocks() {
                        if y >= 0 && y < BOARD_HEIGHT as i32 && x >= 0 && x < BOARD_WIDTH as i32 {
                            frame.render_widget(
                                Paragraph::new("[]").style(Style::default().fg(Color::DarkGray)),
                                Rect {
                                    x: inner_rect.x + (x * 2) as u16,
                                    y: inner_rect.y + y as u16,
                                    width: 2,
                                    height: 1,
                                },
                            );
                        }
                    }
                }
                for (x, y) in app.current_piece.global_blocks() {
                    if y >= 0 && y < BOARD_HEIGHT as i32 && x >= 0 && x < BOARD_WIDTH as i32 {
                        frame.render_widget(
                            Paragraph::new(block_symbol).style(
                                Style::default()
                                    .bg(app.current_piece.shape.color())
                                    .fg(Color::Black),
                            ),
                            Rect {
                                x: inner_rect.x + (x * 2) as u16,
                                y: inner_rect.y + y as u16,
                                width: 2,
                                height: 1,
                            },
                        );
                    }
                }
            }

            // MODALS
            if app.mode == GameMode::Paused {
                render_modal(frame, " PAUSED ", " (p) Resume | (m) Menu ", Color::Blue);
            } else if app.mode == GameMode::ConfirmingRestart {
                render_modal(frame, " RESTART? ", " (y)es | (n)o ", Color::Yellow);
            } else if app.mode == GameMode::EnteringName {
                let block = Block::default()
                    .title(" New High Score! ")
                    .borders(Borders::ALL)
                    .border_set(ASCII_BORDER)
                    .border_style(Style::default().fg(Color::Green));
                let area = centered_rect_fixed(40, 7, area);
                frame.render_widget(Clear, area);
                let text = vec![
                    Line::from(format!(" Score: {}", app.score)),
                    Line::from(format!(" Name:  {}", app.current_input)),
                    Line::from(" (Enter to submit) "),
                ];
                frame.render_widget(Paragraph::new(text).block(block), area);
            } else if app.mode == GameMode::GameOver {
                render_modal(
                    frame,
                    " GAME OVER ",
                    " (r)estart | (m)enu | (q)uit ",
                    Color::Red,
                );
            }
        }
    }
}

fn render_modal(frame: &mut Frame, title: &str, msg: &str, color: Color) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_set(ASCII_BORDER)
        .border_style(Style::default().fg(color));

    let area = centered_rect_fixed(40, 5, frame.area()); // Fixed size: 40 wide, 5 high
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(format!("\n{}", msg))
            .alignment(Alignment::Center)
            .block(block),
        area,
    );
}

fn render_ascii_title(frame: &mut Frame, area: Rect) {
    let ascii = vec![
        Line::from(vec![
            Span::styled("  _____ ", Style::default().fg(Color::Red)),
            Span::styled("_____ ", Style::default().fg(Color::Green)),
            Span::styled("_____ ", Style::default().fg(Color::Yellow)),
            Span::styled("____  ", Style::default().fg(Color::Blue)),
            Span::styled("___ ", Style::default().fg(Color::Magenta)),
            Span::styled("____ ", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled(" |_   _|", Style::default().fg(Color::Red)),
            Span::styled(" ____|", Style::default().fg(Color::Green)),
            Span::styled("_   _|", Style::default().fg(Color::Yellow)),
            Span::styled("  _ \\", Style::default().fg(Color::Blue)),
            Span::styled("|_ _/", Style::default().fg(Color::Magenta)),
            Span::styled(" ___|", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("   | | ", Style::default().fg(Color::Red)),
            Span::styled("|  _|  ", Style::default().fg(Color::Green)),
            Span::styled(" | | ", Style::default().fg(Color::Yellow)),
            Span::styled("| |_) |", Style::default().fg(Color::Blue)),
            Span::styled("| |", Style::default().fg(Color::Magenta)),
            Span::styled("\\___ \\", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("    | | ", Style::default().fg(Color::Red)),
            Span::styled("| |___ ", Style::default().fg(Color::Green)),
            Span::styled(" | | ", Style::default().fg(Color::Yellow)),
            Span::styled("|  _ < ", Style::default().fg(Color::Blue)),
            Span::styled("| |", Style::default().fg(Color::Magenta)),
            Span::styled(" ___) |", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("   |_| ", Style::default().fg(Color::Red)),
            Span::styled("|_____|", Style::default().fg(Color::Green)),
            Span::styled(" |_| ", Style::default().fg(Color::Yellow)),
            Span::styled("|_| \\_\\", Style::default().fg(Color::Blue)),
            Span::styled("___|", Style::default().fg(Color::Magenta)),
            Span::styled("____/", Style::default().fg(Color::Cyan)),
        ]),
    ];
    frame.render_widget(Paragraph::new(ascii).alignment(Alignment::Center), area);
}

fn render_next_piece(frame: &mut Frame, area: Rect, piece: &crate::game::Piece, symbol: &str) {
    let color = piece.shape.color();
    for (bx, by) in piece.blocks {
        let x = area.x + 3 + (bx * 2) as u16; // Adjusted for 14-width box
        let y = area.y + 2 + by as u16;
        frame.render_widget(
            Paragraph::new(symbol).style(Style::default().bg(color).fg(Color::Black)),
            Rect {
                x,
                y,
                width: 2,
                height: 1,
            },
        );
    }
}

fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.x + r.width.saturating_sub(width) / 2;
    let y = r.y + r.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width: width.min(r.width),
        height: height.min(r.height),
    }
}
