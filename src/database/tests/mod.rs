use super::prelude::Database;

pub mod get_audio_files;
pub mod get_collections;
pub mod get_tags;
pub mod insert;
pub mod recursive_benchmark;

const TEST_DIRECTORY: &str = "test_files/";
const TEST_META_DIRECTORY: &str = "test_files/.vsb-meta/";
const TEST_META: &str = "test_files/.vsb-meta/.vsb-meta";
const TEST_DATABASE: &str = "test_files/.vsb-meta/.vsb-database";

fn init_test_database() -> Database {
    use crate::database::*;
    use rusqlite::Connection;

    let mut handle = Database::from_connection("", Some(Connection::open_in_memory().unwrap()));
    handle.get_connection().unwrap().execute_batch(include_str!("../sqls/schema.sql")).unwrap();
    handle.get_connection().unwrap().execute_batch(include_str!("../sqls/test.sql")).unwrap();

    handle
}

pub fn check_meta_directory_exists() -> bool {
    std::fs::read_dir(TEST_META_DIRECTORY).is_ok()
}

pub fn check_meta_exists() -> bool {
    std::fs::read(TEST_META).is_ok()
}

pub fn check_database_exists() -> bool {
    std::fs::read(TEST_DATABASE).is_ok()
}
