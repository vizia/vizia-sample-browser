use serde::{Deserialize, Serialize};

pub mod handler;
pub use handler::*;

mod tests;

pub type CollectionID = i32;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Collection {
    id: CollectionID,
    parent_collection: Option<CollectionID>,
    name: String,
}

pub type AudioFileID = i32;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFiles {
    id: AudioFileID,
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

struct AudioFilesTags {
    audio_file: AudioFileID,
    tag: TagID,
}
