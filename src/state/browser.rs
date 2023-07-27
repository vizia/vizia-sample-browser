use std::path::{Path, PathBuf};

use vizia::prelude::*;

#[derive(Debug, Lens, Clone, Data)]
pub struct BrowserState {
    pub libraries: Vec<Directory>,
    // pub root_file: File,
    pub selected: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrowserEvent {
    ViewAll,
    SetSelected(PathBuf),
    #[allow(dead_code)]
    SelectNext,
    #[allow(dead_code)]
    SelectPrev,
    ToggleOpen(PathBuf),
    ToggleShowSearch,
}

#[derive(Debug, Clone, Data, Lens)]
pub struct Directory {
    pub name: String,
    pub path: Option<PathBuf>,
    pub children: Vec<Directory>,
    pub is_open: bool,
    pub num_files: usize,
}

impl Default for Directory {
    fn default() -> Self {
        Self { name: String::new(), path: None, children: Vec::new(), is_open: true, num_files: 0 }
    }
}

impl Default for BrowserState {
    fn default() -> Self {
        Self {
            libraries: vec![Directory {
                name: String::from("root"),
                path: Some(PathBuf::from("test_files/Drum Sounds")),
                children: vec![],
                is_open: true,
                num_files: 0,
            }],
            selected: Some(PathBuf::from("test_files/Drum Sounds")),
        }
    }
}

impl Model for BrowserState {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|browser_event, _| match browser_event {
            // Temp: Load the assets directory for the treeview
            BrowserEvent::ViewAll => {
                if let Some(root) = visit_dirs(Path::new("test_files"), &mut 0) {
                    self.libraries[0] = root;
                }
            }

            BrowserEvent::ToggleOpen(path) => {
                toggle_open(&mut self.libraries[0], path);
            }

            // Set the selected directory item by path
            BrowserEvent::SetSelected(path) => {
                self.selected = Some(path.clone());
            }

            // Move selection the next directory item
            BrowserEvent::SelectNext => {
                let next = recursive_next(&self.libraries[0], None, self.selected.clone());
                if let RetItem::Found(path) = next {
                    self.selected = path;
                }
            }

            // Move selection the previous directory item
            BrowserEvent::SelectPrev => {
                let next = recursive_prev(&self.libraries[0], None, self.selected.clone());
                if let RetItem::Found(path) = next {
                    self.selected = path;
                }
            }

            _ => {}
        });
    }
}

#[derive(Debug, Clone)]
enum RetItem<'a> {
    Found(Option<PathBuf>),
    NotFound(Option<&'a Directory>),
}

fn toggle_open(root: &mut Directory, path: &PathBuf) {
    if root.path == Some(path.clone()) {
        root.is_open ^= true;
    } else {
        for child in root.children.iter_mut() {
            toggle_open(child, path);
        }
    }
}

// Returns the next directory item after `dir` by recursing down the hierarchy
fn recursive_next<'a>(
    root: &'a Directory,
    mut prev: Option<&'a Directory>,
    dir: Option<PathBuf>,
) -> RetItem<'a> {
    if let Some(prev) = prev {
        if prev.path == dir {
            return RetItem::Found(root.path.clone());
        }
    }

    prev = Some(root);
    if root.is_open {
        for child in root.children.iter() {
            let next = recursive_next(child, prev, dir.clone());
            match next {
                RetItem::Found(_) => return next,
                RetItem::NotFound(file) => prev = file,
            }
        }
    }

    RetItem::NotFound(prev)
}

// Returns the previous directory item before `dir` by recursing down the hierarchy
fn recursive_prev<'a>(
    root: &'a Directory,
    mut prev: Option<&'a Directory>,
    dir: Option<PathBuf>,
) -> RetItem<'a> {
    if root.path == dir {
        if let Some(prev) = prev {
            return RetItem::Found(prev.path.clone());
        }
    }

    prev = Some(root);
    if root.is_open {
        for child in root.children.iter() {
            let next = recursive_prev(child, prev, dir.clone());
            match next {
                RetItem::Found(_) => return next,
                RetItem::NotFound(file) => prev = file,
            }
        }
    }

    RetItem::NotFound(prev)
}

// Recursively build directory tree from root path
fn visit_dirs(dir: &Path, num_files: &mut usize) -> Option<Directory> {
    let name = dir.file_name()?.to_str()?.to_string();
    let mut children = Vec::new();

    let mut file_count = 0;

    if dir.is_dir() {
        for entry in std::fs::read_dir(dir).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                children.push(visit_dirs(&path, &mut file_count)?);
            } else {
                // TODO: Check for audio files
                file_count += 1;
            }
        }
    }

    *num_files += file_count;

    // Sort by alphabetical (should this be a setting?)
    children.sort_by(|a, b| a.name.cmp(&b.name));

    Some(Directory {
        name,
        path: Some(PathBuf::from(dir)),
        children,
        is_open: true,
        num_files: file_count,
    })
}
