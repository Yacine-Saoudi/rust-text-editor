use crate::Terminal;
use crate::Document;
use crate::Row;
use std::env;
use std::io::Error;
use termion::event::Key;
use termion::color;
use core::time::Duration;
use std::time::Instant;

const STATUS_FG_COLOR: color::Rgb = color::Rgb(63,63,63);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239,239,239);

#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

struct StatusMessage {
    text: String,
    time: Instant,
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
        }
    }
}

pub struct Editor {
    quitting: bool,
    terminal: Terminal,
    cursor_position: Position,
    document: Document,
    offset: Position,
    status_message: StatusMessage,
}

impl Editor {
    #[allow(clippy::single_call_fn, clippy::expect_used)]
    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("HELP: Ctrl-S to save | Ctrl-Q to quit");
        let document = if let Some(file_name) = args.get(1) {
            let doc = Document::open(file_name);
            if let Ok(doc) = doc {
                doc
            } 
            else {
                initial_status = format!("ERR: Could not open file: {file_name}");
                Document::default()
            }
        } else {
            Document::default()
        };

        Self{
            quitting: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            cursor_position: Position::default(),
            document,
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
        }
    }
    
    pub fn run(&mut self){
        loop {
            if let Err(err) = self.refresh_screen() {
                die(&err);
            }
            if self.quitting {
                break;
            }
            if let Err(err) = self.process_keypress() {
                die(&err);
            }
        }
    }

    fn refresh_screen(&self) -> Result<(), Error>{
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());
        if self.quitting {
            Terminal::clear_screen();
            println!("baiii!~\r");
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }
        Terminal::cursor_show();
        Terminal::flush()
    }

    fn draw_status_bar(&self) {
        let mut status;
        let width = self.terminal.size().width as usize;
        let modified_indicator = if self.document.is_dirty() {
            " (modified)"
        } else {
            ""
        };
        let mut file_name = "[No Name]".to_owned();
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }
        status = format!("{} - {} lines{}", file_name, self.document.len(), modified_indicator);

        let line_indicator = format!(
            "{}/{}",
            self.cursor_position.y.saturating_add(1),
            self.document.len()
        );
        let len = status.len().saturating_add(line_indicator.len());
        status.push_str(&" ".repeat(width.saturating_sub(len)));

        status = format!("{status}{line_indicator}",);
        status.truncate(width);
        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_bg_color(STATUS_FG_COLOR);
        println!("{status}\r");
        Terminal::reset_bg_color();
        Terminal::reset_fg_color();
    }

    fn draw_message_bar(&self) {
       Terminal::clear_current_line();
       let message = &self.status_message;
       if message.time.elapsed() < Duration::new(5, 0) {
        let mut text = message.text.clone();
        text.truncate(self.terminal.size().width as usize);
        print!("{text}");
       } 
    }

    pub fn draw_row(&self, row: &Row){
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(width);
        let row = row.render(start, end);
        println!("{row}\r");
    }

    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self
            .document
            .row(self.offset.y.saturating_add(terminal_row as usize))
            {
                self.draw_row(row);
            } else {
                println!("~\r");
            }
        }
    }

    fn process_keypress(&mut self) -> Result <(), Error> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            Key::Ctrl('q') => {
                if self.document.is_dirty() {
                    let should_quit = self.prompt("Quit without saving? (y/n) ").unwrap_or(None);
                    self.quitting = should_quit.unwrap() == *"y";
                } else {
                    self.quitting = true;
                }
            }
            Key::Backspace => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(Key::Left);
                    self.document.delete(&self.cursor_position);
                }
            },
            Key::Char(c) => {
                self.document.insert(&self.cursor_position, c);
                self.move_cursor(Key::Right);
            },
            Key::Ctrl('s') => self.save(),
        //    Key::Char('k' | 'j' | 'h' | 'l' | '^' | '$' | 'G' | 'g') 
           | Key::Up 
           | Key::Down 
           | Key::Left 
           | Key::Right 
           | Key::Ctrl('b' | 'f') 
            => self.move_cursor(pressed_key),
            _ => (),
        }
        self.scroll();
        Ok(())
    }

    fn save(&mut self) {
        if self.document.file_name.is_none() {
            let new_name = self.prompt("Save as: ").unwrap_or(None);
            if new_name.is_none() {
                self.status_message = StatusMessage::from("Save aborted".to_string());
                return;
            }
            self.document.file_name = new_name;
        }
        if self.document.save().is_ok() {
            self.status_message = StatusMessage::from("File saved succesfully.".to_string());
        } else {
            self.status_message = StatusMessage::from("Error writing to file.".to_string());
        }
    }

    #[allow(clippy::min_ident_chars)]
    fn prompt(&mut self, prompt: &str) -> Result <Option<String>, Error> {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{prompt}{result}"));
            self.refresh_screen()?;
            match Terminal::read_key()? {
                Key::Backspace => result.truncate(result.len().saturating_sub(1)),
                Key::Char('\n') => break,
                Key::Char(c) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                },
                Key::Esc => {
                    result.truncate(0);
                    break;
                }
                _ => (),
            }
        }
        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() {
            return Ok(None);
        }
        Ok(Some(result))
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let offset = &mut self.offset;
        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    fn move_cursor(&mut self, key: Key){
        let Position { mut y, mut x } = self.cursor_position;
        let _size = self.terminal.size();
        let height = self.document.len();
        let terminal_height = self.terminal.size().height as usize;
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        match key {
            Key::Up | Key::Char('k') => y = y.saturating_sub(1),
            Key::Down | Key::Char('j') => {
                if y < height {
                    y = y.saturating_add(1);
                }
            },
            Key::Left | Key::Char('h') => {
                if x > 0 {
                    x = x.saturating_sub(1);
                } else if y > 0 {
                    y = y.saturating_sub(1);
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
             },
            Key::Right |Key::Char('l') => {
                if x < width {
                    x = x.saturating_add(1);
                } else if y <= height {
                    y = y.saturating_add(1);
                    x = 0;
                }
            },
            Key::Char('^') => x = 0,
            Key::Char('$') => x = width,
            Key::Char('G') => y = height,
            Key::Char('g') => y = 0,
            Key::Ctrl('b') => { // go up
                y = if y > terminal_height {
                    y.saturating_sub(terminal_height)
                } else {
                    0
                }
            },
            Key::Ctrl('f') => { // go down
                y = if y.saturating_add(terminal_height) < height {
                    y.saturating_add(terminal_height)
                } else {
                    height
                }
            },
            _ => (),
        }
        width = if let Some(row) = self.document.row(y){
            row.len()
        } else {
            0
        };

        if x > width {
            x = width;
        }

        self.cursor_position = Position { x, y }
    }
}

fn die(e: &Error){
    Terminal::clear_screen();
    panic!("{}", e);
}
