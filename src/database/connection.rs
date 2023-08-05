use std::fs::File;

use rusqlite::Connection;

use super::{Database, DatabaseError, DatabaseStore};

pub trait DatabaseConnectionHandle {
    fn get_connection(&self) -> Option<&Connection>;
    fn open_connection(&mut self) -> Result<(), DatabaseError>;
    fn close_connection(&mut self) -> Result<(), DatabaseError>;
}

impl DatabaseConnectionHandle for Database {
    fn get_connection(&self) -> Option<&Connection> {
        self.conn.as_ref()
    }

    fn open_connection(&mut self) -> Result<(), DatabaseError> {
        let database_exists = File::open(self.get_database_path()).is_ok();

        if self.conn.is_none() {
            self.conn = Some(Connection::open(self.get_database_path())?);
        }

        if !database_exists {
            self.get_connection().unwrap().execute_batch(include_str!("./schema.sql")).unwrap();
        }

        Ok(())
    }

    fn close_connection(&mut self) -> Result<(), DatabaseError> {
        if self.get_connection().is_some() {
            self.conn = None;
        }

        Ok(())
    }
}
