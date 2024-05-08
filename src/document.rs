use std::fs;
use std::io::{Error, Write};
use crate::Position;
use crate::Row;

#[derive(Default)]
#[allow(clippy::partial_pub_fields)]
pub struct Document {
    rows: Vec<Row>,
    pub file_name: Option<String>,
    dirty: bool,
}

impl Document {
    #[allow(clippy::single_call_fn)]
    pub fn open(filename: &str) -> Result<Self, Error> {
        #[allow(clippy::question_mark_used)]
        let contents = fs::read_to_string(filename)?;
        let mut rows = Vec::new();
        for value in contents.lines(){
            rows.push(Row::from(value));
        }
        Ok(Self {
            rows,
            file_name: Some(filename.to_owned()),
            dirty: false,
        })
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    #[allow(clippy::question_mark_used)]
    pub fn save(&mut self) -> Result<(), Error> {
        if let Some(file_name) = &self.file_name {
            let mut file: fs::File = fs::File::create(file_name)?;
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
            self.dirty = false;
        }
        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn insert_newline(&mut self, at: &Position) {
        if at.y > self.rows.len() {
            return;
        }
        self.dirty = true;
        if at.y == self.rows.len() {
            self.rows.push(Row::default());
            return;
        }
        #[allow(clippy::indexing_slicing)]
        let new_row = self.rows[at.y].split(at.x);
        #[allow(clippy::arithmetic_side_effects)]
        self.rows.insert(at.y + 1, new_row);
    }

    #[allow(clippy::min_ident_chars)]
    pub fn insert(&mut self, at: &Position, c:char){
        if c == '\n' {
            self.insert_newline(at);
            return;
        }

        if at.y == self.rows.len() {
            let mut row = Row::default();
            row.insert(0, c);
            self.rows.push(row);
        } else {
            #[allow(clippy::unwrap_used, clippy::get_unwrap)]
            let row = self.rows.get_mut(at.y).unwrap();
            row.insert(at.x, c);
        }
        self.dirty = true;
    }

    #[allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
    pub fn delete(&mut self, at: &Position) {
        let len = self.rows.len();
        if at.y >= len {
            return;
        }
        self.dirty = true;
        if at.x == self.rows[at.y].len() && at.y + 1 < len {
            let next_row = self.rows.remove(at.y +1);
            let row = &mut self.rows[at.y];
            row.append(&next_row);
        } else {
            let row = &mut self.rows[at.y];
            row.delete(at.x);
        }
    }
}