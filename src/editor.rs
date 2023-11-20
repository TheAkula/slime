use std::env;
use std::io::Error;
use std::time::{Instant, Duration};

use crossterm::event::{Event, KeyCode, KeyModifiers, KeyEvent};
use crossterm::style::{Color, Colors};

use crate::Row;
use crate::Terminal;
use crate::Document;

#[derive(Default, Clone)]
pub struct Position<T> {
  pub x: T,
  pub y: T,
}

pub struct StatusMessage {
  text: String,
  time: Instant,
}

impl StatusMessage {
  fn from(message: String) -> Self {
    Self {
      text: message,
      time: Instant::now()
    }
  }
}

#[derive(PartialEq, Copy, Clone)]            

pub enum SearchDir {
  Forward,
  Backward,
}

pub struct Editor {
  should_quit: bool,  
  terminal: Terminal,
  cursor_position: Position<usize>,
  cursor_offset: Position<usize>,
  document: Document,
  status_message: StatusMessage,
  quit_times: u8,  
}

const STATUS_BAR_BG: Color = Color::Rgb { r: 239, g: 239, b: 239 };
const STATUS_BAR_FG: Color = Color::Rgb { r: 63, g: 63, b: 63 };
const VERSION: &str = env!("CARGO_PKG_VERSION");
const STATUS_MESSAGE_LIVE_TIME: u64 = 5; // seconds
const QUIT_TIMES: u8 = 3;

impl Editor {
  pub fn run(&mut self) -> std::io::Result<()> { 
    self.refresh_screen()?;                   

    while !self.should_quit {                           
      if let Some(event) = self.terminal.read_event()? {                         
        if let Err(err) = self.process_event(event) {
          self.die(err)?;        
        }                                                    
        self.refresh_screen()?;
      }      
    }      

    self.refresh_screen()?;
    
    Ok(())
  }

  pub fn default() -> Result<Editor, Error> {    
    let args: Vec<String> = env::args().collect();
    
    let mut initial_status = String::from("HELP: Ctrl-C = exit");    
    let document = if args.len() > 1 {
      let file_name = &args[1];
      let doc = Document::open(&file_name);
      if doc.is_ok() {
        doc.unwrap()
      } else {
        initial_status = format!("ERR: Could not open file {}", file_name);
        Document::default()
      }
    } else {
      Document::default()
    };

    Ok(Self{
      should_quit: false,
      terminal: Terminal::default().expect("Failed to initialize terminal"),
      cursor_position: Position::default(),
      document,
      cursor_offset: Position::default(), 
      status_message: StatusMessage::from(initial_status),    
      quit_times: QUIT_TIMES,       
    })
  }

  fn draw_row(&mut self, row: &Row, row_index: usize) -> Result<(), Error> {
    let start = self.cursor_offset.x;
    let end = self.cursor_offset.x + (self.terminal.size().width as usize);    
    let terminal_row = row.render(start, end);
    self.terminal.move_cursor(0, (row_index - self.cursor_offset.y) as u16)?;
    self.terminal.print_string(&terminal_row)        
  }

  fn draw_rows(&mut self) -> Result<(), Error> {        
    for terminal_row_index in 0..self.terminal.size().height.saturating_sub(1) {
      let row_index = (terminal_row_index as usize) + self.cursor_offset.y;
      self.terminal.move_cursor(0, terminal_row_index)?;
      self.terminal.clear_current_line()?;      
      if row_index >= self.document.rows_size() {
        self.terminal.print_string("~\r")?;
      }
      if let Some(row) = self.document.row(row_index as usize) {
        // TODO: replace with draw_row method call (mutable and immutable borrow)
        // self.draw_row(row)?;
        let start = self.cursor_offset.x;
        let end = self.cursor_offset.x + (self.terminal.size().width as usize);    
        let terminal_row = row.render(start, end);
        self.terminal.move_cursor(0, terminal_row_index)?;
        self.terminal.print_string(&terminal_row)?;        
      }
    }
    self.terminal.move_cursor(0, 0)?;

    Ok(())
  }

  fn draw_message_bar(&mut self) -> Result<(), Error> {
    self.terminal.move_cursor(0, self.terminal.size().height.saturating_sub(1))?;
    self.terminal.clear_current_line()?;
    let message = &self.status_message;
    if Instant::now() - message.time < Duration::new(STATUS_MESSAGE_LIVE_TIME, 0) {      
      let mut text = message.text.clone();
      text.truncate(self.terminal.size().width as usize);      
      self.terminal.print_string(&text)?;
    }   

    Ok(())
  }

