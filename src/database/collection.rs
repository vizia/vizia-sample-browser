use std::path::{Path, PathBuf};

use super::{Database, DatabaseConnectionHandle, DatabaseError};
use serde::{Deserialize, Serialize};

pub type CollectionID = usize;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Collection {
    id: CollectionID,
    parent_collection: Option<CollectionID>,
    name: String,
    path: PathBuf,
}

impl Collection {
    pub fn new(
        id: CollectionID,
        parent_collection: Option<CollectionID>,
        name: String,
        path: PathBuf,
    ) -> Self {
        Self { id, parent_collection, name, path }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn parent_collection(&self) -> Option<usize> {
        self.parent_collection
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

pub trait DatabaseCollectionHandler {
    fn get_collection(&self, id: CollectionID) -> Result<Collection, DatabaseError>;
    fn get_root_collection(&self) -> Result<Collection, DatabaseError>;
    fn get_all_collections(&self) -> Result<Vec<Collection>, DatabaseError>;
    fn get_child_collections(&self, parent: CollectionID)
        -> Result<Vec<Collection>, DatabaseError>;
    fn insert_collection(&mut self, collection: Collection) -> Result<(), DatabaseError>;
}

impl DatabaseCollectionHandler for Database {
    fn get_collection(&self, id: CollectionID) -> Result<Collection, DatabaseError> {
        if let Some(connection) = self.get_connection() {
            let mut query = connection.prepare(
                "SELECT id, parent_collection, name, path FROM collections WHERE id = (?1)",
            )?;

            let collection = query.query_row([id], |row| {
                let path: String = row.get(3)?;
                Ok(Collection::new(
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    Path::new(&path).to_path_buf(),
                ))
            })?;

            return Ok(collection);
        }

        Err(DatabaseError::ConnectionClosed)
    }

    fn get_root_collection(&self) -> Result<Collection, DatabaseError> {
        if let Some(connection) = self.get_connection() {
            let mut query = connection.prepare(
                "SELECT id, parent_collection, name, path FROM collections WHERE parent_collection IS NULL",
            )?;

            let col: Collection = query.query_row([], |row| {
                let path: String = row.get(3)?;
                Ok(Collection::new(
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    Path::new(&path).to_path_buf(),
                ))
            })?;

            return Ok(col);
        }

        Err(DatabaseError::ConnectionClosed)
    }

    fn get_all_collections(&self) -> Result<Vec<Collection>, DatabaseError> {
        if let Some(connection) = self.get_connection() {
            let mut query =
                connection.prepare("SELECT id, parent_collection, name, path FROM collections")?;

            let collections = query.query_map([], |row| {
                Ok(Collection::new(
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    Path::new(&{
                        let s: String = row.get(3)?;
                        s
                    })
                    .to_path_buf(),
                ))
            })?;

            return Ok(collections.map(|v| v.unwrap()).collect());
        }

        Err(DatabaseError::ConnectionClosed)
    }

    fn get_child_collections(
        &self,
        parent: CollectionID,
    ) -> Result<Vec<Collection>, DatabaseError> {
        if let Some(connection) = self.get_connection() {
            let mut query = connection.prepare(
                "SELECT id, parent_collection, name, path FROM collections WHERE parent_collection = (?1)",
            )?;

            let collections = query.query_map([parent], |row| {
                let path: String = row.get(3)?;
                Ok(Collection::new(
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    Path::new(&path).to_path_buf(),
                ))
            })?;

            return Ok(collections.map(|v| v.unwrap()).collect());
        }

        Err(DatabaseError::ConnectionClosed)
    }

    fn insert_collection(&mut self, collection: Collection) -> Result<(), DatabaseError> {
        if let Some(connection) = self.get_connection() {
            connection.execute(
                "INSERT INTO collections (id, parent_collection, name, path) VALUES (?1, ?2, ?3, ?4)",
                (collection.id, collection.parent_collection, collection.name, collection.path.to_str().unwrap()),
            )?;
        }

        Ok(())
    }
}

impl From<Collection> for usize {
    fn from(value: Collection) -> Self {
        value.id
    }
}
