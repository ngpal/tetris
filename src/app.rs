use std::time::{Duration, Instant};
use ratatui::widgets::ListState;
use ratatui::prelude::Color;
use crate::game::{Piece, Shape, BOARD_WIDTH, BOARD_HEIGHT};
use crate::config::{Config, load_leaderboard, save_leaderboard};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum GameMode {
    MainMenu,
    Options,
    Leaderboard,
    Playing,
    Paused,
    ConfirmingRestart,
    EnteringName,
    GameOver,
}

pub struct App {
    pub running: bool,
    pub mode: GameMode,
    pub config: Config,
    pub board: [[Option<Color>; BOARD_WIDTH]; BOARD_HEIGHT],
    pub current_piece: Piece,
    pub next_piece: Piece,
    pub score: u32,
    pub leaderboard: Vec<(String, u32)>,
    pub last_name: String,
    pub current_input: String,
    pub last_tick: Instant,
    pub tick_rate: Duration,
    pub clearing_lines: Option<(u32, Instant)>,
    pub menu_state: ListState,
    pub options_state: ListState,
}

impl App {
    pub fn new(provided_name: Option<String>) -> Self {
        let (leaderboard, mut last_name) = load_leaderboard();
        if let Some(name) = provided_name {
            last_name = name;
        }
        let mut menu_state = ListState::default();
        menu_state.select(Some(0));
        let mut options_state = ListState::default();
        options_state.select(Some(0));

        let mut app = Self {
            running: true,
            mode: GameMode::MainMenu,
            config: Config::load(),
            board: [[None; BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece: Piece::new(Shape::I),
            next_piece: Piece::new(Shape::I),
            score: 0,
            leaderboard,
            last_name,
            current_input: String::new(),
            last_tick: Instant::now(),
            tick_rate: Duration::from_millis(800),
            clearing_lines: None,
            menu_state,
            options_state,
        };
        app.spawn_piece();
        app.spawn_piece();
        app
    }

    pub fn reset_for_game(&mut self) {
        self.board = [[None; BOARD_WIDTH]; BOARD_HEIGHT];
        self.score = 0;
        self.mode = GameMode::Playing;
        self.spawn_piece();
        self.spawn_piece();
        self.last_tick = Instant::now();
    }

    pub fn toggle_pause(&mut self) {
        if self.mode == GameMode::Playing {
            self.mode = GameMode::Paused;
        } else if self.mode == GameMode::Paused {
            self.mode = GameMode::Playing;
            self.last_tick = Instant::now();
        }
    }

    pub fn spawn_piece(&mut self) {
        self.current_piece = self.next_piece.clone();
        let shape = Shape::all()[rand::random::<usize>() % 7];
        self.next_piece = Piece::new(shape);
        
        if self.is_collision(&self.current_piece) {
            self.mode = GameMode::GameOver;
            self.check_high_score();
        }
    }

    fn check_high_score(&mut self) {
        let is_high = self.leaderboard.len() < 10 
            || self.score > self.leaderboard.last().map(|e| e.1).unwrap_or(0);
        
        if is_high && self.score > 0 {
            self.mode = GameMode::EnteringName;
            self.current_input = self.last_name.clone();
        }
    }

    pub fn submit_score(&mut self) {
        let name = if self.current_input.trim().is_empty() {
            if self.last_name.is_empty() { "Player".to_string() } else { self.last_name.clone() }
        } else {
            self.current_input.trim().to_string()
        } ;

        self.last_name = name.clone();
        self.leaderboard.push((name, self.score));
        self.leaderboard.sort_by(|a, b| b.1.cmp(&a.1));
        self.leaderboard.truncate(10);

        save_leaderboard(&self.leaderboard, &self.last_name);
        self.mode = GameMode::GameOver;
    }

    pub fn is_collision(&self, piece: &Piece) -> bool {
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

    pub fn lock_piece(&mut self) {
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

    pub fn finalize_clear_lines(&mut self) {
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

    pub fn tick(&mut self) {
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

    pub fn move_left(&mut self) {
        if self.mode != GameMode::Playing || self.clearing_lines.is_some() { return; }
        let mut next = self.current_piece.clone();
        next.pos.0 -= 1;
        if !self.is_collision(&next) {
            self.current_piece = next;
        }
    }

    pub fn move_right(&mut self) {
        if self.mode != GameMode::Playing || self.clearing_lines.is_some() { return; }
        let mut next = self.current_piece.clone();
        next.pos.0 += 1;
        if !self.is_collision(&next) {
            self.current_piece = next;
        }
    }

    pub fn rotate(&mut self) {
        if self.mode != GameMode::Playing || self.clearing_lines.is_some() { return; }
        let next = self.current_piece.rotated();
        if !self.is_collision(&next) {
            self.current_piece = next;
        }
    }

    pub fn hard_drop(&mut self) {
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

    pub fn game_over(&self) -> bool {
        self.mode == GameMode::GameOver || self.mode == GameMode::EnteringName
    }

    pub fn get_ghost_piece(&self) -> Piece {
        let mut ghost = self.current_piece.clone();
        while !self.is_collision(&ghost) {
            ghost.pos.1 += 1;
        }
        ghost.pos.1 -= 1;
        ghost
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn menu_next(&mut self) {
        let i = match self.menu_state.selected() {
            Some(i) => if i >= 3 { 0 } else { i + 1 },
            None => 0,
        };
        self.menu_state.select(Some(i));
    }

    pub fn menu_prev(&mut self) {
        let i = match self.menu_state.selected() {
            Some(i) => if i == 0 { 3 } else { i - 1 },
            None => 0,
        };
        self.menu_state.select(Some(i));
    }

    pub fn menu_select(&mut self) {
        match self.menu_state.selected() {
            Some(0) => self.reset_for_game(),
            Some(1) => self.mode = GameMode::Options,
            Some(2) => self.mode = GameMode::Leaderboard,
            Some(3) => self.quit(),
            _ => {}
        }
    }

    pub fn options_next(&mut self) {
        let i = match self.options_state.selected() {
            Some(i) => if i >= 2 { 0 } else { i + 1 },
            None => 0,
        };
        self.options_state.select(Some(i));
    }

    pub fn options_prev(&mut self) {
        let i = match self.options_state.selected() {
            Some(i) => if i == 0 { 2 } else { i - 1 },
            None => 0,
        };
        self.options_state.select(Some(i));
    }

    pub fn options_select(&mut self) {
        match self.options_state.selected() {
            Some(0) => { self.config.show_ghost = !self.config.show_ghost; self.config.save(); },
            Some(1) => { self.config.use_fill = !self.config.use_fill; self.config.save(); },
            Some(2) => self.mode = GameMode::MainMenu,
            _ => {}
        }
    }
}
