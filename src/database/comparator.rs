use super::prelude::*;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::{collections::VecDeque, fmt::Debug, fs::read_dir, path::PathBuf};
use vizia::prelude::*;

pub static IGNORE_DIRS: [&str; 1] = [".vsb-meta"];

pub type DirectoryEntryID = usize;
#[derive(Clone, Debug, Serialize, Deserialize, Lens, Eq)]
pub struct DirectoryEntry {
    pub name: String,
    pub path: PathBuf,
    pub id: DirectoryEntryID,
    pub parent: Option<DirectoryEntryID>,
    pub parent_path: Option<PathBuf>,
}

impl DirectoryEntry {
    pub fn from_path(
        path: &PathBuf,
        id: DirectoryEntryID,
        parent: Option<DirectoryEntryID>,
        parent_path: Option<PathBuf>,
    ) -> Self {
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let path = path.clone();

        Self { name, path, id, parent, parent_path }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change<T> {
    Add(T),
    Remove(T),
    None,
}
pub fn compare_vec<T>(v1: &Vec<T>, v2: &Vec<T>) -> Vec<Change<T>>
where
    T: Clone + Ord + Debug,
{
    let mut v1_i = 0;
    let mut v2_i = 0;

    let mut changes = Vec::new();

    loop {
        if v1_i == v1.len() && v2_i == v2.len() {
            break;
        } else if v1.get(v1_i).is_some() && v2.get(v2_i).is_some() {
            let v1_val = v1.get(v1_i).unwrap();
            let v2_val = v2.get(v2_i).unwrap();

            match v1_val.cmp(v2_val) {
                std::cmp::Ordering::Less => {
                    changes.push(Change::Remove(v1_val.clone()));
                    println!("Remove {v1_val:?}");

                    // Advance v2
                    v1_i += 1;
                }
                std::cmp::Ordering::Greater => {
                    changes.push(Change::Add(v2_val.clone()));
                    println!("Add {v2_val:?}");
                    // Advance v1
                    v2_i += 1;
                }
                std::cmp::Ordering::Equal => {
                    v1_i += 1;
                    v2_i += 1;
                }
            }
        } else if v1_i == v1.len() {
            for dir in v2[v2_i..].iter() {
                changes.push(Change::Add(dir.clone()));
                println!("Add {dir:?}");
            }
            break;
        } else if v2_i == v2.len() {
            for dir in v1[v1_i..].iter() {
                changes.push(Change::Remove(dir.clone()));
                println!("Remove {dir:?}");
            }
            break;
        }
    }

    changes
}

pub fn build_dir_trees_from_directory(dir: &PathBuf) -> Vec<DirectoryEntry> {
    let mut dirs = Vec::new();

    let mut dir_stack: VecDeque<(PathBuf, Option<usize>, Option<usize>, Option<PathBuf>)> =
        VecDeque::new();

    let mut next_id = 0;

    dir_stack.push_back((dir.clone(), None, None, None));

    while let Some((next_dir, parent_id, parent, parent_path)) = dir_stack.pop_front() {
        let dir = DirectoryEntry::from_path(&next_dir, next_id, parent, parent_path);
        next_id += 1;

        if next_dir.is_dir() {
            read_dir(next_dir)
                .unwrap()
                .filter_map(|v| v.ok())
                .filter(|v| !IGNORE_DIRS.contains(&v.path().file_name().unwrap().to_str().unwrap()))
                .sorted_by(|a, b| a.file_name().cmp(&b.file_name()))
                .for_each(|v| {
                    dir_stack.push_back((
                        v.path(),
                        Some(dir.id),
                        dir.parent,
                        Some(dir.path.clone()),
                    ))
                });
        }

        dirs.push(dir);
    }

    dirs
}

pub trait DatabaseComparator {
    fn check_changes(&self) -> bool;
    fn update_changes(&self);
}

impl DatabaseComparator for Database {
    fn check_changes(&self) -> bool {
        true
    }

    fn update_changes(&self) {}
}

impl Hash for DirectoryEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.path.hash(state);
        self.parent_path.hash(state);
    }
}

impl PartialEq for DirectoryEntry {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}

impl PartialOrd for DirectoryEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.path.partial_cmp(&other.path)
    }
}

impl Ord for DirectoryEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.cmp(&other.path)
    }
}