  fn draw_status_bar(&mut self) -> Result<(), Error> {
    let mut file_name = "[No Name]".to_string();    
    if let Some(path) = &mut self.document.path {
      file_name = path.clone();
      file_name.truncate(20);
    }    
    let mut status = format!("{} -- {} lines", file_name, self.document.rows_size());

    if self.document.is_dirty() {
      status.push_str(" (modified)");
    }

    let width = self.terminal.size().width as usize;
    
    let line_indicator = format!(
      "{}/{}",
      self.cursor_position.y,
      self.cursor_position.x,
    );    

    let len = status.len() + line_indicator.len();
    
    if width > len {
      status.push_str(&" ".repeat(width - len));
    }

    status = format!("{}{}", status, line_indicator);

    status.truncate(width);
    
    self.terminal.set_colors(Colors::new(STATUS_BAR_FG, STATUS_BAR_BG))?;
    
    let x = 0;
    let y = self.terminal.size().height.saturating_sub(2);

    self.terminal.move_cursor(x, y)?;    
    self.terminal.print_string(&status)?;
    self.terminal.reset_colors()?;
    Ok(())
  }

  fn search(&mut self) {
    let old_position = self.cursor_position.clone();
    let mut search_dir = SearchDir::Forward;
    
    let query = self
      .prompt("Search: ", |editor, key_event, query| {
        let mut moved = false;

        match key_event.code {
          KeyCode::Right | KeyCode::Down => {
            search_dir = SearchDir::Forward;
            editor.process_move(KeyCode::Right)?;
            moved = true;
          },
          KeyCode::Up | KeyCode::Left => search_dir = SearchDir::Backward,
          _ => search_dir = SearchDir::Forward,
        }  

        if let Some(position) = editor.document.find(&query[..], &editor.cursor_position, search_dir) {
          editor.cursor_position = position;
          editor.scroll();         
        } else if moved {
          editor.process_move(KeyCode::Left)?;
        }

        Ok(())
      }).unwrap_or(None); 

    if query.is_none() {      
      self.status_message = StatusMessage::from("Find aborted".to_string());
      self.cursor_position = old_position;
      self.scroll();
    }
  }

  fn prompt<C>(&mut self, prompt: &str, mut callback: C) -> Result<Option<String>, Error>
  where
    C: FnMut(&mut Self, KeyEvent, &String) -> Result<(), Error>
  {
    let mut result = String::new();
    let mut run_prompt = true;
    while run_prompt {
      self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
      self.refresh_screen()?;
      
      if let Some(event) = self.terminal.read_event()? {
        match event {
          Event::Key(key_event) => {
            match key_event {
              KeyEvent{code: KeyCode::Char('j'), modifiers: KeyModifiers::CONTROL, ..}
                | KeyEvent{code: KeyCode::Enter, ..} => {
                self.status_message = StatusMessage::from(String::new());
                run_prompt = false; 
              },              
              _ => match key_event.code {
                KeyCode::Char(c) => {
                  result.push(c);
                },
                KeyCode::Backspace => {
                  result.pop();
                },
                KeyCode::Esc => {
                  result.truncate(0);
                  run_prompt = false;
                },
                _ => {}
              }              
            }
            callback(self, key_event, &result)?;
          },
          _ => {}          
        }        
      }
    }

    if result.is_empty() {
      Ok(None)
    } else {
      Ok(Some(result))
    }    
  }

  fn draw_welcome_message(&mut self) -> Result<(), Error> {
    let mut message = format!("Slime editor -- version {}", VERSION);
    let width = self.terminal.size().width;
    let height = self.terminal.size().height;
    let len = message.len();
    let pos_x = width.saturating_sub(len as u16) / 2;
    let pos_y = height / 2;
    self.terminal.move_cursor(pos_x, pos_y)?;
    message.truncate(width as usize);
    self.terminal.print_string(&message)?;    
    self.terminal.move_cursor(0, 0)
  }

  fn refresh_screen(&mut self) -> Result<(), Error> {  
    self.terminal.hide_cursor()?;
    self.terminal.move_cursor(0, 0)?;

    if self.should_quit {            
      self.terminal.clear_screen()?;      
    } else {
      self.draw_rows()?;      
      self.draw_status_bar()?;
      self.draw_message_bar()?;
      self.terminal.move_cursor(
        self.cursor_position.x.saturating_sub(self.cursor_offset.x) as u16, 
        self.cursor_position.y.saturating_sub(self.cursor_offset.y) as u16)?;

      if self.document.is_empty() {
        self.draw_welcome_message()?;
      } 
    }           

    self.terminal.show_cursor()?;

    Ok(())
  }

  fn process_event(&mut self, event: Event) -> Result<(), Error> {  
    match event {
      Event::Key(event) => {
        self.process_keyboard(event)?
      },
      Event::Resize(new_cols, new_rows) => {
        self.terminal.resize(new_cols, new_rows);        

        self.refresh_screen()?
      }
      _ => {}
    }

    Ok(())
  }

  fn save(&mut self) {
    if self.document.path.is_none() {
      let file_name = self.prompt("Save as: ", |_, _, _| { Ok(()) }).unwrap_or(None);
      if file_name.is_none() {
        self.status_message = StatusMessage::from("Save aborted".to_string());
        return;
      } else {
        self.document.path = Some(file_name.unwrap());
      }
    }
    if self.document.save_to_disk().is_ok() {
      self.status_message = StatusMessage::from("File saved".to_string());
    } else {
      self.status_message = StatusMessage::from("Failed to save file!".to_string());
    }
  }

