use std::fs;
use std::path::PathBuf;
use std::env;

pub const LEADERBOARD_FILE: &str = "leaderboard.txt";
pub const CONFIG_FILE: &str = "config.txt";

fn get_data_dir() -> PathBuf {
    let home = env::var("HOME").or_else(|_| env::var("USERPROFILE")).unwrap_or_else(|_| ".".to_string());
    let path = PathBuf::from(home).join(".terminal-tetris");
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub show_ghost: bool,
    pub use_fill: bool,
}

impl Config {
    pub fn load() -> Self {
        let path = get_data_dir().join(CONFIG_FILE);
        if let Ok(content) = fs::read_to_string(&path) {
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
            return Self { show_ghost, use_fill };
        }
        Self { show_ghost: true, use_fill: true }
    }

    pub fn save(&self) {
        let path = get_data_dir().join(CONFIG_FILE);
        let content = format!("show_ghost={}\nuse_fill={}\n", self.show_ghost, self.use_fill);
        let _ = fs::write(path, content);
    }
}

pub fn load_leaderboard() -> (Vec<(String, u32)>, String) {
    let path = get_data_dir().join(LEADERBOARD_FILE);
    let mut board = Vec::new();
    let mut last_name = String::new();

    if let Ok(content) = fs::read_to_string(&path) {
        let lines: Vec<&str> = content.lines().collect();
        if let Some(name) = lines.get(0) {
            last_name = name.to_string();
        }
        for line in lines.iter().skip(1) {
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
    let path = get_data_dir().join(LEADERBOARD_FILE);
    let mut content = format!("{}\n", last_name);
    for (name, score) in board {
        content.push_str(&format!("{}: {}\n", name, score));
    }
    let _ = fs::write(path, content);
}
