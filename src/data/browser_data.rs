//! GUI state used for the browser panel

use super::app_data::{AppData, AppEvent};
use crate::database::prelude::*;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::{Path, PathBuf},
};
use vizia::prelude::*;

/// The data model for the browser panel
#[derive(Debug, Lens, Clone, Default)]
pub struct BrowserData {
    // The libraries to display in the browser
    pub libraries: Vec<Directory>,
    // The selected items in the browser
    pub selected: HashSet<PathBuf>,
    // The focused item in the browser
    pub focused: Option<PathBuf>,
    // The search text in the search box
    pub search_text: String,
    // Whether to filter the search results
    pub filter_search: bool,
    // Whether the search should be case sensitive
    pub search_case_sensitive: bool,
}

impl BrowserData {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrowserEvent {
    // Search for a directory
    Search(String),
    /// Select a directory item by path
    Select(PathBuf, CollectionID),
    // Deselect all selected items
    Deselect,
    // Add a directory item to the selection
    AddSelection(PathBuf),
    // Set the focused directory item by path
    SetFocused(Option<PathBuf>),
    // Move selection to the next directory item
    SelectNext,
    // Move selection to the previous directory item
    SelectPrev,
    // Toggle the visibility of a directory item
    ToggleDirectory(PathBuf),
    // Expand the focused directory item
    ExpandDirectory,
    // Collapse the focused directory item
    CollapseDirectory,
    // Toggle the visibility of the search box
    ToggleShowSearch,
    // Toggle the filtering of the search results
    ToggleSearchFilter,
    // Toggle the case sensitivity of the search
    ToggleSearchCaseSensitivity,
}

#[derive(Debug, Clone, Data, Lens, Default)]
pub struct Directory {
    // The ID of the collection
    pub id: CollectionID,
    // The ID of the parent collection
    pub parent_id: Option<CollectionID>,
    // The name of the collection
    pub name: String,
    // The path of the collection
    pub path: PathBuf,
    // The children of the collection
    pub children: Vec<Directory>,
    // Whether the collection is open
    pub is_open: bool,
    // The number of files in the collection
    pub num_files: usize,
    // The indices of the matched characters in the name
    pub match_indices: Vec<usize>,
    // Whether the collection is shown
    pub shown: bool,
}

impl Model for BrowserData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|browser_event, _| match browser_event {
            BrowserEvent::Search(search_text) => {
                self.focused = None;
                self.search_text = search_text.clone();
                if !self.libraries.is_empty() {
                    search(
                        &mut self.libraries[0],
                        &self.search_text,
                        self.filter_search,
                        !self.search_case_sensitive,
                    );
                }
            }

            BrowserEvent::ToggleSearchFilter => {
                self.filter_search ^= true;
                if !self.search_text.is_empty() && !self.libraries.is_empty() {
                    search(
                        &mut self.libraries[0],
                        &self.search_text,
                        self.filter_search,
                        !self.search_case_sensitive,
                    );
                }
            }

            BrowserEvent::ToggleSearchCaseSensitivity => {
                self.search_case_sensitive ^= true;
                if !self.search_text.is_empty() && !self.libraries.is_empty() {
                    search(
                        &mut self.libraries[0],
                        &self.search_text,
                        self.filter_search,
                        !self.search_case_sensitive,
                    );
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

            BrowserEvent::Select(path, collection) => {
                self.selected.clear();
                self.selected.insert(path.clone());
                self.focused = Some(path.clone());
                cx.emit(AppEvent::ViewCollection(*collection));
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

            // Move selection to the next directory item
            BrowserEvent::SelectNext => {
                if let Some(focused) = &self.focused {
                    let next = recursive_next(&self.libraries[0], None, focused);
                    if let RetItem::Found(next_dir) = next {
                        cx.emit(BrowserEvent::Select(next_dir.path.clone(), next_dir.id));
                    }
                } else {
                    cx.emit(BrowserEvent::Select(
                        self.libraries[0].path.clone(),
                        self.libraries[0].id,
                    ));
                }
            }

            // Move selection to the previous directory item
            BrowserEvent::SelectPrev => {
                if let Some(focused) = &self.focused {
                    let prev = recursive_prev(&self.libraries[0], None, focused);
                    if let RetItem::Found(prev_dir) = prev {
                        cx.emit(BrowserEvent::Select(prev_dir.path.clone(), prev_dir.id));
                    }
                }
            }

            _ => {}
        });
    }
}

#[derive(Debug, Clone)]
enum RetItem<'a> {
    Found(&'a Directory),
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
            return RetItem::Found(root);
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
            return RetItem::Found(prev);
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

fn search<'a>(
    root: &'a mut Directory,
    search_text: &String,
    filter: bool,
    ignore_case: bool,
) -> bool {
    let mut parent_is_shown = !filter;
    let mut matcher = SkimMatcherV2::default();

    if ignore_case {
        matcher = matcher.ignore_case()
    } else {
        matcher = matcher.respect_case()
    }

    if let Some((_, indices)) = matcher.fuzzy_indices(&root.name, search_text) {
        root.match_indices = indices;
        parent_is_shown = true;
    } else {
        root.match_indices.clear();
    }

    let mut child_is_shown = false;
    for child in root.children.iter_mut() {
        child_is_shown |= search(child, search_text, filter, ignore_case);
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
