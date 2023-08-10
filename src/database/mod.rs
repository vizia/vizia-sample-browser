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

pub mod tags;
pub use tags::*;

mod tests;

pub mod prelude {
    pub use super::audio_file::*;
    pub use super::collection::*;
    pub use super::connection::*;
    pub use super::error::*;
    pub use super::handler::*;
    pub use super::store::*;
    pub use super::tags::*;
    pub use rusqlite::*;
}

fn file_exists(path: &PathBuf) -> bool {
    std::fs::read(path).is_ok()
}

fn directory_exists(path: &PathBuf) -> bool {
    std::fs::read_dir(path).is_ok()
}
