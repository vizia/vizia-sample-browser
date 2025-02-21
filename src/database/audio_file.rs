use std::path::PathBuf;

use vizia::prelude::*;

use super::{CollectionID, Database, DatabaseConnection, DatabaseError, AUDIO_FILE_EXTENSIONS};
use serde::{Deserialize, Serialize};

pub type AudioFileID = usize;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Data, Lens)]
pub struct AudioFile {
    pub id: AudioFileID,
    pub name: String,
    pub collection: CollectionID,
    pub duration: f32,
    pub sample_rate: f32,
    pub bit_depth: f32,
    pub num_channels: f32,
    pub bpm: Option<f32>,
    pub key: Option<f32>,
    pub size: f32,
}

impl AudioFile {
    pub fn from_path(path: &PathBuf, id: AudioFileID) -> Option<Self> {
        let extension = path.extension().map(|v| v.to_str().unwrap()).unwrap_or("");

        if !AUDIO_FILE_EXTENSIONS.contains(&extension) {
            return None;
        }

        let name = path.file_name().unwrap().to_str().unwrap();

        let audio_file =
            AudioFile::new(id, name.to_string(), id, 0.0, 0.0, 0.0, 0.0, None, None, 0.0);

        Some(audio_file)
    }

    pub fn new(
        id: AudioFileID,
        name: String,
        collection: CollectionID,
        duration: f32,
        sample_rate: f32,
        bit_depth: f32,
        num_channels: f32,
        bpm: Option<f32>,
        key: Option<f32>,
        size: f32,
    ) -> Self {
        Self {
            id,
            name,
            collection,
            duration,
            sample_rate,
            bit_depth,
            num_channels,
            bpm,
            key,
            size,
        }
    }
}

pub trait DatabaseAudioFileHandler {
    fn get_all_audio_files(&self) -> Result<Vec<AudioFile>, DatabaseError>;
    fn get_audio_file_by_path(&self, path: &PathBuf) -> Result<AudioFile, DatabaseError>;
    fn get_child_audio_files(&self, parent: CollectionID) -> Result<Vec<AudioFile>, DatabaseError>;
    fn insert_audio_file(&mut self, audio_file: AudioFile) -> Result<(), DatabaseError>;
    fn remove_audio_file(&mut self, audio_file: AudioFileID) -> Result<(), DatabaseError>;
}

impl DatabaseAudioFileHandler for Database {
    fn get_all_audio_files(&self) -> Result<Vec<AudioFile>, DatabaseError> {
        if let Some(connection) = self.get_connection() {
            let mut query = connection
                .prepare("SELECT id, name, collection, duration, sample_rate, bit_depth, num_channels, bpm, key, size FROM audio_files")?;

            let audio_files = query.query_map([], |row| {
                Ok(AudioFile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    collection: row.get(2)?,
                    duration: row.get(3)?,
                    sample_rate: row.get(4)?,
                    bit_depth: row.get(5)?,
                    num_channels: row.get(6)?,
                    bpm: row.get(7)?,
                    key: row.get(8)?,
                    size: row.get(9)?,
                })
            })?;

            return Ok(audio_files.map(|v| v.unwrap()).collect());
        }

        Err(DatabaseError::ConnectionClosed)
    }

    fn get_audio_file_by_path(&self, path: &PathBuf) -> Result<AudioFile, DatabaseError> {
        if let Some(connection) = self.get_connection() {
            let mut query = connection.prepare(
                "SELECT id, name, collection, duration, sample_rate, bit_depth, num_channels, bpm, key, size FROM audio_files WHERE path = (?1)",
            )?;

            let col: AudioFile = query.query_row([path.to_str().unwrap()], |row| {
                let path: String = row.get(3)?;
                Ok(AudioFile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    collection: row.get(2)?,
                    duration: row.get(3)?,
                    sample_rate: row.get(4)?,
                    bit_depth: row.get(5)?,
                    num_channels: row.get(6)?,
                    bpm: row.get(7)?,
                    key: row.get(8)?,
                    size: row.get(9)?,
                })
            })?;

            return Ok(col);
        }

        Err(DatabaseError::ConnectionClosed)
    }

    fn get_child_audio_files(&self, parent: CollectionID) -> Result<Vec<AudioFile>, DatabaseError> {
        if let Some(connection) = self.get_connection() {
            let mut query = connection.prepare(
                "SELECT id, name, collection, duration, sample_rate, bit_depth, num_channels, bpm, key, size FROM audio_files WHERE collection = (?1)",
            )?;

            let audio_files = query.query_map([parent], |row| {
                Ok(AudioFile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    collection: row.get(2)?,
                    duration: row.get(3)?,
                    sample_rate: row.get(4)?,
                    bit_depth: row.get(5)?,
                    num_channels: row.get(6)?,
                    bpm: row.get(7)?,
                    key: row.get(8)?,
                    size: row.get(9)?,
                })
            })?;

            return Ok(audio_files.map(|v| v.unwrap()).collect());
        }
        Err(DatabaseError::ConnectionClosed)
    }

    fn insert_audio_file(&mut self, audio_file: AudioFile) -> Result<(), DatabaseError> {
        if let Some(connection) = self.get_connection() {
            connection.execute(
                "INSERT INTO audio_files (id, name, collection, duration, sample_rate, bit_depth, num_channels, bpm, key, size) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                (
                    audio_file.id,
                    audio_file.name,
                    audio_file.collection,
                    audio_file.duration,
                    audio_file.sample_rate,
                    audio_file.bit_depth,
                    audio_file.num_channels,
                    audio_file.bpm,
                    audio_file.key,
                    audio_file.size,
                ),
            )?;
        }

        Ok(())
    }

    fn remove_audio_file(&mut self, audio_file: AudioFileID) -> Result<(), DatabaseError> {
        if let Some(connection) = self.get_connection() {
            connection.execute("DELETE FROM audio_files WHERE id = (?1)", [audio_file])?;
        }

        Ok(())
    }
}

impl From<AudioFile> for usize {
    fn from(value: AudioFile) -> Self {
        value.id
    }
}
