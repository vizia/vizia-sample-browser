use crate::state::browser::Directory;

use super::*;
use rusqlite::Connection;
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

pub const DATABASE_FILE_NAME: &str = ".database.vsb";
pub const AUDIO_FILE_EXTENSIONS: [&'static str; 1] = ["wav"];

#[derive(Debug, Lens)]
pub struct Database {
    pub path: PathBuf,
    pub conn: Option<Connection>,
    pub meta: DatabaseMetadata,
}

impl Database {
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

        match directory_created {
            true => s.update_database(),
            false => s.initialize_empty_database(),
        }

        s.store_metadata();

        Ok(s)
    }

    fn update_database(&mut self) {
        let mut hasher = DefaultHasher::new();
        let tree = build_dir_trees_from_directory(&Path::new("test_files/").to_path_buf());

        for entry in tree.iter() {
            entry.hash(&mut hasher);
        }

        let hash_generated = hasher.finish();

        if hash_generated != self.meta.hash_id {
            // Update database & entries
            self.walk_to_new_tree(&tree);

            self.meta.entries = tree;
            self.meta.hash_id = hash_generated;
        }
    }

    pub fn walk_to_new_tree(&mut self, new_tree: &Vec<DirectoryEntry>) {
        let root = self.meta.entries.iter().find(|v| v.parent.is_none()).unwrap();

        fn children_of_directory(
            tree: &Vec<DirectoryEntry>,
            dir: DirectoryEntryID,
        ) -> Vec<DirectoryEntry> {
            tree.iter().filter(|v| v.parent == Some(dir)).map(|v| v.clone()).collect()
        }

        let mut dir_stack = VecDeque::new();
        let mut to_add = Vec::new();
        let mut to_remove = Vec::new();

        dir_stack.push_back(root.clone());

        // First walk, retreive changes and store them
        while let Some(dir) = dir_stack.pop_front() {
            let mut children = children_of_directory(&self.meta.entries, dir.id);
            let children_new = children_of_directory(new_tree, dir.id);

            let changes = compare_vec(&children, &children_new);

            for change in changes {
                match change {
                    Change::Add(c) => {
                        // Add directory recursively here
                        to_add.push((dir.path.clone(), c.clone()));
                        let pos = children.iter().position(|v| *v == c).unwrap();
                        children.remove(pos);
                    }
                    Change::Remove(c) => {
                        // Remove directory recursively here
                        to_remove.push((dir.path.clone(), c.clone()));
                        let pos = children.iter().position(|v| *v == c).unwrap();
                        children.remove(pos);
                    }
                    Change::None => {}
                }
            }

            children.iter().for_each(|v| dir_stack.push_back(v.clone()))
        }

        // Remove directories that need to be removed

        for (path, dir) in to_add {
            self.insert_recursive_directory(&dir.path, &path);
        }

        for (path, dir) in to_remove {
            self.remove_recursive_dir(&dir.path);
        }
    }

    fn insert_recursive_directory(&mut self, parent_path: &PathBuf, path: &PathBuf) {
        let read_dir = read_dir(path).unwrap();

        let mut child_directories = Vec::new();
        let mut child_files = Vec::new();

        read_dir.filter(|v| v.is_ok()).map(|v| v.unwrap()).for_each(|v| {
            match v.metadata().unwrap().is_dir() {
                true => child_directories.push(v),
                false => child_files.push(v),
            }
        });

        //

        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        if name == ".vsb-meta" {
            return;
        }

        self.meta.last_collection_id += 1;
        let id = self.meta.last_collection_id;

        let parent_col = self.get_collection_by_path(&parent_path).unwrap();

        let collection = Collection::new(id, Some(parent_col.id()), name, path.clone());

        println!("INSERT COLLECTION {:?}", collection);

        self.insert_collection(collection);

        //

        for file in child_files {
            self.meta.last_audio_file += 1;
            let audio_file = AudioFile::from_path(&file.path(), self.meta.last_audio_file).unwrap();

            println!("INSERT AUDIOFILE {:?}", audio_file);
            self.insert_audio_file(audio_file);
        }

        for directory in child_directories {
            self.insert_recursive_directory(path, &directory.path());
        }
    }

    fn remove_recursive_dir(&mut self, path: &PathBuf) {
        let coll = self.get_collection_by_path(path).unwrap();
        println!("REMOVE COLLECTION {:?}", coll);
        self.remove_collection(coll.id());
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
            if name == ".vsb-meta" {
                return;
            }

            let id = collection_count.load(std::sync::atomic::Ordering::Relaxed);
            collection_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let parent_id = match parent_path.is_none() {
                true => None,
                false => Some(*colls.get(parent_path.unwrap()).unwrap()),
            };

            // Insert collection
            let collection = Collection::new(id, parent_id, name, path.clone());

            db.insert_collection(collection);

            colls.insert(path.clone(), id);
            drop(colls);

            // Insert each non-directory child
            for child_file in files {
                let audio_file = AudioFile::from_path(
                    &child_file.path(),
                    audio_file_count.load(std::sync::atomic::Ordering::Relaxed),
                )
                .unwrap();
                audio_file_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                db.insert_audio_file(audio_file);
            }
        });

        self.meta.last_collection_id = collection_count.load(std::sync::atomic::Ordering::Relaxed);
        self.meta.last_audio_file = audio_file_count.load(std::sync::atomic::Ordering::Relaxed);
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
        // let meta_dir = self.get_meta_directory_path();
        // std::fs::remove_dir_all(meta_dir);
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

impl PartialEq for Database {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self.conn.is_some() == other.conn.is_some()
            && self.meta == other.meta
    }
}
