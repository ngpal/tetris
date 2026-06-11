use std::{
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
    widgets::{Block, Borders, Paragraph},
};

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;

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

struct App {
    running: bool,
    board: [[Option<Color>; BOARD_WIDTH]; BOARD_HEIGHT],
    current_piece: Piece,
    score: u32,
    game_over: bool,
    last_tick: Instant,
    tick_rate: Duration,
}

impl App {
    fn new() -> Self {
        let shape = Shape::all()[rand::random::<usize>() % 7];
        Self {
            running: true,
            board: [[None; BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece: Piece::new(shape),
            score: 0,
            game_over: false,
            last_tick: Instant::now(),
            tick_rate: Duration::from_millis(500),
        }
    }

    fn spawn_piece(&mut self) {
        let shape = Shape::all()[rand::random::<usize>() % 7];
        self.current_piece = Piece::new(shape);
        if self.is_collision(&self.current_piece) {
            self.game_over = true;
        }
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
        self.clear_lines();
        self.spawn_piece();
    }

    fn clear_lines(&mut self) {
        let mut lines_cleared = 0;
        let mut new_board = [[None; BOARD_WIDTH]; BOARD_HEIGHT];
        let mut new_y = BOARD_HEIGHT - 1;

        for y in (0..BOARD_HEIGHT).rev() {
            let full = (0..BOARD_WIDTH).all(|x| self.board[y][x].is_some());
            if full {
                lines_cleared += 1;
            } else {
                new_board[new_y] = self.board[y];
                if new_y > 0 {
                    new_y -= 1;
                }
            }
        }

        self.board = new_board;
        self.score += lines_cleared * 100;
    }

    fn tick(&mut self) {
        if self.game_over {
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
        let mut next = self.current_piece.clone();
        next.pos.0 -= 1;
        if !self.is_collision(&next) {
            self.current_piece = next;
        }
    }

    fn move_right(&mut self) {
        let mut next = self.current_piece.clone();
        next.pos.0 += 1;
        if !self.is_collision(&next) {
            self.current_piece = next;
        }
    }

    fn rotate(&mut self) {
        let next = self.current_piece.rotated();
        if !self.is_collision(&next) {
            self.current_piece = next;
        }
    }

    fn hard_drop(&mut self) {
        while !self.game_over {
            let mut next = self.current_piece.clone();
            next.pos.1 += 1;
            if self.is_collision(&next) {
                self.lock_piece();
                break;
            }
            self.current_piece = next;
        }
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
    app_result?;
    Ok(())
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut app = App::new();

    while app.running {
        terminal.draw(|frame| ui(frame, &app))?;

        let timeout = app.tick_rate.saturating_sub(app.last_tick.elapsed());

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => app.quit(),
                        KeyCode::Left => app.move_left(),
                        KeyCode::Right => app.move_right(),
                        KeyCode::Up => app.rotate(),
                        KeyCode::Down => app.tick(),
                        KeyCode::Char(' ') => app.hard_drop(),
                        _ => {}
                    }
                }
            }
        }

        if app.last_tick.elapsed() >= app.tick_rate {
            app.tick();
            app.last_tick = Instant::now();
        }
    }

    Ok(())
}

fn ui(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(20)])
        .split(area);

    let info_block = Block::default()
        .title("Info")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));
    
    let info_text = vec![
        Line::from(vec![Span::raw("Score: "), Span::styled(app.score.to_string(), Style::default().fg(Color::Yellow))]),
        Line::from(""),
        Line::from("Controls:"),
        Line::from("  ← / → : Move"),
        Line::from("  ↑      : Rotate"),
        Line::from("  ↓      : Soft Drop"),
        Line::from("  Space  : Hard Drop"),
        Line::from("  q      : Quit"),
    ];
    
    let info = Paragraph::new(info_text).block(info_block);
    frame.render_widget(info, layout[0]);

    let game_width = (BOARD_WIDTH * 2 + 2) as u16;
    let game_height = (BOARD_HEIGHT + 2) as u16;
    
    let game_rect = Rect {
        x: layout[1].x,
        y: layout[1].y,
        width: game_width,
        height: game_height,
    };

    let game_block = Block::default()
        .title("Tetris")
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
                frame.render_widget(Paragraph::new("[]").style(Style::default().fg(color)), rect);
            }
        }
    }

    if !app.game_over {
        // Render Ghost Piece
        let ghost = app.get_ghost_piece();
        for (x, y) in ghost.global_blocks() {
            if y >= 0 && y < BOARD_HEIGHT as i32 && x >= 0 && x < BOARD_WIDTH as i32 {
                let rect = Rect {
                    x: inner_rect.x + (x * 2) as u16,
                    y: inner_rect.y + y as u16,
                    width: 2,
                    height: 1,
                };
                // Dotted look or just dimmed color for ghost
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
                frame.render_widget(Paragraph::new("[]").style(Style::default().fg(app.current_piece.shape.color())), rect);
            }
        }
    } else {
        let msg = Paragraph::new("GAME OVER")
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        let msg_rect = Rect {
            x: inner_rect.x,
            y: inner_rect.y + (BOARD_HEIGHT / 2) as u16,
            width: inner_rect.width,
            height: 1,
        };
        frame.render_widget(msg, msg_rect);
    }
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

