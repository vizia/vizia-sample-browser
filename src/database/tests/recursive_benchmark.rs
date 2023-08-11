use crate::{database::prelude::*, state::browser::Directory};
use std::{
    collections::{HashMap, VecDeque},
    path::Path,
    rc::Rc,
    sync::Mutex,
    time::Instant,
};

#[derive(Clone)]
struct RecursiveInner {
    id: CollectionID,
    parent_id: Option<CollectionID>,
    name: String,
    path: PathBuf,
    children: Vec<Rc<Mutex<RecursiveInner>>>,
}

impl RecursiveInner {
    fn to_directory(inner: &mut RecursiveInner) -> Directory {
        let children = inner
            .children
            .iter_mut()
            .map(|child| RecursiveInner::to_directory(&mut child.lock().unwrap()))
            .collect();

        Directory {
            id: inner.id,
            parent_id: inner.parent_id,
            name: inner.name.clone(),
            path: inner.path.clone(),
            children,
            is_open: false,
            shown: true,
            ..Default::default()
        }
    }
}

#[test]
fn test_recursive_algorithm() {
    let mut db = Database::from_directory(Path::new("test_files/").to_path_buf()).unwrap();

    let collections = db.get_all_collections().unwrap();
    let root = collections.iter().find(|v| v.parent_collection().is_none()).unwrap();

    for n in 0..10 {
        let pre = Instant::now();

        let result = implicit_collections_to_directories(&collections);

        let after = Instant::now();

        if n == 0 {
            print_recursive_structure(&result, 0);
        }

        let elapsed = after - pre;

        println!("(TRY {}) Implicit algorithm time: {} µs", n, elapsed.as_micros());
    }

    for n in 0..10 {
        let pre = Instant::now();

        let result = recursive_collections_to_directories(&collections, root.clone());

        let after = Instant::now();

        if n == 0 {
            print_recursive_structure(&result, 0);
        }

        let elapsed = after - pre;

        println!("(TRY {}) Recursive algorithm time: {} µs", n, elapsed.as_micros());
    }
}

fn implicit_collections_to_directories(collections: &Vec<Collection>) -> Directory {
    let mut hm: HashMap<CollectionID, Rc<Mutex<RecursiveInner>>> = HashMap::new();

    for coll in collections {
        hm.insert(
            coll.id(),
            Rc::new(Mutex::new(RecursiveInner {
                id: coll.id(),
                parent_id: coll.parent_collection(),
                name: coll.name().to_string(),
                path: coll.path().clone(),
                children: Vec::new(),
            })),
        );
    }

    fn children_of_collection(
        map: &HashMap<CollectionID, Rc<Mutex<RecursiveInner>>>,
        coll: CollectionID,
    ) -> VecDeque<CollectionID> {
        map.values()
            .filter(|v| v.lock().unwrap().parent_id == Some(coll))
            .map(|v| v.lock().unwrap().id)
            .collect()
    }

    let mut root_dir = hm.values().find(|v| v.lock().unwrap().parent_id.is_none()).unwrap();
    let mut directory_stack: VecDeque<Rc<Mutex<RecursiveInner>>> = VecDeque::new();
    directory_stack.push_back(root_dir.clone());

    while let Some(mut coll) = directory_stack.pop_front() {
        let id: usize = coll.lock().unwrap().id;
        let mut children = children_of_collection(&hm, id);
        let mut children_dir: VecDeque<Rc<Mutex<RecursiveInner>>> = VecDeque::new();
        children.iter_mut().for_each(|v| children_dir.push_back(hm.get(&v).unwrap().clone()));

        for mut child_ref in children_dir {
            let mut child = child_ref.lock().unwrap();

            // Each child inside the current focused directory appends to the recursive structure
            coll.lock().unwrap().children.push(child_ref.clone());

            // Reference each of those children to iterate in the stack
            directory_stack.push_back(child_ref.clone());
        }
    }

    // Transform root dir to Directory
    let mut root_directory = root_dir.lock().unwrap().clone();
    let directory = RecursiveInner::to_directory(&mut root_directory);

    directory
}

fn recursive_collections_to_directories(
    collections: &Vec<Collection>,
    current: Collection,
) -> Directory {
    let children: Vec<Directory> = collections
        .iter()
        .filter(|v| v.parent_collection() == Some(current.id()))
        .map(|v| recursive_collections_to_directories(collections, v.clone()))
        .collect();

    Directory {
        id: current.id(),
        parent_id: current.parent_collection(),
        name: current.name().to_string(),
        path: current.path().clone(),
        shown: true,
        is_open: false,
        children,
        ..Default::default()
    }
}

fn print_recursive_structure(dir: &Directory, depth: usize) {
    println!("{} - ({}) {}", "    ".repeat(depth), dir.id, dir.name);
    for child in dir.children.iter() {
        print_recursive_structure(child, depth + 1);
    }
}
