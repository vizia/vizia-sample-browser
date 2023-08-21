use super::prelude::*;

pub trait DatabaseMeta {
    fn update_database(&mut self);
    fn walk_to_new_tree(&mut self, new_tree: &Vec<DirectoryEntry>);
    fn insert_recursive_directory(&mut self, parent_path: &PathBuf, path: &PathBuf);
}

impl DatabaseMeta for Database {
    /// Queries all directories and files of the database path and updates the database
    fn update_database(&mut self) {
        // Create new hash
        let mut hasher = DefaultHasher::new();
        let tree = build_dir_trees_from_directory(&self.path);

        for entry in tree.iter() {
            entry.hash(&mut hasher);
        }

        let hash_generated = hasher.finish();

        // Update metadata and database if required

        if hash_generated != self.meta.hash_id {
            // Update database & entries
            self.walk_to_new_tree(&tree);

            self.meta.entries = tree;
            self.meta.hash_id = hash_generated;
        }
    }

    fn walk_to_new_tree(&mut self, new_tree: &Vec<DirectoryEntry>) {
        /// Returns all children of the directory in a tree
        fn children_of_directory<'a>(
            tree: &'a Vec<DirectoryEntry>,
            dir: &'a DirectoryEntry,
        ) -> Vec<&'a DirectoryEntry> {
            tree.iter().filter(|v| v.parent_path == Some(dir.path.clone())).collect()
        }

        let root = self.meta.entries.iter().find(|v| v.parent.is_none()).unwrap();

        let mut dir_stack = VecDeque::new();
        let mut to_add = Vec::new();
        let mut to_remove = Vec::new();

        dir_stack.push_back(root.clone());

        // Recursively walk the directories storing each change for later
        while let Some(dir) = dir_stack.pop_front() {
            let mut children = children_of_directory(&self.meta.entries, &dir);
            let mut children_new = children_of_directory(new_tree, &dir);

            let changes = compare_vec(&children, &children_new);

            for change in changes {
                match change {
                    Change::Add(c) => {
                        // Add directory recursively here
                        to_add.push((dir.path.clone(), c.clone()));
                        let pos = children_new.iter().position(|v| *v == c).unwrap();
                        children_new.remove(pos);
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

            children.iter().for_each(|v| dir_stack.push_back((*v).clone()))
        }

        // Insert new directories
        for (path, dir) in to_add {
            self.insert_recursive_directory(&path, &dir.path);
        }

        // Remove directories that need to be removed
        for (path, dir) in to_remove {
            let coll = self.get_collection_by_path(&dir.path).unwrap();
            println!("REMOVE COLLECTION {:?}", coll);
            self.remove_collection(coll.id());
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
}
