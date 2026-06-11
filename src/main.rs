use std::{
    collections::HashMap,
    fs,
    io,
    time::{Duration, Instant},
};

use ratatui::crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;
const LEADERBOARD_FILE: &str = "leaderboard.txt";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

impl Shape {
    fn blocks(&self) -> [(i32, i32); 4] {
        match self {
            Shape::I => [(0, 1), (1, 1), (2, 1), (3, 1)],
            Shape::J => [(0, 0), (0, 1), (1, 1), (2, 1)],
            Shape::L => [(2, 0), (0, 1), (1, 1), (2, 1)],
            Shape::O => [(1, 0), (2, 0), (1, 1), (2, 1)],
            Shape::S => [(1, 0), (2, 0), (0, 1), (1, 1)],
            Shape::T => [(1, 0), (0, 1), (1, 1), (2, 1)],
            Shape::Z => [(0, 0), (1, 0), (1, 1), (2, 1)],
        }
    }

    fn color(&self) -> Color {
        match self {
            Shape::I => Color::Cyan,
            Shape::J => Color::Blue,
            Shape::L => Color::Indexed(208), // Orange
            Shape::O => Color::Yellow,
            Shape::S => Color::Green,
            Shape::T => Color::Magenta,
            Shape::Z => Color::Red,
        }
    }

    fn all() -> [Shape; 7] {
        [
            Shape::I,
            Shape::J,
            Shape::L,
            Shape::O,
            Shape::S,
            Shape::T,
            Shape::Z,
        ]
    }
}

#[derive(Clone)]
struct Piece {
    shape: Shape,
    pos: (i32, i32),
    blocks: [(i32, i32); 4],
}

impl Piece {
    fn new(shape: Shape) -> Self {
        let blocks = shape.blocks();
        Self {
            shape,
            pos: (BOARD_WIDTH as i32 / 2 - 2, 0),
            blocks,
        }
    }

    fn rotate(&mut self) {
        if self.shape == Shape::O {
            return;
        }
        for block in &mut self.blocks {
            let x = block.0;
            let y = block.1;
            block.0 = 2 - y;
            block.1 = x;
        }
    }

    fn rotated(&self) -> Self {
        let mut next = self.clone();
        next.rotate();
        next
    }

    fn global_blocks(&self) -> [(i32, i32); 4] {
        let mut out = [(0, 0); 4];
        for i in 0..4 {
            out[i] = (self.pos.0 + self.blocks[i].0, self.pos.1 + self.blocks[i].1);
        }
        out
    }
}

#[derive(PartialEq, Eq)]
enum GameMode {
    Playing,
    Paused,
    ConfirmingRestart,
    EnteringName,
    GameOver,
}

struct App {
    running: bool,
    mode: GameMode,
    board: [[Option<Color>; BOARD_WIDTH]; BOARD_HEIGHT],
    current_piece: Piece,
    score: u32,
    leaderboard: HashMap<String, u32>,
    last_name: String,
    current_input: String,
    last_tick: Instant,
    tick_rate: Duration,
    clearing_lines: Option<(u32, Instant)>,
}

