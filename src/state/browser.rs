use crate::{app_data::AppData, database::prelude::*};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::{Path, PathBuf},
};
use vizia::prelude::*;

#[derive(Debug, Lens, Clone, Data)]
pub struct BrowserState {
    pub libraries: Vec<Directory>,
    pub selected: HashSet<PathBuf>,
    pub focused: Option<PathBuf>,
    pub search_text: String,
    pub filter_search: bool,
    pub search_case_sensitive: bool,
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
    ToggleSearchCaseSensitivity,
    ShowTree,
    ShowList,
}

#[derive(Debug, Clone, Data, Lens)]
pub struct Directory {
    pub id: CollectionID,
    pub parent_id: Option<CollectionID>,
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
            libraries: Vec::new(),
            selected: HashSet::new(),
            focused: None,
            search_text: String::new(),
            filter_search: false,
            search_case_sensitive: false,
        }
    }
}

impl Model for BrowserState {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|browser_event, _| match browser_event {
            // Temp: Load the assets directory for the treeview
            BrowserEvent::ViewAll => {
                let db_ref = AppData::database.get(cx);
                let db = db_ref.lock().unwrap();
                let root = collections_to_directories(&mut db.get_all_collections().unwrap());
                self.libraries[0] = root;
            }

            BrowserEvent::Search(search_text) => {
                self.focused = None;
                self.search_text = search_text.clone();
                search(
                    &mut self.libraries[0],
                    &self.search_text,
                    self.filter_search,
                    !self.search_case_sensitive,
                );
            }

            BrowserEvent::ToggleSearchFilter => {
                self.filter_search ^= true;
                if !self.search_text.is_empty() {
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
                if !self.search_text.is_empty() {
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
                    if let RetItem::Found(next_dir) = next {
                        self.focused = Some(next_dir.path.clone());
                        cx.focus_next();
                    }
                }
            }

            // Move selection the previous directory item
            BrowserEvent::FocusPrev => {
                if let Some(focused) = &self.focused {
                    let prev = recursive_prev(&self.libraries[0], None, focused);
                    if let RetItem::Found(prev_dir) = prev {
                        self.focused = Some(prev_dir.path.clone());
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

fn collections_to_directories(collections: &mut Vec<Collection>) -> Directory {
    let mut hm: HashMap<CollectionID, Directory> = HashMap::new();

    for coll in collections {
        hm.insert(
            coll.id(),
            Directory {
                id: coll.id(),
                parent_id: coll.parent_collection(),
                name: coll.name().to_string(),
                path: coll.path().clone(),
                is_open: false,
                num_files: 0,
                shown: true,
                match_indices: Vec::new(),
                children: Vec::new(),
            },
        );
    }

    fn children_of_collection(
        map: &HashMap<CollectionID, Directory>,
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
