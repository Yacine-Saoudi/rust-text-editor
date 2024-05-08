#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::implicit_return,
    clippy::shadow_reuse,
    clippy::print_stdout,
    clippy::wildcard_enum_match_arm,
    clippy::else_if_without_else,
)]
mod editor;
mod terminal;
mod document;
mod row;

use editor::Editor;
use row::Row;
 use terminal::Terminal;
 use editor::Position;
 use document::Document;

fn main() {
    Editor::default().run();
}
