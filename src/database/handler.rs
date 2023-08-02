use super::*;
use rusqlite::Connection;
use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    error::Error,
    fs::{read_dir, DirEntry, File},
    path::{Path, PathBuf},
    rc::Rc,
    sync::atomic::AtomicUsize,
};

pub const DATABASE_FILE_NAME: &str = ".database.vsb";
pub const AUDIO_FILE_EXTENSIONS: [&'static str; 1] = ["wav"];

#[derive(Debug)]
pub struct Database {
    path: PathBuf,
    conn: Option<Connection>,
}

#[derive(Debug)]
pub enum DatabaseHandleError {
    ConnectionClosed,
    PathNotDirecotry,
    PathWithoutMetadata,
    Other(Box<dyn Any>),
}

impl<E> From<E> for DatabaseHandleError
where
    E: Error + Any + Sized,
{
    fn from(value: E) -> Self {
        Self::Other(Box::new(value))
    }
}

impl Database {
    pub fn from_directory(path: PathBuf) -> Result<Self, DatabaseHandleError> {
        // Check file is directory
        let file = match File::open(&path) {
            Ok(f) => f,
            Err(e) => return Err(DatabaseHandleError::Other(Box::new(e))),
        };

        if file.metadata().is_err() {
            return Err(DatabaseHandleError::PathWithoutMetadata);
        }

        if !file.metadata().unwrap().file_type().is_dir() {
            return Err(DatabaseHandleError::PathNotDirecotry);
        }

        // Open connection
        let mut s = Self { path: path.clone(), conn: None };

        let database_exists = File::open(s.get_database_path()).is_ok();

        s.open_connection()?;

        if !database_exists {
            let audio_file_count = AtomicUsize::new(0);
            let collection_count = AtomicUsize::new(0);

            let collections: Rc<RefCell<HashMap<PathBuf, usize>>> =
                Rc::new(RefCell::new(HashMap::new()));

            let connection = Rc::new(RefCell::new(s.connection().unwrap()));

            // Recursively check each directory under the root
            recursive_directory_closure(&path, None, |path, parent_path, files| {
                let mut colls = collections.borrow_mut();

                let name = path.file_name().unwrap().to_str().unwrap().to_string();
                let id = collection_count.load(std::sync::atomic::Ordering::Relaxed);
                collection_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let parent_id = if parent_path.is_none() {
                    None
                } else {
                    Some(*colls.get(parent_path.unwrap()).unwrap())
                };

                // Insert collection
                let collection = Collection { id, parent_collection: parent_id, name };

                connection
                    .borrow_mut()
                    .execute(
                        "INSERT INTO collections (id, parent_collection, name) VALUES (?1, ?2, ?3)",
                        (collection.id, collection.parent_collection, collection.name),
                    )
                    .unwrap();

                colls.insert(path.clone(), id);
                drop(colls);

                // Insert each non-directory child
                for child_file in files {
                    let p = child_file.path();
                    let extension = p.extension().map(|v| v.to_str().unwrap()).unwrap_or("");

                    if !AUDIO_FILE_EXTENSIONS.contains(&extension) {
                        break;
                    }

                    let file_id = audio_file_count.load(std::sync::atomic::Ordering::Relaxed);
                    audio_file_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let name = child_file.file_name().to_str().unwrap().to_string();

                    let audio_file = AudioFile {
                        id: file_id,
                        name,
                        collection: id,
                        duration: 0.0,
                        sample_rate: 0.0,
                        bit_depth: 0.0,
                        bpm: None,
                        key: None,
                        size: 0.0,
                    };

                    connection
                        .borrow_mut()
                        .execute(
                            "INSERT INTO audio_files (id, name, collection, duration, sample_rate, bit_depth, bpm, key, size) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                            (
                                audio_file.id,
                                audio_file.name,
                                audio_file.collection,
                                audio_file.duration,
                                audio_file.sample_rate,
                                audio_file.bit_depth,
                                audio_file.bpm,
                                audio_file.key,
                                audio_file.size,
                            ),
                        )
                        .unwrap();
                }
            })?;
        }

        Ok(s)
    }

    pub fn from_connection(path: &str, connection: Option<Connection>) -> Self {
        Database { path: Path::new(path).to_path_buf(), conn: connection }
    }

    pub fn open_connection(&mut self) -> rusqlite::Result<()> {
        let database_exists = File::open(self.get_database_path()).is_ok();

        if self.conn.is_none() {
            self.conn = Some(Connection::open(self.get_database_path())?);
        }

        if !database_exists {
            self.connection().unwrap().execute_batch(include_str!("./schema.sql")).unwrap();
        }

        Ok(())
    }

    pub fn close_connection(&mut self) {
        if let Some(c) = self.connection() {
            drop(c);
            self.conn = None;
        }
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }

    pub fn get_database_path(&self) -> PathBuf {
        let mut path = self.get_path().clone();
        path.push(DATABASE_FILE_NAME);
        path.to_path_buf()
    }

    pub fn connection(&self) -> Option<&Connection> {
        self.conn.as_ref()
    }

    fn insert_collection(&mut self, collection: Collection) -> Result<(), DatabaseHandleError> {
        self.check_connection()?;

        self.connection()
            .unwrap()
            .execute(
                "INSERT INTO collections (id, parent_collection, name) VALUES (?1, ?2, ?3)",
                (collection.id, collection.parent_collection, collection.name),
            )
            .unwrap();

        Ok(())
    }

    fn insert_audio_file(&mut self, audio_file: AudioFile) -> Result<(), DatabaseHandleError> {
        self.check_connection()?;

        self.connection()
            .unwrap()
            .execute(
                "INSERT INTO audio_files (id, name, collection, duration, sample_rate, bit_depth, bpm, key, size) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                (
                    audio_file.id,
                    audio_file.name,
                    audio_file.collection,
                    audio_file.duration,
                    audio_file.sample_rate,
                    audio_file.bit_depth,
                    audio_file.bpm,
                    audio_file.key,
                    audio_file.key,
                    audio_file.size,
                ),
            )
            .unwrap();

        Ok(())
    }

    fn check_connection(&self) -> Result<(), DatabaseHandleError> {
        if self.conn.is_none() {
            return Err(DatabaseHandleError::ConnectionClosed);
        }

        Ok(())
    }

    pub fn get_root_collection(&self) -> Result<Collection, DatabaseHandleError> {
        self.check_connection()?;

        let mut query = self.connection().unwrap().prepare(
            "SELECT id, parent_collection, name FROM collections WHERE parent_collection IS NULL",
        )?;

        let col: Collection = query.query_row([], |row| {
            Ok(Collection { id: row.get(0)?, parent_collection: None, name: row.get(2)? })
        })?;

        Ok(col)
    }

    pub fn get_all_collections(&self) -> Result<Vec<Collection>, DatabaseHandleError> {
        self.check_connection()?;

        let mut query = self
            .connection()
            .unwrap()
            .prepare("SELECT id, parent_collection, name FROM collections")?;
        let collections = query.query_map([], |row| {
            Ok(Collection { id: row.get(0)?, parent_collection: row.get(1)?, name: row.get(2)? })
        })?;

        Ok(collections.map(|v| v.unwrap()).collect())
    }

    pub fn get_child_collections(
        &self,
        parent: CollectionID,
    ) -> Result<Vec<Collection>, DatabaseHandleError> {
        self.check_connection()?;

        let mut query = self.connection().unwrap().prepare(
            "SELECT id, parent_collection, name FROM collections WHERE parent_collection = (?1)",
        )?;
        let collections = query.query_map([parent], |row| {
            Ok(Collection { id: row.get(0)?, name: row.get(2)?, parent_collection: row.get(1)? })
        })?;
        Ok(collections.map(|v| v.unwrap()).collect())
    }

    pub fn get_all_audio_files(&self) -> Result<Vec<AudioFile>, DatabaseHandleError> {
        self.check_connection()?;

        let mut query = self
            .connection()
            .unwrap()
            .prepare("SELECT id, name, collection, duration, sample_rate, bit_depth, bpm, key, size FROM audio_files")?;

        let audio_files = query.query_map([], |row| {
            Ok(AudioFile {
                id: row.get(0)?,
                name: row.get(1)?,
                collection: row.get(2)?,
                duration: row.get(3)?,
                sample_rate: row.get(4)?,
                bit_depth: row.get(5)?,
                bpm: row.get(6)?,
                key: row.get(7)?,
                size: row.get(8)?,
            })
        })?;

        Ok(audio_files.map(|v| v.unwrap()).collect())
    }

    pub fn get_child_audio_files(
        &self,
        parent: CollectionID,
    ) -> Result<Vec<AudioFile>, DatabaseHandleError> {
        self.check_connection()?;

        let mut query = self.connection().unwrap().prepare(
            "SELECT id, name, collection, duration, sample_rate, bit_depth, bpm, key, size FROM audio_files WHERE collection = (?1)",
        )?;
        let collections = query.query_map([parent], |row| {
            Ok(AudioFile {
                id: row.get(0)?,
                name: row.get(1)?,
                collection: row.get(2)?,
                duration: row.get(3)?,
                sample_rate: row.get(4)?,
                bit_depth: row.get(5)?,
                bpm: row.get(6)?,
                key: row.get(7)?,
                size: row.get(8)?,
            })
        })?;
        Ok(collections.map(|v| v.unwrap()).collect())
    }
}

fn recursive_directory_closure<F>(
    path: &PathBuf,
    parent_path: Option<&PathBuf>,
    mut closure: F,
) -> Result<(), std::io::Error>
where
    F: FnMut(&PathBuf, Option<&PathBuf>, &Vec<DirEntry>) + Clone,
{
    let read_dir = read_dir(&path)?;

    let mut child_directories = Vec::new();
    let mut child_files = Vec::new();

    read_dir.filter(|v| v.is_ok()).map(|v| v.unwrap()).for_each(|v| {
        match v.metadata().unwrap().is_dir() {
            true => child_directories.push(v),
            false => child_files.push(v),
        }
    });

    (closure)(&path, parent_path, &child_files);

    for directory in child_directories {
        recursive_directory_closure(&directory.path(), Some(&path), closure.clone())?;
    }

    Ok(())
}
