use std::{io::{Error, self}, time::Duration};

use crossterm::{
  terminal::{self, Clear},
  cursor::{MoveTo, Hide, Show},
  ExecutableCommand,
  style::{Print, SetColors, Colors, Color, SetForegroundColor, SetBackgroundColor}, 
  event::{Event, poll, read}};

pub struct Size {
  pub width: u16,
  pub height: u16,
}

pub struct Terminal {
  stdout: io::Stdout,
  // terminal size
  size: Size,
}

impl Terminal {
  pub fn default() -> Result<Terminal, Error> {
    let stdout = io::stdout();
    let _raw_mode = terminal::enable_raw_mode();    
    let (cols, rows) = terminal::size()?;

    Ok(Terminal{
      stdout,
      size: Size { width: cols, height: rows }
    })
  }  

  pub fn size(&self) -> &Size {
    &self.size
  }

  pub fn resize(&mut self, width: u16, height: u16) {
    self.size.width = width;
    self.size.height = height;
  }

  pub fn move_cursor(&mut self, x: u16, y: u16) -> Result<(), Error> {
    self.stdout.execute(MoveTo(x, y))?;

    Ok(())
  }

  pub fn hide_cursor(&mut self) -> Result<(), Error> {
    self.stdout.execute(Hide)?;

    Ok(())
  }

  pub fn show_cursor(&mut self) -> Result<(), Error> {
    self.stdout.execute(Show)?;

    Ok(())
  }

  pub fn print_char(&mut self, ch: char) -> Result<(), Error> {
    self.stdout.execute(Print(ch))?;

    Ok(())    
  }

  pub fn print_string(&mut self, str: &str) -> Result<(), Error> {
    self.stdout.execute(Print(str))?;

    Ok(())
  }

  pub fn clear_screen(&mut self) -> Result<(), Error> {
    self.stdout
      .execute(Clear(terminal::ClearType::All))?
      .execute(MoveTo(0, 0))?;

    Ok(())
  }  

  pub fn read_event(&self) -> Result<Option<Event>, Error> {
    if poll(Duration::from_millis(100))? {
      match read() {
        Ok(e) => {          
          
          return Ok(Some(e));
        },
        Err(err) => {
          return Err(err);
        }
      }      
    }

    Ok(None)
  }

  pub fn clear_current_line(&mut self) -> Result<(), Error> {
    self.stdout.execute(Clear(terminal::ClearType::CurrentLine))?;

    Ok(())
  }

  pub fn set_colors(&mut self, colors: Colors) -> Result<(), Error> {
    self.stdout.execute(SetColors(colors))?;

    Ok(())
  }
  pub fn reset_colors(&mut self) -> Result<(), Error> {
    self.stdout.execute(SetColors(Colors::new(Color::Reset, Color::Reset)))?;

    Ok(())
  }
  pub fn set_fg_color(&mut self, color: Color) -> Result<(), Error> {
    self.stdout.execute(SetForegroundColor(color))?;

    Ok(())
  }
  pub fn reset_fg_color(&mut self) -> Result<(), Error> {
    self.stdout.execute(SetForegroundColor(Color::Reset))?;

    Ok(())
  }
  pub fn set_bg_color(&mut self, color: Color) -> Result<(), Error> {
    self.stdout.execute(SetBackgroundColor(color))?;

    Ok(())
  }
  pub fn reset_bg_color(&mut self) -> Result<(), Error> {
    self.stdout.execute(SetBackgroundColor(Color::Reset))?;

    Ok(())
  }
}
