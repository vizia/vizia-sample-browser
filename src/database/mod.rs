use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;

mod tests;

pub type CollectionID = i32;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Collection {
    id: CollectionID,
    parent_collection: Option<CollectionID>,
    name: String,
}

pub type AudioFileID = i32;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFiles {
    id: AudioFileID,
    collection: CollectionID,
    duration: f32,
    sample_rate: f32,
    bit_depth: f32,
    bpm: Option<f32>,
    key: Option<f32>,
    size: f32,
}

pub type TagID = String;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    id: TagID,
    color: f32,
}

struct AudioFilesTags {
    audio_file: AudioFileID,
    tag: TagID,
}

pub struct DatabaseHandle<'a> {
    rel_path: &'a str,
    conn: Connection,
}

impl<'a> DatabaseHandle<'a> {
    pub fn get_root_collection(&self) -> rusqlite::Result<Collection> {
        let mut query = self.conn.prepare(
            "SELECT id, parent_collection, name FROM collection WHERE parent_collection IS NULL",
        )?;

        let col: Collection = query.query_row([], |row| {
            Ok(Collection { id: row.get(0)?, parent_collection: None, name: row.get(2)? })
        })?;

        Ok(col)
    }

    pub fn get_all_collections(&self) -> rusqlite::Result<Vec<Collection>> {
        let mut query = self.conn.prepare("SELECT id, parent_collection, name FROM collection")?;
        let collections = query.query_map([], |row| {
            Ok(Collection { id: row.get(0)?, name: row.get(2)?, parent_collection: row.get(1)? })
        })?;
        Ok(collections.map(|v| v.unwrap()).collect())
    }

    pub fn get_child_collections(&self, parent: CollectionID) -> rusqlite::Result<Vec<Collection>> {
        let mut query = self.conn.prepare(
            "SELECT id, parent_collection, name FROM collection WHERE parent_collection = (?1)",
        )?;
        let collections = query.query_map([parent], |row| {
            Ok(Collection { id: row.get(0)?, name: row.get(2)?, parent_collection: row.get(1)? })
        })?;
        Ok(collections.map(|v| v.unwrap()).collect())
    }
}

pub fn startup_database(path: &str) -> rusqlite::Result<DatabaseHandle> {
    let connection = Connection::open(path)?;

    if fs::metadata(path).is_err() {
        // Define tables
        connection.execute_batch(include_str!("schema.sql"))?;
    }

    Ok(DatabaseHandle { rel_path: path, conn: connection })
}
