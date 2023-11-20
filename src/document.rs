use std::fs::File;
use std::io::Write;
use std::{io::Error, fs};

use crate::Row;
use crate::Position;
use crate::editor::SearchDir;

#[derive(Default, Clone)]
pub struct Document {
  pub path: Option<String>,
  rows: Vec<Row>,
  dirty: bool,
}

impl Document {    
  pub fn open(path: &str) -> Result<Self, Error> {
    let contents = fs::read_to_string(path)?;
    let mut rows = Vec::new();
    for value in contents.lines() {
      rows.push(Row::from(value));
    }    
    Ok(Self{
      rows,
      path: Some(path.to_string()),
      dirty: false,
    })
  }
  pub fn row(&self, index: usize) -> Option<&Row> {
    self.rows.get(index)
  }  
  pub fn rows_size(&self) -> usize {
    self.rows.len()
  }
  pub fn is_empty(&self) -> bool {
    self.rows.len() == 0
  }
  pub fn insert(&mut self, at: &Position<usize>, ch: char) {
    if at.y > self.rows_size() {
      return;
    }
    self.dirty = true;
    if ch == '\n' {
      self.insert_enter_key(at);
      return;
    }        
    if at.y == self.rows_size() {
      let mut row = Row::default();
      row.insert(0, ch);
      self.rows.push(row);
    } else if at.y < self.rows_size() {
      let row = self.row_mut(at.y).unwrap();
      row.insert(at.x, ch);      
    }
  }
  pub fn insert_str(&mut self, at: &Position<usize>, s: &str) {
    if at.y == self.rows_size() {
      let mut row = Row::default();
      row.insert_str(0, s);
      self.rows.push(row);
    } else if at.y < self.rows_size() {
      let row = self.row_mut(at.y).unwrap();
      row.insert_str(at.x, s);      
    }
  }  
  pub fn delete(&mut self, at: &Position<usize>) {
    if at.y < self.rows_size() {                     
      if at.y < self.rows_size() - 1 {
        if let [prev_row, row, ..] = &mut self.rows[(at.y)..(at.y + 2)] {        
          if at.x == prev_row.size() {
            prev_row.insert_str(prev_row.size(), row.string());
            self.rows.remove(at.y + 1);

            return;
          }
        }
      } 

      let row = self.row_mut(at.y).unwrap();              
      row.delete(at.x);                     
    }          
  } 
  pub fn save_to_disk(&mut self) -> Result<(), Error> {
    if let Some(path) = &self.path {
      let mut file = File::create(path)?;
      for row in &self.rows {
        file.write_all(row.as_bytes())?;
        file.write_all(b"\n")?;
      }      
    }

    self.dirty = false;
    Ok(())
  }
  pub fn is_dirty(&self) -> bool {
    self.dirty
  }
  pub fn find(&self, query: &str, at: &Position<usize>, direction: SearchDir) -> Option<Position<usize>> {    
    if at.y > self.rows_size() {
      return None
    }

    let start = if direction == SearchDir::Forward {
      at.y
    } else {
      0
    };

    let end = if direction == SearchDir::Forward {
      self.rows_size()      
    } else {
      at.y.saturating_add(1)
    };

    let mut position = Position { x: at.x, y: at.y };

    for _ in start..end {
      if let Some(row) = self.row(position.y) {
        if let Some(x) = row.find(query, position.x, direction) {
          position.x = x;
          return Some(position);
        }
        if direction == SearchDir::Forward {
          position.y = position.y.saturating_add(1);
          position.x = 0;
        } else {
          position.y = position.y.saturating_sub(1);
          position.x = self.rows[position.y].size();
        }
      } else {
        return None;
      }
    }
    
    None
  }
  fn row_mut(&mut self, index: usize) -> Option<&mut Row> {
    if index < self.rows.len() {
      Some(&mut self.rows[index])
    } else {
      None
    }
  }  
  fn insert_enter_key(&mut self, at: &Position<usize>) {          
    if at.y < self.rows_size() {
      let row = self.row_mut(at.y).unwrap();

      let mut new_row = Row::default();
      if let Some(slice) = row.delete_slice(at.x, row.size()) {        
        new_row.insert_str(0, &slice);        
      }
      self.rows.insert(at.y + 1, new_row);
    }  
  }  
}
