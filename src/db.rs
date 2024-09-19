use rusqlite::{Connection, Result};

pub struct Database;

impl Database {
    pub fn new() -> Result<Connection> {
        Connection::open("epub_editor.db")
    }

    // 数据库操作方法
}
