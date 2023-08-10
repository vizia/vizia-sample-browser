#[test]
fn insert_from_directory() {
    use crate::database::prelude::*;

    let mut handle = Database::from_connection("", Some(Connection::open_in_memory().unwrap()));
    handle.get_connection().unwrap().execute_batch(include_str!("../sqls/schema.sql")).unwrap();
    handle.get_connection().unwrap().execute_batch(include_str!("../sqls/test.sql")).unwrap();

    for col in handle.get_all_collections().unwrap() {
        println!("{:?}", col)
    }

    for audio_file in handle.get_all_audio_files().unwrap() {
        println!("{:?}", audio_file);
    }

    assert!(true)
}