impl App {
    fn new() -> Self {
        let (leaderboard, last_name) = Self::load_leaderboard();
        let mut app = Self {
            running: true,
            mode: GameMode::Playing,
            board: [[None; BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece: Piece::new(Shape::I), // placeholder
            score: 0,
            leaderboard,
            last_name,
            current_input: String::new(),
            last_tick: Instant::now(),
            tick_rate: Duration::from_millis(800),
            clearing_lines: None,
        };
        app.spawn_piece();
        app
    }

    fn load_leaderboard() -> (HashMap<String, u32>, String) {
        let mut board = HashMap::new();
        let mut last_name = String::new();
        if let Ok(content) = fs::read_to_string(LEADERBOARD_FILE) {
            let lines: Vec<&str> = content.lines().collect();
            if let Some(first) = lines.first() {
                if first.starts_with("LAST_NAME:") {
                    last_name = first.replace("LAST_NAME:", "").trim().to_string();
                }
            }
            for line in lines {
                if line.starts_with("LAST_NAME:") { continue; }
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    if let Ok(score) = parts[1].trim().parse() {
                        board.insert(parts[0].trim().to_string(), score);
                    }
                }
            }
        }
        (board, last_name)
    }

    fn save_leaderboard(&self) {
        let mut results = vec![format!("LAST_NAME:{}", self.last_name)];
        for (name, score) in &self.leaderboard {
            results.push(format!("{}:{}", name, score));
        }
        let _ = fs::write(LEADERBOARD_FILE, results.join("\n"));
    }

    fn reset(&mut self) {
        let (leaderboard, last_name) = Self::load_leaderboard();
        *self = Self::new();
        self.leaderboard = leaderboard;
        self.last_name = last_name;
    }

    fn toggle_pause(&mut self) {
        if self.mode == GameMode::Playing {
            self.mode = GameMode::Paused;
        } else if self.mode == GameMode::Paused {
            self.mode = GameMode::Playing;
            self.last_tick = Instant::now(); // Reset tick timing on resume
        }
    }

    fn spawn_piece(&mut self) {
        let shape = Shape::all()[rand::random::<usize>() % 7];
        self.current_piece = Piece::new(shape);
        if self.is_collision(&self.current_piece) {
            self.mode = GameMode::GameOver;
            self.check_high_score();
        }
    }

    fn check_high_score(&mut self) {
        let is_high = self.leaderboard.get(&self.last_name).map_or(true, |&s| self.score > s) 
            || self.leaderboard.len() < 10 
            || self.score > *self.leaderboard.values().min().unwrap_or(&0);
        
        if is_high && self.score > 0 {
            self.mode = GameMode::EnteringName;
            self.current_input = self.last_name.clone();
        }
    }

    fn submit_score(&mut self) {
        let name = if self.current_input.trim().is_empty() {
            if self.last_name.is_empty() { "Player".to_string() } else { self.last_name.clone() }
        } else {
            self.current_input.trim().to_string()
        } ;

        self.last_name = name.clone();
        
        let entry = self.leaderboard.entry(name).or_insert(0);
        if self.score > *entry {
            *entry = self.score;
        }

        self.save_leaderboard();
        self.mode = GameMode::GameOver;
    }

    fn is_collision(&self, piece: &Piece) -> bool {
        for (x, y) in piece.global_blocks() {
            if x < 0 || x >= BOARD_WIDTH as i32 || y >= BOARD_HEIGHT as i32 {
                return true;
            }
            if y >= 0 && self.board[y as usize][x as usize].is_some() {
                return true;
            }
        }
        false
    }

    fn lock_piece(&mut self) {
        for (x, y) in self.current_piece.global_blocks() {
            if y >= 0 && y < BOARD_HEIGHT as i32 && x >= 0 && x < BOARD_WIDTH as i32 {
                self.board[y as usize][x as usize] = Some(self.current_piece.shape.color());
            }
        }
        self.start_clear_lines();
    }

    fn start_clear_lines(&mut self) {
        let mut lines_to_clear = 0;
        for y in 0..BOARD_HEIGHT {
            if (0..BOARD_WIDTH).all(|x| self.board[y][x].is_some()) {
                lines_to_clear += 1;
                for x in 0..BOARD_WIDTH {
                    self.board[y][x] = Some(Color::White);
                }
            }
        }

        if lines_to_clear > 0 {
            self.clearing_lines = Some((lines_to_clear, Instant::now()));
        } else {
            self.spawn_piece();
        }
    }

    fn finalize_clear_lines(&mut self) {
        let mut new_board = [[None; BOARD_WIDTH]; BOARD_HEIGHT];
        let mut new_y = BOARD_HEIGHT - 1;

        for y in (0..BOARD_HEIGHT).rev() {
            let full = (0..BOARD_WIDTH).all(|x| self.board[y][x] == Some(Color::White));
            if !full {
                new_board[new_y] = self.board[y];
                if new_y > 0 {
                    new_y -= 1;
                }
            }
        }

        self.board = new_board;
        if let Some((count, _)) = self.clearing_lines {
            self.score += count * 100;
        }
        self.clearing_lines = None;
        self.spawn_piece();
    }

    fn tick(&mut self) {
        if self.mode != GameMode::Playing { return; }

        if let Some((_, start)) = self.clearing_lines {
            if start.elapsed() >= Duration::from_millis(300) {
                self.finalize_clear_lines();
            }
            return;
        }

        let mut next_piece = self.current_piece.clone();
        next_piece.pos.1 += 1;

        if self.is_collision(&next_piece) {
            self.lock_piece();
        } else {
            self.current_piece = next_piece;
        }
    }

    fn move_left(&mut self) {
        if self.mode != GameMode::Playing || self.clearing_lines.is_some() { return; }
        let mut next = self.current_piece.clone();
        next.pos.0 -= 1;
        if !self.is_collision(&next) {
            self.current_piece = next;
        }
    }

    fn move_right(&mut self) {
        if self.mode != GameMode::Playing || self.clearing_lines.is_some() { return; }
        let mut next = self.current_piece.clone();
        next.pos.0 += 1;
        if !self.is_collision(&next) {
            self.current_piece = next;
        }
    }

    fn rotate(&mut self) {
        if self.mode != GameMode::Playing || self.clearing_lines.is_some() { return; }
        let next = self.current_piece.rotated();
        if !self.is_collision(&next) {
            self.current_piece = next;
        }
    }

    fn hard_drop(&mut self) {
        if self.mode != GameMode::Playing || self.clearing_lines.is_some() { return; }
        while !self.game_over() {
            let mut next = self.current_piece.clone();
            next.pos.1 += 1;
            if self.is_collision(&next) {
                self.lock_piece();
                break;
            }
            self.current_piece = next;
        }
    }

    fn game_over(&self) -> bool {
        self.mode == GameMode::GameOver || self.mode == GameMode::EnteringName
    }

    fn get_ghost_piece(&self) -> Piece {
        let mut ghost = self.current_piece.clone();
        while !self.is_collision(&ghost) {
            ghost.pos.1 += 1;
        }
        ghost.pos.1 -= 1;
        ghost
    }

    fn quit(&mut self) {
        self.running = false;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = setup_terminal()?;
    let app_result = run(&mut terminal);
    restore_terminal(&mut terminal)?;
    
    // Print Leaderboard on quit
    if let Ok(res) = app_result {
        println!("\n=== LEADERBOARD ===");
        let mut sorted: Vec<_> = res.leaderboard.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        for (i, (name, score)) in sorted.iter().take(10).enumerate() {
            println!("{}. {:<15} {}", i + 1, name, score);
        }
        println!("===================\n");
    }

    Ok(())
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<App> {
    let mut app = App::new();

    while app.running {
        terminal.draw(|frame| ui(frame, &app))?;

        let timeout = if app.clearing_lines.is_some() || app.mode != GameMode::Playing {
            Duration::from_millis(50)
        } else {
            app.tick_rate.saturating_sub(app.last_tick.elapsed())
        };

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.mode {
                        GameMode::Playing => match key.code {
                            KeyCode::Char('q') => app.quit(),
                            KeyCode::Char('p') | KeyCode::Esc => app.toggle_pause(),
                            KeyCode::Char('r') => app.mode = GameMode::ConfirmingRestart,
                            KeyCode::Left => app.move_left(),
                            KeyCode::Right => app.move_right(),
                            KeyCode::Up => app.rotate(),
                            KeyCode::Down => app.tick(),
                            KeyCode::Char(' ') => app.hard_drop(),
                            _ => {}
                        },
                        GameMode::Paused => match key.code {
                            KeyCode::Char('p') | KeyCode::Esc => app.toggle_pause(),
                            KeyCode::Char('q') => app.quit(),
                            _ => {}
                        },
                        GameMode::ConfirmingRestart => match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => app.reset(),
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
                            KeyCode::Char('r') => app.reset(),
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

fn ui(frame: &mut Frame, app: &App) {
    let area = frame.area();
    
    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let title = Paragraph::new("=== TERMINAL TETRIS ===")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(title, outer_layout[0]);

    let game_area = outer_layout[1];
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(20)])
        .split(game_area);

    let left_panel_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(10),
            Constraint::Min(0),
        ])
        .split(layout[0]);

    let score_block = Block::default()
        .title("Stats")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    
    let high_score = app.leaderboard.values().max().unwrap_or(&0);
    let score_text = vec![
        Line::from(vec![Span::raw("Score:     "), Span::styled(app.score.to_string(), Style::default().fg(Color::Yellow))]),
        Line::from(vec![Span::raw("High Score:"), Span::styled(high_score.to_string(), Style::default().fg(Color::Green))]),
    ];
    frame.render_widget(Paragraph::new(score_text).block(score_block), left_panel_layout[0]);

    let ctrl_block = Block::default()
        .title("Controls")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));
    
