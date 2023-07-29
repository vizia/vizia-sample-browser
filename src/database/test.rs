use chrono::Utc;
use rand::Rng;

use super::{startup_database, Collection};

#[test]
fn test() {
    let handle = startup_database(".test.vsb").unwrap();

    handle.conn.execute_batch(include_str!("schema.sql")).unwrap();
    // Insert dummy data
    let mut rand_thread = rand::thread_rng();
    for i in 0..100 {
        let tmp_col = Collection {
            id: i,
            name: {
                let rand = &mut rand_thread;
                let vec: Vec<u8> =
                    vec![0u8; 10].iter().map(move |v| *v + rand.gen_range(1..=10)).collect();
                String::from_utf8(vec).unwrap()
            },
            parent_collection: None,
            created_at: Utc::now(),
        };

        handle
            .conn
            .execute(
                "
                INSERT INTO collection (name, created_at) VALUES (?1, ?2)
            ",
                (&tmp_col.name, &tmp_col.created_at),
            )
            .unwrap();
    }

    //Query
    {
        let mut query = handle.conn.prepare("SELECT id, name, created_at FROM collection").unwrap();
        let collection_iter = query
            .query_map([], |row| {
                Ok(Collection {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    parent_collection: None,
                    created_at: row.get(2)?,
                })
            })
            .unwrap();

        for collection in collection_iter {
            println!("{collection:?}");
        }
    }
}
