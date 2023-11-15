#![warn(clippy::all, clippy::pedantic, clippy::restriction)]            
mod editor;

use core::panic;

use editor::Editor;
mod terminal;
mod row;
mod document;
pub use row::Row;
pub use document::Document;
pub use terminal::Terminal;
pub use editor::Position;

fn main() -> std::io::Result<()> {
    let res = Editor::default();

    match res {
        Err(err) => {
            panic!("{}", err)
        },
        Ok(mut editor) => {
            editor.run()
        }
    }    
}