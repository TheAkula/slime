use std::cmp::{self};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct Row {
  string: String,
  len: usize,
}

impl Row {
  pub fn render(&self, start: usize, end: usize) -> String {
    let end = cmp::min(end, self.string.len());
    let start = cmp::min(start, end);
    let mut result = String::new();
    for grapheme in self.string[..]
      .graphemes(true)
      .skip(start)
      .take(end - start)
    {
      if grapheme == "\t" {
        result.push_str(" ")
      } else {
        result.push_str(grapheme);
      }      
    }
    result
  }
  pub fn size(&self) -> usize {
    self.string[..].graphemes(true).count()
  }
  pub fn insert(&mut self, at: usize, ch: char) {
    if at >= self.len {
      self.string.push(ch);      
    } else {
      let mut result: String = self.string[..].graphemes(true).take(at).collect();
      let remainder: String = self.string[..].graphemes(true).skip(at).collect();
      result.push(ch);
      result.push_str(&remainder);
      self.string = result;
    }
    self.update_len();
  }
  pub fn insert_str(&mut self, at: usize, s: &str) {
    if at >= self.len {
      self.string.push_str(s);      
    } else {
      let mut result: String = self.string[..].graphemes(true).take(at).collect();
      let remainder: String = self.string[..].graphemes(true).skip(at).collect();
      result.push_str(s);
      result.push_str(&remainder);      
    }
    self.update_len();
  }
  pub fn delete(&mut self, at: usize) {
    if at < self.len {
      let mut result: String = self.string[..].graphemes(true).take(at).collect();
      let remainder: String = self.string[..].graphemes(true).skip(at + 1).collect();
      result.push_str(&remainder);
      self.string = result;
      self.update_len();
    }
  }
  pub fn delete_slice(&mut self, from: usize, to: usize) -> Option<String> {
    if to > from && to <= self.len {
      let removed_part: String = self.string[..].graphemes(true).skip(from).take(to - from).collect();
      let mut result: String = self.string[..].graphemes(true).take(from).collect();
      let remainder: String = self.string[..].graphemes(true).skip(from + to - from).collect();
      result.push_str(&remainder);
      self.string = result;
      self.update_len();

      return Some(removed_part);
    }

    None
  }
  pub fn string(&self) -> &str {
    &self.string
  }
  pub fn as_bytes(&self) -> &[u8] {
    self.string.as_bytes()
  }
  fn update_len(&mut self) {
    self.len = self.string[..].graphemes(true).count();
  }
}

impl From<String> for Row {
  fn from(string: String) -> Row {
    let mut row = Self {
      string,
      len: 0,
    };

    row.update_len();

    row
  }
}

impl From<&str> for Row {
  fn from(slice: &str) -> Row {
    let mut row = Self {
      string: String::from(slice),
      len: 0,
    };

    row.update_len();

    row
  }  
}
