use ratatui::prelude::Color;

pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    I, J, L, O, S, T, Z,
}

impl Shape {
    pub fn blocks(&self) -> [(i32, i32); 4] {
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

    pub fn color(&self) -> Color {
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

    pub fn all() -> [Shape; 7] {
        [Shape::I, Shape::J, Shape::L, Shape::O, Shape::S, Shape::T, Shape::Z]
    }
}

#[derive(Clone)]
pub struct Piece {
    pub shape: Shape,
    pub pos: (i32, i32),
    pub blocks: [(i32, i32); 4],
}

impl Piece {
    pub fn new(shape: Shape) -> Self {
        let blocks = shape.blocks();
        Self {
            shape,
            pos: (BOARD_WIDTH as i32 / 2 - 2, 0),
            blocks,
        }
    }

    pub fn rotate(&mut self) {
        if self.shape == Shape::O { return; }
        for block in &mut self.blocks {
            let x = block.0;
            let y = block.1;
            block.0 = 2 - y;
            block.1 = x;
        }
    }

    pub fn rotated(&self) -> Self {
        let mut next = self.clone();
        next.rotate();
        next
    }

    pub fn global_blocks(&self) -> [(i32, i32); 4] {
        let mut out = [(0, 0); 4];
        for i in 0..4 {
            out[i] = (self.pos.0 + self.blocks[i].0, self.pos.1 + self.blocks[i].1);
        }
        out
    }
}
