use super::{Database, DatabaseError};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{create_dir, File},
    path::PathBuf,
};
use vizia::prelude::*;

pub const DATABASE_META_DIRECTORY_NAME: &str = ".vsb-meta/";
pub const DATABASE_DATABASE_NAME: &str = ".vsb-database";
pub const DATABASE_META_NAME: &str = ".vsb-meta";

pub type Hash = String;

#[derive(Clone, Debug, Serialize, Deserialize, Lens, PartialEq)]
pub struct DatabaseMetadata {
    pub(super) map: HashMap<PathBuf, Hash>,
}

impl DatabaseMetadata {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    pub fn need_update(&self, other: &Self) -> bool {
        let mut self_collect: Vec<PathBuf> = self.map.keys().map(|v| v.clone()).collect();
        let mut other_collect: Vec<PathBuf> = other.map.keys().map(|v| v.clone()).collect();

        self_collect.len() != other_collect.len() || self_collect != other_collect
    }
}

pub trait DatabaseStore {
    // Get
    fn get_root_path(&self) -> &PathBuf;

    fn get_meta_directory_path(&self) -> PathBuf {
        let mut path = self.get_root_path().clone();
        path.push(DATABASE_META_DIRECTORY_NAME);
        path
    }

    fn get_database_path(&self) -> PathBuf {
        let mut path = self.get_meta_directory_path().clone();
        path.push(DATABASE_DATABASE_NAME);
        path
    }

    fn get_meta_path(&self) -> PathBuf {
        let mut path = self.get_meta_directory_path().clone();
        path.push(DATABASE_META_NAME);
        path
    }

    // Exists
    fn root_exists(&self) -> bool {
        std::fs::read_dir(self.get_root_path()).is_ok()
    }

    fn meta_directory_exists(&self) -> bool {
        std::fs::read_dir(self.get_meta_directory_path()).is_ok()
    }

    fn database_exists(&self) -> bool {
        std::fs::read(self.get_database_path()).is_ok()
    }

    fn meta_exists(&self) -> bool {
        std::fs::read(self.get_meta_path()).is_ok()
    }

    //
    fn initialize_or_create_stores(&self) -> Result<(), DatabaseError> {
        create_dir(self.get_meta_directory_path());

        Ok(())
    }

    fn store_metadata(&self);
    fn retreive_metadata(&mut self);
}

impl DatabaseStore for Database {
    fn get_root_path(&self) -> &PathBuf {
        &self.path
    }

    fn store_metadata(&self) {
        let to_store =
            ron::ser::to_string_pretty(&self.meta, ron::ser::PrettyConfig::default()).unwrap();
        std::fs::write(self.get_meta_path(), to_store).unwrap()
    }

    fn retreive_metadata(&mut self) {
        self.meta = ron::from_str(&std::fs::read_to_string(self.get_meta_path()).unwrap()).unwrap()
    }
}