    let ctrl_text = vec![
        Line::from(" ← / → : Move"),
        Line::from(" ↑      : Rotate"),
        Line::from(" ↓      : Soft Drop"),
        Line::from(" Space  : Hard Drop"),
        Line::from(" p/Esc  : Pause"),
        Line::from(" r      : Restart"),
        Line::from(" q      : Quit"),
    ];
    frame.render_widget(Paragraph::new(ctrl_text).block(ctrl_block), left_panel_layout[1]);

    let game_width = (BOARD_WIDTH * 2 + 2) as u16;
    let game_height = (BOARD_HEIGHT + 2) as u16;
    
    let game_rect = Rect {
        x: layout[1].x,
        y: layout[1].y,
        width: game_width,
        height: game_height,
    };

    let game_block = Block::default()
        .title("Board")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    
    frame.render_widget(game_block, game_rect);

    let inner_rect = Rect {
        x: game_rect.x + 1,
        y: game_rect.y + 1,
        width: game_width - 2,
        height: game_height - 2,
    };

    // Render Board
    for y in 0..BOARD_HEIGHT {
        for x in 0..BOARD_WIDTH {
            if let Some(color) = app.board[y][x] {
                let rect = Rect {
                    x: inner_rect.x + (x * 2) as u16,
                    y: inner_rect.y + y as u16,
                    width: 2,
                    height: 1,
                };
                frame.render_widget(Paragraph::new("[]").style(Style::default().bg(color).fg(Color::Black)), rect);
            }
        }
    }

