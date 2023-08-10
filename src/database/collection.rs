use super::{Database, DatabaseConnectionHandle, DatabaseError};
use serde::{Deserialize, Serialize};

pub type CollectionID = usize;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Collection {
    id: CollectionID,
    parent_collection: Option<CollectionID>,
    name: String,
}

impl Collection {
    pub fn new(id: CollectionID, parent_collection: Option<CollectionID>, name: String) -> Self {
        Self { id, parent_collection, name }
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
}

pub trait DatabaseCollectionHandler {
    fn get_root_collection(&self) -> Result<Collection, DatabaseError>;
    fn get_all_collections(&self) -> Result<Vec<Collection>, DatabaseError>;
    fn get_child_collections(&self, parent: CollectionID)
        -> Result<Vec<Collection>, DatabaseError>;
    fn insert_collection(&mut self, collection: Collection) -> Result<(), DatabaseError>;
}

impl DatabaseCollectionHandler for Database {
    fn get_root_collection(&self) -> Result<Collection, DatabaseError> {
        if let Some(connection) = self.get_connection() {
            let mut query = connection.prepare(
                "SELECT id, parent_collection, name FROM collections WHERE parent_collection IS NULL",
            )?;

            let col: Collection = query.query_row([], |row| {
                Ok(Collection { id: row.get(0)?, parent_collection: None, name: row.get(2)? })
            })?;

            return Ok(col);
        }

        Err(DatabaseError::ConnectionClosed)
    }

    fn get_all_collections(&self) -> Result<Vec<Collection>, DatabaseError> {
        if let Some(connection) = self.get_connection() {
            let mut query =
                connection.prepare("SELECT id, parent_collection, name FROM collections")?;

            let collections = query.query_map([], |row| {
                Ok(Collection {
                    id: row.get(0)?,
                    parent_collection: row.get(1)?,
                    name: row.get(2)?,
                })
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
                "SELECT id, parent_collection, name FROM collections WHERE parent_collection = (?1)",
            )?;

            let collections = query.query_map([parent], |row| {
                Ok(Collection {
                    id: row.get(0)?,
                    name: row.get(2)?,
                    parent_collection: row.get(1)?,
                })
            })?;

            return Ok(collections.map(|v| v.unwrap()).collect());
        }

        Err(DatabaseError::ConnectionClosed)
    }

    fn insert_collection(&mut self, collection: Collection) -> Result<(), DatabaseError> {
        if let Some(connection) = self.get_connection() {
            connection.execute(
                "INSERT INTO collections (id, parent_collection, name) VALUES (?1, ?2, ?3)",
                (collection.id, collection.parent_collection, collection.name),
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
