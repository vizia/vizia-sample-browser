#[test]
fn test() {
    use crate::database::{Collection, DatabaseHandle};
    use rand::Rng;
    use rusqlite::Connection;

    let handle = DatabaseHandle { conn: Connection::open_in_memory().unwrap(), rel_path: "" };

    handle.conn.execute_batch(include_str!("../schema.sql")).unwrap();
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
        };

        handle
            .conn
            .execute(
                "
                INSERT INTO collection (name) VALUES (?1)
            ",
                [&tmp_col.name],
            )
            .unwrap();
    }

    //Query
    {
        let mut query = handle.conn.prepare("SELECT id, name FROM collection").unwrap();
        let collection_iter = query
            .query_map([], |row| {
                Ok(Collection { id: row.get(0)?, name: row.get(1)?, parent_collection: None })
            })
            .unwrap();

        for collection in collection_iter {
            println!("{collection:?}");
        }
    }
}