  fn process_keyboard(&mut self, event: KeyEvent) -> Result<(), Error> {
    match event {
      // KP_ENTER
      KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('j'), ..}
        | KeyEvent{code: KeyCode::Enter, ..} => {
          self.document.insert(&self.cursor_position, '\n');
          self.process_move(KeyCode::Right)?;
      },
      // Ctrl-C
      KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('c'), ..} => {
        if self.quit_times > 0 && self.document.is_dirty() {          
          self.status_message = StatusMessage::from(
            format!(
              "WARNING! File has unsaved changes. Press Ctrl-C {} more times to quit.",
              self.quit_times
            ));          
          self.quit_times -= 1;
          return Ok(());
        }
        self.should_quit = true;                  
      },
      // Ctrl-S
      KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('s'), ..} => self.save(),
      // Ctrl-F
      KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('f'), ..} => self.search(),
      // Ctrl-END
      KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::End, ..} => {
        let last_index = self.document.rows_size().saturating_sub(1);
        if let Some(last_row) = self.document.row(last_index) {
          self.cursor_position = Position {
            x: last_row.size(),
            y: last_index,
          }
        }
      },
      // Ctrl-HOME
      KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Home, ..} => {
        self.cursor_position = Position {x: 0, y: 0};
      },
      _ => match event.code {
        KeyCode::Char(c) => {          
          self.document.insert(&self.cursor_position, c);
          self.process_move(KeyCode::Right)?;                  
        },               
        KeyCode::Backspace => {                
          if !(self.cursor_position.x == 0 && self.cursor_position.y == 0) {
            self.process_move(KeyCode::Left)?;          
            self.document.delete(&self.cursor_position);
          }
        },
        KeyCode::Delete => {
          self.document.delete(&self.cursor_position);        
        },                      
        KeyCode::Up
          | KeyCode::Down
          | KeyCode::Left 
          | KeyCode::Right
          | KeyCode::Home
          | KeyCode::End
          | KeyCode::PageDown
          | KeyCode::PageUp => 
          self.process_move(event.code)?,      
        _ => {}
      }
    }

    if self.quit_times < QUIT_TIMES {
      self.quit_times = QUIT_TIMES;
      self.status_message = StatusMessage::from(String::new());
    }

    self.scroll();

    Ok(())      
  }  

  fn scroll(&mut self) {
    let Position { x, y } = self.cursor_position;
    let mut offset_x = self.cursor_offset.x;
    let mut offset_y = self.cursor_offset.y;
    let terminal_width = self.terminal.size().width as usize;
    let terminal_height = self.terminal.size().height.saturating_sub(2) as usize;      
    let max_x = offset_x.saturating_add(terminal_width);
    let max_y = offset_y.saturating_add(terminal_height);
        
    if x >= max_x {
      offset_x = x.saturating_sub(terminal_width).saturating_add(1);
    } else if x < offset_x {
      offset_x = x;
    }    
    
    if y >= max_y {            
      offset_y = y.saturating_sub(terminal_height).saturating_add(1);
    } else if y < offset_y {
      offset_y = y
    }

    self.cursor_offset = Position{x: offset_x, y: offset_y};    
  }

  fn process_move(&mut self, key: KeyCode) -> Result<(), Error> {    
    let Position { mut x, mut y } = self.cursor_position;
    
    let terminal_height = self.terminal.size().height as usize;
    match key {
      KeyCode::Left => {
        if x > 0 {
          x -= 1;            
        } else if y > 0 {
          y -= 1;
          if let Some(row) = self.document.row(y) {
            x = row.size();
          } else {
            x = 0;
          }
        }        
      },
      KeyCode::Right => {
        if let Some(row) = self.document.row(y) {
          if x < row.size() {            
            x = x.saturating_add(1);
          } else if y < self.document.rows_size().saturating_sub(1) {
            y += 1;
            x = 0;
          }                      
        } else {
          x = 0;
        }
      }
      KeyCode::Up => y = y.saturating_sub(1),
      KeyCode::Down => y = y.saturating_add(1),
      KeyCode::Home => x = 0,
      KeyCode::End => {
        if let Some(row) = self.document.row(y) {
          x = row.size()
        } else {
          x = 0;
        }
      }
      KeyCode::PageDown => y = y.saturating_add(terminal_height.saturating_sub(1)),
      KeyCode::PageUp => y = y.saturating_sub(terminal_height.saturating_sub(1)),
      _ => {},
    }
    if let Some(row) = self.document.row(y) {
      x = x.clamp(0, row.size());
    } else {
      x = 0;
    }
    y = y.clamp(0, self.document.rows_size().saturating_sub(1));
    self.cursor_position = Position{ x, y };
      
    Ok(())
  }

  fn die(&mut self, err: Error) -> Result<(), Error>{
    self.terminal.clear_screen()?;

    panic!("{}", err)    
  }
}
