use crate::Position;
use std::io::{self, stdout, Error, Write};
use termion::input::TermRead;
use termion::event::Key;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::color;
use termion::cursor::{Hide, Show, Goto};

pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Terminal {
    size: Size,
    _stdout: RawTerminal<std::io::Stdout>,
}

impl Terminal {
    #[allow(clippy::single_call_fn)]
    pub fn default() -> Result <Self, Error> {
        let size = termion::terminal_size()?;
        Ok(Self {
            size: Size {
                width: size.0,
                height: size.1.saturating_sub(2),
            },
            _stdout: stdout().into_raw_mode()?,
        })
    }

    pub fn size(&self) -> &Size {
        &self.size
    }

    #[allow(clippy::absolute_paths)]
    pub fn clear_screen() {
        print!("{}", termion::clear::All);
    }

    pub fn set_bg_color(color: color::Rgb) {
        print!("{}", color::Bg(color));
    }

    #[allow(clippy::single_call_fn)]
    pub fn reset_bg_color() {
        print!("{}", color::Bg(color::Reset));
    }

    #[allow(clippy::single_call_fn)]
    pub fn reset_fg_color() {
        print!("{}", color::Fg(color::Reset));
    }

    #[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
    pub fn cursor_position(pos: &Position) {
        let Position{mut x, mut y} = *pos;
        x = x.saturating_add(1);
        y = y.saturating_add(1);
        let x = x as u16;
        let y = y as u16;
        
        print!("{}", Goto(x, y));
    }

    #[allow(clippy::single_call_fn)]
    pub fn flush() -> Result<(), Error> {
        io::stdout().flush()
    }

    pub fn read_key() -> Result<Key, Error> {
        loop {
            if let Some(key) = io::stdin().lock().keys().next() {
                return key;
            }
        }
    }

    #[allow(clippy::single_call_fn)]
    pub fn cursor_hide() {
        print!("{Hide}");
    }
    
    #[allow(clippy::single_call_fn)]
    pub fn cursor_show() {
        print!("{Show}");
    }

    #[allow(clippy::absolute_paths)]
    pub fn clear_current_line() {
        print!("{}", termion::clear::CurrentLine);
    }
}