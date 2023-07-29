#[test]
pub fn get_all_collections() {
    use crate::database::{Collection, DatabaseHandle};
    use rusqlite::Connection;

    let handle = DatabaseHandle { conn: Connection::open_in_memory().unwrap(), rel_path: "" };
    handle.conn.execute_batch(include_str!("../schema.sql")).unwrap();
    handle.conn.execute_batch(include_str!("test.sql")).unwrap();

    assert_eq!(
        handle.get_all_collections(),
        Ok(vec![
            Collection { id: 5, parent_collection: None, name: "Sample Library".to_string() },
            Collection { id: 1, parent_collection: Some(0), name: "Library 1".to_string() },
            Collection { id: 2, parent_collection: Some(0), name: "Library 2".to_string() },
            Collection { id: 3, parent_collection: Some(1), name: "Sub Library 1.1".to_string() },
            Collection { id: 4, parent_collection: Some(1), name: "Sub Library 1.2".to_string() },
        ])
    )
}

#[test]
pub fn get_root_collection() {
    use crate::database::{Collection, DatabaseHandle};
    use rusqlite::Connection;

    let handle = DatabaseHandle { conn: Connection::open_in_memory().unwrap(), rel_path: "" };
    handle.conn.execute_batch(include_str!("../schema.sql")).unwrap();
    handle.conn.execute_batch(include_str!("test.sql")).unwrap();

    assert_eq!(
        handle.get_root_collection(),
        Ok(Collection { id: 5, parent_collection: None, name: "Sample Library".to_string() })
    );
}

#[test]
pub fn get_child_collections() {
    use crate::database::{Collection, DatabaseHandle};
    use rusqlite::Connection;

    let handle = DatabaseHandle { conn: Connection::open_in_memory().unwrap(), rel_path: "" };
    handle.conn.execute_batch(include_str!("../schema.sql")).unwrap();
    handle.conn.execute_batch(include_str!("test.sql")).unwrap();

    assert_eq!(
        handle.get_child_collections(5),
        Ok(vec![
            Collection { id: 1, parent_collection: Some(5), name: "Library 1".to_string() },
            Collection { id: 2, parent_collection: Some(5), name: "Library 2".to_string() },
        ])
    );

    assert_eq!(
        handle.get_child_collections(1),
        Ok(vec![
            Collection { id: 3, parent_collection: Some(1), name: "Sub Library 1.1".to_string() },
            Collection { id: 4, parent_collection: Some(1), name: "Sub Library 1.2".to_string() },
        ])
    );
}
