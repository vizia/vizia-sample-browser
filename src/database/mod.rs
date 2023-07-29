use chrono::{DateTime, Utc};
use rand::Rng;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Collection {
    name: String,
    sub_collections: Option<Vec<Collection>>,
    audio_files: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AudioFiles {
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
struct Tag {
    id: String,
    color: f32,
}

pub fn startup_database() -> rusqlite::Result<()> {
    let connection = Connection::open_in_memory()?;

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

    //Query
    let mut query = connection.prepare("SELECT name, audio_files, created_at FROM collection")?;
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

    Ok(())
}