    if app.mode != GameMode::EnteringName {
        // Render Ghost Piece
        if !app.game_over() && app.clearing_lines.is_none() {
            let ghost = app.get_ghost_piece();
            for (x, y) in ghost.global_blocks() {
                if y >= 0 && y < BOARD_HEIGHT as i32 && x >= 0 && x < BOARD_WIDTH as i32 {
                    let rect = Rect {
                        x: inner_rect.x + (x * 2) as u16,
                        y: inner_rect.y + y as u16,
                        width: 2,
                        height: 1,
                    };
                    frame.render_widget(Paragraph::new("[]").style(Style::default().fg(Color::DarkGray)), rect);
                }
            }

            // Render Current Piece
            for (x, y) in app.current_piece.global_blocks() {
                if y >= 0 && y < BOARD_HEIGHT as i32 && x >= 0 && x < BOARD_WIDTH as i32 {
                    let rect = Rect {
                        x: inner_rect.x + (x * 2) as u16,
                        y: inner_rect.y + y as u16,
                        width: 2,
                        height: 1,
                    };
                    frame.render_widget(Paragraph::new("[]").style(Style::default().bg(app.current_piece.shape.color()).fg(Color::Black)), rect);
                }
            }
        }
    }

    // MODALS
    if app.mode == GameMode::Paused {
        let block = Block::default().title("Paused").borders(Borders::ALL).border_style(Style::default().fg(Color::Blue));
        let area = centered_rect(30, 20, area);
        frame.render_widget(Clear, area);
        frame.render_widget(Paragraph::new("\n    PAUSED\n\n  (p) to resume").alignment(Alignment::Center).block(block), area);
    } else if app.mode == GameMode::ConfirmingRestart {
        let block = Block::default().title("Restart?").borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow));
        let area = centered_rect(30, 20, area);
        frame.render_widget(Clear, area);
        frame.render_widget(Paragraph::new("\n  Restart game?\n\n  (y)es / (n)o").block(block), area);
    } else if app.mode == GameMode::EnteringName {
        let block = Block::default().title("New High Score!").borders(Borders::ALL).border_style(Style::default().fg(Color::Green));
        let area = centered_rect(40, 20, area);
        frame.render_widget(Clear, area);
        let text = vec![
            Line::from(format!("  Score: {}", app.score)),
            Line::from("  Enter your name:"),
            Line::from(format!("  > {}_", app.current_input)),
            Line::from(""),
            Line::from("  (Enter to submit)"),
        ];
        frame.render_widget(Paragraph::new(text).block(block), area);
    } else if app.mode == GameMode::GameOver {
        let block = Block::default().title("Game Over").borders(Borders::ALL).border_style(Style::default().fg(Color::Red));
        let area = centered_rect(30, 20, area);
        frame.render_widget(Clear, area);
        frame.render_widget(Paragraph::new("\n    GAME OVER\n\n  (r)estart / (q)uit").alignment(Alignment::Center).block(block), area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

