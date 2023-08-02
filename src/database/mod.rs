use serde::{Deserialize, Serialize};

pub mod handler;
pub use handler::*;

mod tests;

pub type CollectionID = usize;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Collection {
    id: CollectionID,
    parent_collection: Option<CollectionID>,
    name: String,
}

pub type AudioFileID = usize;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFile {
    id: AudioFileID,
    name: String,
    collection: CollectionID,
    duration: f32,
    sample_rate: f32,
    bit_depth: f32,
    bpm: Option<f32>,
    key: Option<f32>,
    size: f32,
}

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

impl From<Collection> for usize {
    fn from(value: Collection) -> Self {
        value.id
    }
}

impl From<AudioFile> for usize {
    fn from(value: AudioFile) -> Self {
        value.id
    }
}

impl From<Tag> for String {
    fn from(value: Tag) -> Self {
        value.id
    }
}
