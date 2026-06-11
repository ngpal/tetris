use std::fs;

pub const LEADERBOARD_FILE: &str = "leaderboard.txt";
pub const CONFIG_FILE: &str = "config.txt";

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub show_ghost: bool,
    pub use_fill: bool,
}

impl Config {
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string(CONFIG_FILE) {
            let mut show_ghost = true;
            let mut use_fill = true;
            for line in content.lines() {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() == 2 {
                    match parts[0] {
                        "show_ghost" => show_ghost = parts[1] == "true",
                        "use_fill" => use_fill = parts[1] == "true",
                        _ => {}
                    }
                }
            }
            Self { show_ghost, use_fill }
        } else {
            Self { show_ghost: true, use_fill: true }
        }
    }

    pub fn save(&self) {
        let content = format!("show_ghost={}\nuse_fill={}", self.show_ghost, self.use_fill);
        let _ = fs::write(CONFIG_FILE, content);
    }
}

pub fn load_leaderboard() -> (Vec<(String, u32)>, String) {
    let mut board = Vec::new();
    let mut last_name = String::new();
    if let Ok(content) = fs::read_to_string(LEADERBOARD_FILE) {
        let lines: Vec<&str> = content.lines().collect();
        for line in lines {
            if line.starts_with("LAST_NAME:") {
                last_name = line.replace("LAST_NAME:", "").trim().to_string();
                continue;
            }
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() == 2 {
                if let Ok(score) = parts[1].trim().parse::<u32>() {
                    board.push((parts[0].trim().to_string(), score));
                }
            }
        }
    }
    board.sort_by(|a, b| b.1.cmp(&a.1));
    (board, last_name)
}

pub fn save_leaderboard(board: &[(String, u32)], last_name: &str) {
    let mut results = vec![format!("LAST_NAME:{}", last_name)];
    for (name, score) in board {
        results.push(format!("{}:{}", name, score));
    }
    let _ = fs::write(LEADERBOARD_FILE, results.join("\n"));
}
