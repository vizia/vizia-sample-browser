use vizia::prelude::*;

use super::prelude::{AudioFileID, Database, DatabaseError};
use crate::database::prelude::*;
use serde::{Deserialize, Serialize};

pub type TagID = usize;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Data, Lens)]
pub struct Tag {
    pub id: TagID,
    pub name: String,
    pub color: String,
    pub number: u32,
}

impl Tag {
    pub fn new(id: TagID, name: String, color: String, number: u32) -> Self {
        Self { id, color, name, number }
    }
}

pub trait DatabaseTags {
    fn get_all_tags(&self) -> Result<Vec<Tag>, DatabaseError>;
    fn get_tags_for_audio_file(&self, audio_file: AudioFileID) -> Result<Vec<Tag>, DatabaseError>;
    fn insert_tag(&mut self, tag: Tag) -> Result<(), DatabaseError>;
    fn assign_tag_to_audio_file(
        &mut self,
        tag: TagID,
        audio_file: AudioFileID,
    ) -> Result<(), DatabaseError>;
}

impl DatabaseTags for Database {
    fn get_all_tags(&self) -> Result<Vec<Tag>, DatabaseError> {
        if let Some(connection) = self.get_connection() {
            let mut query = connection.prepare("SELECT id, name, color, number FROM tags")?;

            let tags = query.query_map([], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    number: row.get(3)?,
                })
            })?;

            return Ok(tags.map(|v| v.unwrap()).collect());
        }

        Err(DatabaseError::ConnectionClosed)
    }

    fn get_tags_for_audio_file(&self, audio_file: AudioFileID) -> Result<Vec<Tag>, DatabaseError> {
        if let Some(connection) = self.get_connection() {
            let mut query = connection.prepare(
                "
            SELECT id, name, color, number FROM tags
                WHERE id IN (
                    SELECT tag FROM audio_files_tags
                        WHERE audio_file = (?1)
                )
            ",
            )?;

            let tags = query.query_map([audio_file], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    number: row.get(3)?,
                })
            })?;

            return Ok(tags.map(|v| v.unwrap()).collect());
        }

        Err(DatabaseError::ConnectionClosed)
    }

    fn insert_tag(&mut self, tag: Tag) -> Result<(), DatabaseError> {
        if let Some(connection) = self.get_connection() {
            connection.execute(
                "INSERT INTO tags (id, name, color, number) VALUES (?1, ?2, ?3, ?4)",
                (tag.id, tag.name, tag.color, tag.number),
            )?;
        }

        Ok(())
    }

    fn assign_tag_to_audio_file(
        &mut self,
        tag: TagID,
        audio_file: AudioFileID,
    ) -> Result<(), DatabaseError> {
        if let Some(connection) = self.get_connection() {
            connection.execute(
                "INSERT INTO audio_files_tags (audio_file, tags) VALUES (?1, ?2)",
                (audio_file, tag),
            )?;
        }

        Ok(())
    }
}

struct AudioFilesTag {
    audio_file: AudioFileID,
    tag: TagID,
}

impl From<Tag> for String {
    fn from(value: Tag) -> Self {
        value.name
    }
}
