use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use vizia::prelude::*;

#[derive(Debug, Lens, Clone, Data)]
pub struct BrowserState {
    pub libraries: Vec<Directory>,
    pub selected: HashSet<PathBuf>,
    pub focused: Option<PathBuf>,
    pub search_text: String,
    pub filter_search: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrowserEvent {
    ViewAll,
    Search(String),
    Select(PathBuf),
    Deselect,
    AddSelection(PathBuf),
    SetFocused(Option<PathBuf>),
    #[allow(dead_code)]
    FocusNext,
    #[allow(dead_code)]
    FocusPrev,
    ToggleDirectory(PathBuf),
    ExpandDirectory,
    CollapseDirectory,
    ToggleShowSearch,
    ToggleSearchFilter,
    ShowTree,
    ShowList,
}

#[derive(Debug, Clone, Data, Lens)]
pub struct Directory {
    pub name: String,
    pub path: PathBuf,
    pub children: Vec<Directory>,
    pub is_open: bool,
    pub num_files: usize,
    pub match_indices: Vec<usize>,
    pub shown: bool,
}

impl Default for BrowserState {
    fn default() -> Self {
        Self {
            libraries: vec![Directory {
                name: String::from("root"),
                path: PathBuf::from("the-libre-sample-pack"),
                children: vec![],
                is_open: false,
                num_files: 0,
                match_indices: Vec::default(),
                shown: true,
            }],
            selected: HashSet::new(),
            focused: None,
            search_text: String::new(),
            filter_search: false,
        }
    }
}

impl Model for BrowserState {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|browser_event, _| match browser_event {
            // Temp: Load the assets directory for the treeview
            BrowserEvent::ViewAll => {
                if let Some(root) = visit_dirs(Path::new("the-libre-sample-pack"), &mut 0) {
                    self.libraries[0] = root;
                }
            }

            BrowserEvent::Search(search_text) => {
                self.focused = None;
                self.search_text = search_text.clone();
                search(&mut self.libraries[0], &self.search_text, self.filter_search);
            }

            BrowserEvent::ToggleSearchFilter => {
                self.filter_search ^= true;
                if !self.search_text.is_empty() {
                    search(&mut self.libraries[0], &self.search_text, self.filter_search);
                }
            }

            BrowserEvent::ToggleDirectory(path) => {
                toggle_open(&mut self.libraries[0], path);
            }

            BrowserEvent::ExpandDirectory => {
                if let Some(focused) = &self.focused {
                    set_expand_directory(&mut self.libraries[0], focused, true);
                }
            }

            BrowserEvent::CollapseDirectory => {
                if let Some(focused) = &self.focused {
                    if is_collapsed(&mut self.libraries[0], focused) {
                        self.focused = focused.parent().map(|p| p.to_owned());
                    } else {
                        set_expand_directory(&mut self.libraries[0], focused, false);
                    }
                }
            }

            BrowserEvent::Select(path) => {
                self.selected.clear();
                self.selected.insert(path.clone());
            }

            BrowserEvent::AddSelection(path) => {
                self.selected.insert(path.clone());
            }

            BrowserEvent::Deselect => {
                self.selected.clear();
            }

            // Set the selected directory item by path
            BrowserEvent::SetFocused(path) => {
                self.focused = path.clone();
            }

            // Move focus the next directory item
            BrowserEvent::FocusNext => {
                if let Some(focused) = &self.focused {
                    let next = recursive_next(&self.libraries[0], None, focused);
                    if let RetItem::Found(path) = next {
                        self.focused = Some(path);
                        cx.focus_next();
                    }
                }
            }

            // Move selection the previous directory item
            BrowserEvent::FocusPrev => {
                if let Some(focused) = &self.focused {
                    let next = recursive_prev(&self.libraries[0], None, focused);
                    if let RetItem::Found(path) = next {
                        self.focused = Some(path);
                        cx.focus_prev();
                    }
                }
            }

            _ => {}
        });
    }
}

#[derive(Debug, Clone)]
enum RetItem<'a> {
    Found(PathBuf),
    NotFound(Option<&'a Directory>),
}

fn toggle_open(root: &mut Directory, path: &PathBuf) {
    if root.path == *path {
        root.is_open ^= true;
    } else {
        for child in root.children.iter_mut() {
            toggle_open(child, path);
        }
    }
}

fn set_expand_directory(root: &mut Directory, path: &PathBuf, expand: bool) {
    if root.path == *path {
        root.is_open = expand;
    } else {
        for child in root.children.iter_mut() {
            set_expand_directory(child, path, expand);
        }
    }
}

// Returns the next directory item after `dir` by recursing down the hierarchy
fn recursive_next<'a>(
    root: &'a Directory,
    mut prev: Option<&'a Directory>,
    dir: &PathBuf,
) -> RetItem<'a> {
    if let Some(prev) = prev {
        if prev.path == *dir {
            return RetItem::Found(root.path.clone());
        }
    }

    prev = Some(root);
    if root.is_open {
        for child in root.children.iter() {
            let next = recursive_next(child, prev, dir);
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
    dir: &PathBuf,
) -> RetItem<'a> {
    if root.path == *dir {
        if let Some(prev) = prev {
            return RetItem::Found(prev.path.clone());
        }
    }

    prev = Some(root);
    if root.is_open {
        for child in root.children.iter() {
            let next = recursive_prev(child, prev, dir);
            match next {
                RetItem::Found(_) => return next,
                RetItem::NotFound(file) => prev = file,
            }
        }
    }

    RetItem::NotFound(prev)
}

fn search<'a>(root: &'a mut Directory, search_text: &String, filter: bool) -> bool {
    let mut parent_is_shown = !filter;
    let matcher = SkimMatcherV2::default();
    if let Some((_, indices)) = matcher.fuzzy_indices(&root.name, search_text) {
        root.match_indices = indices;
        parent_is_shown = true;
    } else {
        root.match_indices.clear();
    }

    let mut child_is_shown = false;
    for child in root.children.iter_mut() {
        child_is_shown |= search(child, search_text, filter);
    }

    root.shown = parent_is_shown | child_is_shown;

    return root.shown;
}

fn is_collapsed<'a>(root: &'a Directory, dir: &PathBuf) -> bool {
    if root.path == *dir {
        if !root.is_open {
            return true;
        }
    } else {
        for child in root.children.iter() {
            if is_collapsed(child, dir) {
                return true;
            }
        }
    }

    false
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

    let has_children = !children.is_empty();

    Some(Directory {
        name,
        path: PathBuf::from(dir),
        children,
        is_open: has_children,
        num_files: file_count,
        match_indices: Vec::default(),
        shown: true,
    })
}
