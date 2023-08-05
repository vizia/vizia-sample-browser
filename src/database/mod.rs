use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub mod handler;
pub use handler::*;

pub mod connection;
pub use connection::*;

pub mod store;
pub use store::*;

pub mod collection;
pub use collection::*;

pub mod error;
pub use error::*;

pub mod audio_file;
pub use audio_file::*;

mod tests;

pub type TagID = String;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    id: TagID,
    color: f32,
}

struct AudioFilesTag {
    audio_file: AudioFileID,
    tag: TagID,
}

impl From<Tag> for String {
    fn from(value: Tag) -> Self {
        value.id
    }
}

fn file_exists(path: &PathBuf) -> bool {
    std::fs::read(path).is_ok()
}

fn directory_exists(path: &PathBuf) -> bool {
    std::fs::read_dir(path).is_ok()
}
