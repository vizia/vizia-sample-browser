use chrono::{DateTime, Utc};
use rand::Rng;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    name: String,
    sub_collections: Option<Vec<Collection>>,
    audio_files: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFiles {
    location: String,
    tags: Vec<Tag>,
    duration: f32,
    sample_rate: f32,
    bit_depth: f32,
    bpm: Option<f32>,
    key: Option<f32>,
    size: f32,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    id: String,
    color: f32,
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

        // Insert dummy data
        let mut rand_thread = rand::thread_rng();
        for _ in 0..100 {
            let tmp_col = Collection {
                name: {
                    let rand = &mut rand_thread;
                    let vec: Vec<u8> =
                        vec![0u8; 10].iter().map(move |v| *v + rand.gen_range(1..=10)).collect();
                    String::from_utf8(vec).unwrap()
                },
                audio_files: None,
                sub_collections: None,
                created_at: Utc::now(),
            };

            connection.execute(
                "
                INSERT INTO collection (name, created_at) VALUES (?1, ?2)
            ",
                (&tmp_col.name, &tmp_col.created_at),
            )?;
        }
    }

    //Query
    {
        let mut query =
            connection.prepare("SELECT name, audio_files, created_at FROM collection")?;
        let collection_iter = query.query_map([], |row| {
            Ok(Collection {
                name: row.get(0)?,
                audio_files: row.get(1)?,
                created_at: row.get(2)?,
                sub_collections: None,
            })
        })?;

        for collection in collection_iter {
            println!("{collection:?}");
        }
    }

    Ok(DatabaseHandle { rel_path: path, conn: connection })
}
