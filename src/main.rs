use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, ClearType},
};
use std::io::{stdout, Result, Write};
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    Normal,
    Insert,
}

struct Editor {
    content: Vec<String>,
    cursor_x: usize,
    cursor_y: usize,
    mode: Mode,
    quit: bool,
}

impl Editor {
    fn new() -> Self {
        Self {
            content: vec![String::new()],
            cursor_x: 0,
            cursor_y: 0,
            mode: Mode::Normal,
            quit: false,
        }
    }

    fn run(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), terminal::EnterAlternateScreen)?;

        while !self.quit {
            self.draw_screen()?;
            self.process_keypress()?;
        }

        execute!(stdout(), terminal::LeaveAlternateScreen, cursor::Show)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    fn draw_screen(&mut self) -> Result<()> {
        execute!(
            stdout(),
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        // Draw content
        for (i, line) in self.content.iter().enumerate() {
            if i > 0 {
                print!("\r\n");
            }
            print!("{}", line);
        }

        // Draw status line
        let mode_str = match self.mode {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
        };
        let status = format!(
            "\r\n-- {} -- Cursor: ({}, {})",
            mode_str, self.cursor_x, self.cursor_y
        );
        print!("{}", status);

        // Move cursor to current position
        execute!(
            stdout(),
            cursor::MoveTo(self.cursor_x as u16, self.cursor_y as u16),
            cursor::Show
        )?;

        stdout().flush()?;
        Ok(())
    }

    fn process_keypress(&mut self) -> Result<()> {
        if let Event::Key(key) = event::read()? {
            match self.mode {
                Mode::Normal => self.handle_normal_mode(key),
                Mode::Insert => self.handle_insert_mode(key),
            }?;
        }
        Ok(())
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('i') => self.mode = Mode::Insert,
            KeyCode::Char('h') => self.move_cursor_left(),
            KeyCode::Char('j') => self.move_cursor_down(),
            KeyCode::Char('k') => self.move_cursor_up(),
            KeyCode::Char('l') => self.move_cursor_right(),
            _ => (),
        }
        Ok(())
    }

    fn handle_insert_mode(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Char(c) => self.insert_char(c),
            KeyCode::Backspace => self.delete_char(),
            KeyCode::Enter => self.insert_newline(),
            _ => (),
        }
        Ok(())
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        let line_len = self.content[self.cursor_y].width();
        if self.cursor_x < line_len {
            self.cursor_x += 1;
        }
    }

    fn move_cursor_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            let line_len = self.content[self.cursor_y].width();
            if self.cursor_x > line_len {
                self.cursor_x = line_len;
            }
        }
    }

    fn move_cursor_down(&mut self) {
        if self.cursor_y < self.content.len() - 1 {
            self.cursor_y += 1;
            let line_len = self.content[self.cursor_y].width();
            if self.cursor_x > line_len {
                self.cursor_x = line_len;
            }
        }
    }

    fn insert_char(&mut self, c: char) {
        let line = &mut self.content[self.cursor_y];
        line.insert(self.cursor_x, c);
        self.cursor_x += 1;
    }

    fn delete_char(&mut self) {
        let line = &mut self.content[self.cursor_y];
        if self.cursor_x > 0 {
            line.remove(self.cursor_x - 1);
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            let current_line = self.content.remove(self.cursor_y);
            self.cursor_y -= 1;
            self.cursor_x = self.content[self.cursor_y].len();
            self.content[self.cursor_y].push_str(&current_line);
        }
    }

    fn insert_newline(&mut self) {
        let current_line = &mut self.content[self.cursor_y];
        let new_line = current_line.split_off(self.cursor_x);
        self.content.insert(self.cursor_y + 1, new_line);
        self.cursor_y += 1;
        self.cursor_x = 0;
    }
}

fn main() -> Result<()> {
    let mut editor = Editor::new();
    editor.run()
}
