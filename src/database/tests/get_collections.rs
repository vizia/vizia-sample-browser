#[test]
pub fn get_all_collections() {
    use crate::database::{Collection, DatabaseHandle};
    use rusqlite::Connection;

    let handle = DatabaseHandle::from_connection("", Some(Connection::open_in_memory().unwrap()));
    handle.connection().unwrap().execute_batch(include_str!("../schema.sql")).unwrap();
    handle.connection().unwrap().execute_batch(include_str!("test.sql")).unwrap();

    assert_eq!(
        handle.get_all_collections().unwrap(),
        vec![
            Collection { id: 0, parent_collection: None, name: "Sample Library".to_string() },
            Collection { id: 1, parent_collection: Some(0), name: "Library 1".to_string() },
            Collection { id: 2, parent_collection: Some(0), name: "Library 2".to_string() },
            Collection { id: 3, parent_collection: Some(1), name: "Sub Library 1.1".to_string() },
            Collection { id: 4, parent_collection: Some(1), name: "Sub Library 1.2".to_string() },
        ]
    )
}

#[test]
pub fn get_root_collection() {
    use crate::database::{Collection, DatabaseHandle};
    use rusqlite::Connection;

    let handle = DatabaseHandle::from_connection("", Some(Connection::open_in_memory().unwrap()));
    handle.connection().unwrap().execute_batch(include_str!("../schema.sql")).unwrap();
    handle.connection().unwrap().execute_batch(include_str!("test.sql")).unwrap();

    assert_eq!(
        handle.get_root_collection().unwrap(),
        Collection { id: 0, parent_collection: None, name: "Sample Library".to_string() }
    );
}

#[test]
pub fn get_child_collections() {
    use crate::database::{Collection, DatabaseHandle};
    use rusqlite::Connection;

    let handle = DatabaseHandle::from_connection("", Some(Connection::open_in_memory().unwrap()));
    handle.connection().unwrap().execute_batch(include_str!("../schema.sql")).unwrap();
    handle.connection().unwrap().execute_batch(include_str!("test.sql")).unwrap();

    assert_eq!(
        handle.get_child_collections(0).unwrap(),
        vec![
            Collection { id: 1, parent_collection: Some(0), name: "Library 1".to_string() },
            Collection { id: 2, parent_collection: Some(0), name: "Library 2".to_string() },
        ]
    );

    assert_eq!(
        handle.get_child_collections(1).unwrap(),
        vec![
            Collection { id: 3, parent_collection: Some(1), name: "Sub Library 1.1".to_string() },
            Collection { id: 4, parent_collection: Some(1), name: "Sub Library 1.2".to_string() },
        ]
    );
}
