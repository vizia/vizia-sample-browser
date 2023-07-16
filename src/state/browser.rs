use std::path::{Path, PathBuf};

use vizia::prelude::*;

#[derive(Debug, Lens, Clone, Data)]
pub struct BrowserState {
    pub root_file: File,
    pub selected: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrowserEvent {
    ViewAll,
    SetRootPath(PathBuf),
    SetSelected(PathBuf),
    SelectNext,
    SelectPrev,
    ToggleOpen,
}

#[derive(Debug, Clone, Data, Lens)]
pub struct File {
    pub name: String,
    pub file_path: Option<PathBuf>,
    pub children: Vec<File>,
    pub is_open: bool,
    pub is_dir: bool,
}

impl Default for File {
    fn default() -> Self {
        Self {
            name: String::new(),
            file_path: None,
            children: Vec::new(),
            is_open: true,
            is_dir: true,
        }
    }
}

impl Default for BrowserState {
    fn default() -> Self {
        Self {
            root_file: File {
                name: String::from("root"),
                file_path: Some(PathBuf::from("test_files")),
                children: vec![],
                is_open: true,
                is_dir: true,
            },
            selected: Some(PathBuf::from("test_files")),
        }
    }
}

impl Model for BrowserState {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|browser_event, _| match browser_event {
            // Temp: Load the assets directory for the treeview
            BrowserEvent::ViewAll => {
                if let Some(root) = visit_dirs(Path::new("test_files")) {
                    self.root_file = root;
                }
            }

            BrowserEvent::SetRootPath(path) => {
                if let Some(root) = visit_dirs(path.as_path()) {
                    self.root_file = root;
                }
            }

            BrowserEvent::ToggleOpen => {
                if let Some(path) = &self.selected {
                    toggle_open(&mut self.root_file, path);
                }
            }

            // Set the selected directory item by path
            BrowserEvent::SetSelected(path) => {
                self.selected = Some(path.clone());
            }

            // Move selection the next directory item
            BrowserEvent::SelectNext => {
                let next = recursive_next(&self.root_file, None, self.selected.clone());
                if let RetItem::Found(path) = next {
                    self.selected = path;
                }
            }

            // Move selection the previous directory item
            BrowserEvent::SelectPrev => {
                let next = recursive_prev(&self.root_file, None, self.selected.clone());
                if let RetItem::Found(path) = next {
                    self.selected = path;
                }
            }
        });
    }
}

#[derive(Debug, Clone)]
enum RetItem<'a> {
    Found(Option<PathBuf>),
    NotFound(Option<&'a File>),
}

fn toggle_open(root: &mut File, path: &PathBuf) {
    if root.file_path == Some(path.clone()) {
        root.is_open ^= true;
    } else {
        for child in root.children.iter_mut() {
            toggle_open(child, path);
        }
    }
}

// Returns the next directory item after `dir` by recursing down the hierarchy
fn recursive_next<'a>(
    root: &'a File,
    mut prev: Option<&'a File>,
    dir: Option<PathBuf>,
) -> RetItem<'a> {
    if let Some(prev) = prev {
        if prev.file_path == dir {
            return RetItem::Found(root.file_path.clone());
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
    root: &'a File,
    mut prev: Option<&'a File>,
    dir: Option<PathBuf>,
) -> RetItem<'a> {
    if root.file_path == dir {
        if let Some(prev) = prev {
            return RetItem::Found(prev.file_path.clone());
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
fn visit_dirs(dir: &Path) -> Option<File> {
    let name = dir.file_name()?.to_str()?.to_string();
    let mut children = Vec::new();

    if dir.is_dir() {
        for entry in std::fs::read_dir(dir).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                children.push(visit_dirs(&path)?);
            } else {
                children.push(File {
                    name: entry.path().file_name()?.to_str()?.to_string(),
                    file_path: Some(entry.path()),
                    children: vec![],
                    is_open: true,
                    is_dir: false,
                })
            }
        }
    }

    // Sort by alphabetical
    children.sort_by(|a, b| a.name.cmp(&b.name));
    // Sort by directory vs file
    children.sort_by(|a, b| {
        let a_is_dir: bool = a.children.is_empty();
        let b_is_dir: bool = b.children.is_empty();
        a_is_dir.cmp(&b_is_dir)
    });

    Some(File {
        name,
        file_path: Some(PathBuf::from(dir)),
        children,
        is_open: true,
        is_dir: dir.is_dir(),
    })
}
