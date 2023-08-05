use crate::database::DatabaseConnectionHandle;

#[test]
fn local_query() {
    use crate::database::{Collection, Database};
    use rand::Rng;
    use rusqlite::Connection;

    let mut handle = Database::from_connection("", Some(Connection::open_in_memory().unwrap()));
    handle.get_connection().unwrap().execute_batch(include_str!("../schema.sql")).unwrap();
    // Insert dummy data
    let mut rand_thread = rand::thread_rng();
    for i in 0..100 {
        let tmp_col = Collection::new(i, None, {
            let rand = &mut rand_thread;
            let vec: Vec<u8> =
                vec![0u8; 10].iter().map(move |v| *v + rand.gen_range(1..=10)).collect();
            String::from_utf8(vec).unwrap()
        });

        handle
            .get_connection()
            .unwrap()
            .execute(
                "
                INSERT INTO collections (name) VALUES (?1)
            ",
                [&tmp_col.name()],
            )
            .unwrap();
    }

    //Query
    {
        let mut query =
            handle.get_connection().unwrap().prepare("SELECT id, name FROM collections").unwrap();
        let collection_iter =
            query.query_map([], |row| Ok(Collection::new(row.get(0)?, None, row.get(1)?))).unwrap();

        for collection in collection_iter {
            println!("{collection:?}");
        }
    }
}
