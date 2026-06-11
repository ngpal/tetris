use std::io::{self, stdout};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{Event, KeyCode, read},
    execute,
    style::Print,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(
        stdout(),
        EnterAlternateScreen,
        Hide,
        MoveTo(0, 0),
        Print("Hello, world!")
    )?;

    loop {
        if let Ok(event) = read() {
            match event {
                Event::Key(key) => {
                    if key.code == KeyCode::Esc {
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    execute!(stdout(), LeaveAlternateScreen, Show)?;
    disable_raw_mode()?;

    Ok(())
}
