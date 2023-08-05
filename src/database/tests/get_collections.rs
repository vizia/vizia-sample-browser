use crate::database::{DatabaseCollectionHandler, DatabaseConnectionHandle};

#[test]
pub fn get_all_collections() {
    use crate::database::{Collection, Database};
    use rusqlite::Connection;

    let mut handle = Database::from_connection("", Some(Connection::open_in_memory().unwrap()));
    handle.get_connection().unwrap().execute_batch(include_str!("../schema.sql")).unwrap();
    handle.get_connection().unwrap().execute_batch(include_str!("test.sql")).unwrap();

    assert_eq!(
        handle.get_all_collections().unwrap(),
        vec![
            Collection::new(0, None, "Sample Library".to_string()),
            Collection::new(1, Some(0), "Library 1".to_string()),
            Collection::new(2, Some(0), "Library 2".to_string()),
            Collection::new(3, Some(1), "Sub Library 1.1".to_string()),
            Collection::new(4, Some(1), "Sub Library 1.2".to_string()),
        ]
    );
}

#[test]
pub fn get_root_collection() {
    use crate::database::{Collection, Database};
    use rusqlite::Connection;

    let mut handle = Database::from_connection("", Some(Connection::open_in_memory().unwrap()));
    handle.get_connection().unwrap().execute_batch(include_str!("../schema.sql")).unwrap();
    handle.get_connection().unwrap().execute_batch(include_str!("test.sql")).unwrap();

    assert_eq!(
        handle.get_root_collection().unwrap(),
        Collection::new(0, None, "Sample Library".to_string())
    );
}

#[test]
pub fn get_child_collections() {
    use crate::database::{Collection, Database};
    use rusqlite::Connection;

    let mut handle = Database::from_connection("", Some(Connection::open_in_memory().unwrap()));
    handle.get_connection().unwrap().execute_batch(include_str!("../schema.sql")).unwrap();
    handle.get_connection().unwrap().execute_batch(include_str!("test.sql")).unwrap();

    assert_eq!(
        handle.get_child_collections(0).unwrap(),
        vec![
            Collection::new(1, Some(0), "Library 1".to_string()),
            Collection::new(2, Some(0), "Library 2".to_string()),
        ]
    );

    assert_eq!(
        handle.get_child_collections(1).unwrap(),
        vec![
            Collection::new(3, Some(1), "Sub Library 1.1".to_string()),
            Collection::new(4, Some(1), "Sub Library 1.2".to_string()),
        ]
    );
}
