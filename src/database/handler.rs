use crate::data::browser_data::Directory;

use super::*;
use creek::{Decoder, SymphoniaDecoder, SymphoniaDecoderInfo};
use hound::WavReader;
use rusqlite::Connection;
use std::{
    any::Any,
    cell::RefCell,
    collections::{BTreeSet, HashMap, VecDeque},
    error::Error,
    fs::{create_dir, read_dir, DirEntry, File},
    path::{Path, PathBuf},
    rc::Rc,
    sync::atomic::AtomicUsize,
};
use vizia::prelude::*;

pub const DATABASE_FILE_NAME: &str = ".database.vsb";
pub const AUDIO_FILE_EXTENSIONS: [&'static str; 1] = ["wav"];

#[derive(Debug, Lens)]
pub struct Database {
    pub(super) path: PathBuf,
    pub(super) conn: Option<Connection>,
    pub(super) meta: DatabaseMetadata,
}

impl Database {
    pub fn from_directory(path: PathBuf) -> Result<Self, DatabaseError> {
        // Check file is directory
        if !directory_exists(&path) {
            return Err(DatabaseError::PathNotDirectory);
        }

        // Open connection
        let mut s = Self { path, conn: None, meta: DatabaseMetadata::new() };
        s.initialize_or_create_stores()?;

        // let database_exists = File::open(s.get_database_path()).is_ok();

        s.open_connection()?;

        s.initialize_empty_database();
        // if !database_exists {
        // } else {
        //     s.update_database();
        // }

        Ok(s)
    }

    fn clear_database(&mut self) {
        self.get_connection().unwrap().execute_batch(include_str!("sqls/clear.sql")).unwrap();
    }

    fn initialize_empty_database(&mut self) {
        let audio_file_count = AtomicUsize::new(0);
        let collection_count = AtomicUsize::new(0);
        let tags_count = AtomicUsize::new(0);

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
                let p = child_file.path();
                let extension = p.extension().map(|v| v.to_str().unwrap()).unwrap_or("");

                // if !AUDIO_FILE_EXTENSIONS.contains(&extension) {
                //     continue;
                // }

                let file_id = audio_file_count.load(std::sync::atomic::Ordering::Relaxed);

                let name = child_file.file_name().to_str().unwrap().to_string();

                if let Ok((_, file_info)) = SymphoniaDecoder::new(p, 0, 0, ()) {
                    let sample_rate = file_info.sample_rate.unwrap_or(41000);
                    let duration = file_info.num_frames as f32 / sample_rate as f32;

                    let audio_file = AudioFile::new(
                        file_id,
                        name,
                        id,
                        duration,
                        sample_rate as f32,
                        0.0,
                        file_info.num_channels as f32,
                        None,
                        None,
                        0.0,
                    );

                    db.insert_audio_file(audio_file);

                    audio_file_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }

                // let mut reader = WavReader::open(p).unwrap();
                // let spec = reader.spec();

                // let duration = reader.duration() as f32 / spec.sample_rate as f32;

                // let audio_file = AudioFile::new(
                //     file_id,
                //     name,
                //     id,
                //     duration,
                //     spec.sample_rate as f32,
                //     spec.bits_per_sample as f32,
                //     spec.channels as f32,
                //     None,
                //     None,
                //     reader.duration() as f32 * spec.channels as f32 * spec.bits_per_sample as f32
                //         / 8.0,
                // );

                // db.insert_audio_file(audio_file);
            }
        });

        self.insert_tag(Tag {
            id: 4,
            name: String::from("UpTempo"),
            color: String::from("orange"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 10,
            name: String::from("Reggae"),
            color: String::from("yellow"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 11,
            name: String::from("Jazz"),
            color: String::from("orange"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 12,
            name: String::from("RnB"),
            color: String::from("purple"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 13,
            name: String::from("EDM"),
            color: String::from("pink"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 14,
            name: String::from("TrapSoul"),
            color: String::from("red"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 15,
            name: String::from("LoFi"),
            color: String::from("blue"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 16,
            name: String::from("Energetic"),
            color: String::from("green"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 17,
            name: String::from("Chill"),
            color: String::from("yellow"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 18,
            name: String::from("Epic"),
            color: String::from("orange"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 19,
            name: String::from("Romantic"),
            color: String::from("purple"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 20,
            name: String::from("DarkAmbient"),
            color: String::from("pink"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 21,
            name: String::from("Bright"),
            color: String::from("red"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 22,
            name: String::from("Groovy"),
            color: String::from("blue"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 23,
            name: String::from("Sad"),
            color: String::from("green"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 24,
            name: String::from("Synth"),
            color: String::from("yellow"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 25,
            name: String::from("Violin"),
            color: String::from("orange"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 26,
            name: String::from("Brass"),
            color: String::from("purple"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 27,
            name: String::from("ElectricGuitar"),
            color: String::from("pink"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 28,
            name: String::from("808"),
            color: String::from("red"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 29,
            name: String::from("Percussion"),
            color: String::from("blue"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 30,
            name: String::from("Orchestral"),
            color: String::from("green"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 31,
            name: String::from("Kick"),
            color: String::from("yellow"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 32,
            name: String::from("Snare"),
            color: String::from("orange"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 33,
            name: String::from("HiHat"),
            color: String::from("purple"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 34,
            name: String::from("Pads"),
            color: String::from("pink"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 35,
            name: String::from("RadioEdit"),
            color: String::from("red"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 36,
            name: String::from("ExtendedMix"),
            color: String::from("blue"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 37,
            name: String::from("LiveVersion"),
            color: String::from("green"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 38,
            name: String::from("VIPMix"),
            color: String::from("yellow"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 39,
            name: String::from("Remastered"),
            color: String::from("orange"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 40,
            name: String::from("Dub"),
            color: String::from("purple"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 41,
            name: String::from("Stems"),
            color: String::from("pink"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 42,
            name: String::from("SamplePack"),
            color: String::from("red"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 43,
            name: String::from("VocalChop"),
            color: String::from("blue"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 44,
            name: String::from("Filtered"),
            color: String::from("green"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 45,
            name: String::from("Glitch"),
            color: String::from("yellow"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 46,
            name: String::from("Clean"),
            color: String::from("orange"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 47,
            name: String::from("Distorted"),
            color: String::from("purple"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 48,
            name: String::from("Reversed"),
            color: String::from("pink"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 49,
            name: String::from("Muted"),
            color: String::from("red"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 50,
            name: String::from("SFX"),
            color: String::from("blue"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 51,
            name: String::from("Atmosphere"),
            color: String::from("green"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 52,
            name: String::from("Foley"),
            color: String::from("yellow"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 53,
            name: String::from("Rise"),
            color: String::from("orange"),
            number: 0,
        });
        self.insert_tag(Tag {
            id: 54,
            name: String::from("Drop"),
            color: String::from("purple"),
            number: 0,
        });
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
        let meta_dir = self.get_meta_directory_path();
        std::fs::remove_dir_all(meta_dir);
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
