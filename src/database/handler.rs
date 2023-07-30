use std::{
    any::Any,
    error::Error,
    fs::{read_dir, File},
    path::{Path, PathBuf},
};

use rusqlite::Connection;

use super::*;

#[derive(Debug)]
pub struct DatabaseHandle {
    path: PathBuf,
    conn: Option<Connection>,
    children: Vec<DatabaseHandle>,
}

#[derive(Debug)]
pub enum DatabaseHandleError {
    ConnectionClosed,
    PathNotDirecotry,
    PathWithoutMetadata,
    Other(Box<dyn Any>),
}

impl<E> From<E> for DatabaseHandleError
where
    E: Error + Any + Sized,
{
    fn from(value: E) -> Self {
        Self::Other(Box::new(value))
    }
}

impl DatabaseHandle {
    pub fn from_directory(path: PathBuf) -> Result<Self, DatabaseHandleError> {
        let file = match File::open(&path) {
            Ok(f) => f,
            Err(e) => return Err(DatabaseHandleError::Other(Box::new(e))),
        };

        if file.metadata().is_err() {
            return Err(DatabaseHandleError::PathWithoutMetadata);
        }

        if !file.metadata().unwrap().file_type().is_dir() {
            return Err(DatabaseHandleError::PathNotDirecotry);
        }

        Ok(Self { path, conn: None, children: Vec::new() })
    }

    pub fn from_directory_recursive(path: PathBuf) -> Result<Self, DatabaseHandleError> {
        let file = match File::open(&path) {
            Ok(f) => f,
            Err(e) => return Err(DatabaseHandleError::Other(Box::new(e))),
        };

        if file.metadata().is_err() {
            return Err(DatabaseHandleError::PathWithoutMetadata);
        }

        if !file.metadata().unwrap().file_type().is_dir() {
            return Err(DatabaseHandleError::PathNotDirecotry);
        }

        let mut children_handles = Vec::new();

        for child_file in read_dir(&path).unwrap() {
            if let Ok(entry) = child_file {
                if entry.metadata().unwrap().is_dir() {
                    if let Ok(handle) = Self::from_directory_recursive(entry.path()) {
                        children_handles.push(handle);
                    }
                }
            }
        }

        Ok(Self { path, conn: None, children: children_handles })
    }

    pub fn from_connection(path: &str, connection: Option<Connection>) -> Self {
        DatabaseHandle {
            path: Path::new(path).to_path_buf(),
            conn: connection,
            children: Vec::new(),
        }
    }

    pub fn open_connection(&mut self) -> rusqlite::Result<()> {
        if self.conn.is_none() {
            self.conn = Some(Connection::open(self.get_path())?);
        }
        Ok(())
    }

    pub fn close_connection(&mut self) {
        if let Some(c) = self.connection() {
            drop(c);
            self.conn = None;
        }
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }

    pub fn connection(&self) -> Option<&Connection> {
        self.conn.as_ref()
    }

    fn check_connection(&self) -> Result<(), DatabaseHandleError> {
        if self.conn.is_none() {
            return Err(DatabaseHandleError::ConnectionClosed);
        }

        Ok(())
    }

    pub fn get_root_collection(&self) -> Result<Collection, DatabaseHandleError> {
        self.check_connection()?;

        let mut query = self.connection().unwrap().prepare(
            "SELECT id, parent_collection, name FROM collection WHERE parent_collection IS NULL",
        )?;

        let col: Collection = query.query_row([], |row| {
            Ok(Collection { id: row.get(0)?, parent_collection: None, name: row.get(2)? })
        })?;

        Ok(col)
    }

    pub fn get_all_collections(&self) -> Result<Vec<Collection>, DatabaseHandleError> {
        self.check_connection()?;

        let mut query = self
            .connection()
            .unwrap()
            .prepare("SELECT id, parent_collection, name FROM collection")?;
        let collections = query.query_map([], |row| {
            Ok(Collection { id: row.get(0)?, name: row.get(2)?, parent_collection: row.get(1)? })
        })?;
        Ok(collections.map(|v| v.unwrap()).collect())
    }

    pub fn get_child_collections(
        &self,
        parent: CollectionID,
    ) -> Result<Vec<Collection>, DatabaseHandleError> {
        self.check_connection()?;

        let mut query = self.connection().unwrap().prepare(
            "SELECT id, parent_collection, name FROM collection WHERE parent_collection = (?1)",
        )?;
        let collections = query.query_map([parent], |row| {
            Ok(Collection { id: row.get(0)?, name: row.get(2)?, parent_collection: row.get(1)? })
        })?;
        Ok(collections.map(|v| v.unwrap()).collect())
    }
}
