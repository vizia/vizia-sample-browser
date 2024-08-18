use std::path::Path;

use crate::database::{build_dir_trees_from_directory, compare_vec, Change, DirectoryEntry};

use super::TEST_DIRECTORY;

#[test]
fn compare_test() {
    let vec1 = vec![2, 5, 7, 9];
    let vec2 = vec![1, 2, 5, 6, 7, 10];

    let changes = compare_vec(&vec1, &vec2);

    assert_eq!(changes, vec![Change::Add(1), Change::Add(6), Change::Remove(9), Change::Add(10)])
}

#[test]
fn build_tree_from_directory() {
    let path = Path::new(TEST_DIRECTORY).to_path_buf();

    let tree = build_dir_trees_from_directory(&path);

    for dir_entry in tree.iter() {
        println!("{:?}", dir_entry);
    }
}

#[test]
fn compare_directory_entry() {
    let v1 = vec![
        DirectoryEntry {
            name: "One".to_string(),
            id: 0,
            path: Path::new("").to_path_buf(),
            parent: None,
            parent_path: None,
        },
        DirectoryEntry {
            name: "Sub One".to_string(),
            id: 1,
            path: Path::new("/1").to_path_buf(),
            parent: Some(0),
            parent_path: Some(Path::new("").to_path_buf()),
        },
        DirectoryEntry {
            name: "Sub Two".to_string(),
            id: 2,
            path: Path::new("/2").to_path_buf(),
            parent: Some(0),
            parent_path: Some(Path::new("").to_path_buf()),
        },
        DirectoryEntry {
            name: "Sub Three".to_string(),
            id: 2,
            path: Path::new("/3").to_path_buf(),
            parent: Some(0),
            parent_path: Some(Path::new("").to_path_buf()),
        },
    ];

    let v2 = vec![
        DirectoryEntry {
            name: "One".to_string(),
            id: 0,
            path: Path::new("").to_path_buf(),
            parent: None,
            parent_path: None,
        },
        DirectoryEntry {
            name: "Sub One".to_string(),
            id: 1,
            path: Path::new("/1").to_path_buf(),
            parent: Some(0),
            parent_path: Some(Path::new("").to_path_buf()),
        },
        DirectoryEntry {
            name: "Sub Two".to_string(),
            id: 2,
            path: Path::new("/1/2").to_path_buf(),
            parent: Some(1),
            parent_path: Some(Path::new("").to_path_buf()),
        },
    ];

    let changes = compare_vec(&v1, &v2);
}
