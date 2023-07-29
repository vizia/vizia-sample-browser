use chrono::{DateTime, Utc};
use rand::Rng;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

mod test;

pub type CollectionID = i32;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    id: CollectionID,
    parent_collection: Option<CollectionID>,
    name: String,
    created_at: DateTime<Utc>,
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
    created_at: DateTime<Utc>,
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
    // Abstract away sql queries here
}

pub fn startup_database(path: &str) -> rusqlite::Result<DatabaseHandle> {
    let connection = Connection::open(path)?;

    if fs::metadata(path).is_err() {
        // Define tables
        connection.execute_batch(include_str!("schema.sql"))?;
    }

    Ok(DatabaseHandle { rel_path: path, conn: connection })
}
