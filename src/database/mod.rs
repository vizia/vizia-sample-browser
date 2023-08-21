use super::*;
use crate::state::browser::Directory;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{
    any::Any,
    cell::RefCell,
    collections::{hash_map::DefaultHasher, BTreeSet, HashMap, VecDeque},
    error::Error,
    fs::{create_dir, read_dir, DirEntry, File},
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    rc::Rc,
    sync::atomic::AtomicUsize,
};
use vizia::prelude::*;

pub mod connection;
pub use connection::*;

pub mod store;
pub use store::*;

pub mod collection;
pub use collection::*;

pub mod error;
pub use error::*;

pub mod audio_file;
pub use audio_file::*;

pub mod tags;
pub use tags::*;

pub mod meta;
pub use meta::*;

pub mod comparator;
pub use comparator::*;

mod tests;

pub mod prelude {
    pub use super::audio_file::*;
    pub use super::collection::*;
    pub use super::connection::*;
    pub use super::error::*;
    pub use super::meta::*;
    pub use super::store::*;
    pub use super::tags::*;
    pub use super::*;
    pub use rusqlite::*;
}

fn file_exists(path: &PathBuf) -> bool {
    std::fs::read(path).is_ok()
}

fn directory_exists(path: &PathBuf) -> bool {
    std::fs::read_dir(path).is_ok()
}

pub const DATABASE_FILE_NAME: &str = ".database.vsb";
pub const AUDIO_FILE_EXTENSIONS: [&str; 1] = ["wav"];

#[derive(Debug, Lens)]
pub struct Database {
    pub path: PathBuf,
    pub conn: Option<Connection>,
    pub meta: DatabaseMetadata,
}

impl Database {
    pub fn from_connection(path: &str, connection: Option<Connection>) -> Self {
        Database {
            path: Path::new(path).to_path_buf(),
            conn: connection,
            meta: DatabaseMetadata::new(),
        }
    }

    pub fn from_directory(path: PathBuf) -> Result<Self, DatabaseError> {
        // Check file is directoryS
        if !directory_exists(&path) {
            return Err(DatabaseError::PathNotDirectory);
        }

        let mut s: Database = Self { path, conn: None, meta: DatabaseMetadata::new() };

        let directory_created = directory_exists(&s.get_meta_directory_path());

        if directory_created {
            s.meta = ron::from_str(&std::fs::read_to_string(s.get_meta_path()).unwrap()).unwrap();
        } else {
            create_dir(s.get_meta_directory_path());
        }

        s.open_connection()?;

        if directory_created {
            s.update_database();
        } else {
            s.initialize_empty_database();
        }

        s.store_metadata();

        Ok(s)
    }

    fn clear_database(&mut self) {
        self.get_connection().unwrap().execute_batch(include_str!("sqls/clear.sql")).unwrap();
    }

    fn initialize_empty_database(&mut self) -> Result<(), std::io::Error> {
        let mut audio_file_count = 0;
        let mut collection_count = 0;

        let connection = self.get_connection().unwrap();

        let mut dir_stack: VecDeque<(PathBuf, Option<PathBuf>, Option<usize>)> = VecDeque::new();
        dir_stack.push_back((self.path.clone(), None, None));

        while let Some((path, parent, parent_id)) = dir_stack.pop_front() {
            let read_dir = read_dir(&path)?;

            let mut child_directories = VecDeque::new();
            let mut child_files = Vec::new();

            read_dir.filter_map(|v| v.ok()).for_each(|v| match v.metadata().unwrap().is_dir() {
                true => child_directories.push_back(v),
                false => child_files.push(v),
            });

            //

            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            if name.starts_with('.') {
                continue;
            }

            let id = collection_count;
            collection_count += 1;

            for child_dir in child_directories {
                dir_stack.push_back((child_dir.path(), Some(path.clone()), Some(id)));
            }

            // Insert collection
            let collection = Collection::new(id, parent_id, name, path.clone());

            self.insert_collection(collection);

            // Insert each non-directory child
            for child_file in child_files {
                let audio_file =
                    AudioFile::from_path(&child_file.path(), audio_file_count).unwrap();
                audio_file_count += 1;

                self.insert_audio_file(audio_file);
            }
        }
        // Recursively check each directory under the root

        self.meta.last_collection_id = collection_count;
        self.meta.last_audio_file = audio_file_count;

        Ok(())
    }

    pub fn close_database(&mut self) {
        self.store_metadata();
        self.close_connection().unwrap();
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        // let meta_dir = self.get_meta_directory_path();
        // std::fs::remove_dir_all(meta_dir);
    }
}

impl PartialEq for Database {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self.conn.is_some() == other.conn.is_some()
            && self.meta == other.meta
    }
}
