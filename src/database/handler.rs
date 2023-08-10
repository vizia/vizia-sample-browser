use crate::state::browser::Directory;

use super::*;
use itertools::Itertools;
use rusqlite::Connection;
use std::{
    any::Any,
    cell::RefCell,
    collections::{BTreeSet, HashMap, VecDeque},
    error::Error,
    fs::{create_dir, read_dir, DirEntry, File},
    path::{Path, PathBuf},
    rc::Rc,
    sync::atomic::AtomicUsize,
};

pub const DATABASE_FILE_NAME: &str = ".database.vsb";
pub const AUDIO_FILE_EXTENSIONS: [&'static str; 1] = ["wav"];

#[derive(Debug)]
pub struct Database {
    pub(super) path: PathBuf,
    pub(super) conn: Option<Connection>,
    pub(super) meta: DatabaseMetadata,
}

impl Database {
    pub fn from_directory(path: PathBuf) -> Result<Self, DatabaseError> {
        // Check file is directory
        if !directory_exists(&path) {
            return Err(DatabaseError::PathNotDirecotry);
        }

        // Open connection
        let mut s = Self { path, conn: None, meta: DatabaseMetadata::new() };
        s.initialize_or_create_stores()?;

        // let database_exists = File::open(s.get_database_path()).is_ok();

        s.open_connection()?;

        // if !database_exists {
        //     s.initialize_empty_database();
        // } else {
        //     s.update_database();
        // }

        Ok(s)
    }

    fn clear_database(&mut self) {
        self.get_connection().unwrap().execute_batch(include_str!("sqls/clear.sql")).unwrap();
    }

    fn initialize_empty_database(&mut self) {
        let audio_file_count = AtomicUsize::new(0);
        let collection_count = AtomicUsize::new(0);

        let collections: Rc<RefCell<HashMap<PathBuf, usize>>> =
            Rc::new(RefCell::new(HashMap::new()));

        let connection = Rc::new(RefCell::new(self.get_connection().unwrap()));

        let path = self.path.clone();

        // Recursively check each directory under the root
        recursive_directory_closure(self, &path, None, |db, path, parent_path, files| {
            let mut colls = collections.borrow_mut();

            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            let id = collection_count.load(std::sync::atomic::Ordering::Relaxed);
            collection_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let parent_id = match parent_path.is_none() {
                true => None,
                false => Some(*colls.get(parent_path.unwrap()).unwrap()),
            };

            // Insert collection
            let collection = Collection::new(id, parent_id, name);

            db.insert_collection(collection);

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

                let audio_file = AudioFile::new(file_id, name, id, 0.0, 0.0, 0.0, None, None, 0.0);

                db.insert_audio_file(audio_file);
            }
        });
    }

    pub fn from_connection(path: &str, connection: Option<Connection>) -> Self {
        Database {
            path: Path::new(path).to_path_buf(),
            conn: connection,
            meta: DatabaseMetadata::new(),
        }
    }

    pub fn close_database(&mut self) {
        self.store_metadata();
        self.close_connection().unwrap();
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        let meta_dir = self.get_meta_directory_path();
        std::fs::remove_dir_all(meta_dir);
    }
}

fn recursive_directory_closure<F>(
    db: &mut Database,
    path: &PathBuf,
    parent_path: Option<&PathBuf>,
    mut closure: F,
) -> Result<(), std::io::Error>
where
    F: FnMut(&mut Database, &PathBuf, Option<&PathBuf>, &Vec<DirEntry>) + Clone,
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

    (closure)(db, &path, parent_path, &child_files);

    for directory in child_directories {
        recursive_directory_closure(db, &directory.path(), Some(&path), closure.clone())?;
    }

    Ok(())
}

#[derive(Clone, Debug)]
struct RecursiveDir {
    id: CollectionID,
    parent_id: Option<CollectionID>,
    name: String,
    children: Vec<RecursiveDir>,
}

fn query_to_recursive(db: &Database) -> RecursiveDir {
    let collections = db.get_all_collections().unwrap();

    let mut hm: HashMap<CollectionID, RecursiveDir> = HashMap::new();

    for coll in collections {
        hm.insert(
            coll.id(),
            RecursiveDir {
                id: coll.id(),
                parent_id: coll.parent_collection(),
                name: coll.name().to_string(),
                children: Vec::new(),
            },
        );
    }

    fn children_of_collection(
        map: &HashMap<CollectionID, RecursiveDir>,
        coll: CollectionID,
    ) -> VecDeque<CollectionID> {
        map.values().filter(|v| v.parent_id == Some(coll)).map(|v| v.id).collect()
    }

    let mut root_dir = hm.values().find(|v| v.parent_id.is_none()).unwrap().clone();

    let mut collection_stack: VecDeque<CollectionID> = children_of_collection(&hm, root_dir.id);

    while let Some(coll) = collection_stack.pop_front() {
        let mut children = children_of_collection(&hm, coll);
        collection_stack.append(&mut children);

        let coll_data = hm.get(&coll).unwrap().clone();
        root_dir.children.push(coll_data);
    }

    root_dir
}

#[test]
fn query_to_recursive_test() {
    let mut handle = Database::from_connection("", Some(Connection::open_in_memory().unwrap()));
    handle.get_connection().unwrap().execute_batch(include_str!("sqls/schema.sql")).unwrap();
    handle.get_connection().unwrap().execute_batch(include_str!("sqls/test.sql")).unwrap();

    let root = query_to_recursive(&handle);

    print_directory(&root);
}

fn print_directory(dir: &RecursiveDir) {
    println!("{:?}", dir.name);
    for child in dir.children.iter() {
        print_directory(child)
    }
}
