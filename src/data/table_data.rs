use std::path::PathBuf;

use super::app_data::{AppData, AppEvent};
use crate::database::prelude::*;

use vizia::prelude::*;

#[derive(Debug, Lens, Clone, Default)]
pub struct TableData {
    pub table_headers: Vec<String>,
    pub table_rows: Vec<AudioFile>,
    pub selected: Option<usize>,
}

impl TableData {
    pub fn new() -> Self {
        let headers =
            vec!["Name", "Tags", "Duration", "Sample Rate", "Bit Depth", "BPM", "Key", "Size", ""]
                .iter_mut()
                .map(|v| v.to_string())
                .collect::<Vec<_>>();

        Self { table_headers: headers, ..Default::default() }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableEvent {
    Select(usize),
    Deselect,
    SelectNext,
    SelectPrev,
}

impl Model for TableData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|table_event, _| match table_event {
            TableEvent::Select(row_index) => {
                self.selected = Some(*row_index);
                if let Some(audio_file) = self.table_rows.get(*row_index) {
                    cx.emit(AppEvent::SelectSample(audio_file.collection, audio_file.name.clone()));
                }
            }

            TableEvent::Deselect => {
                self.selected = None;
            }

            // Move focus the next directory item
            TableEvent::SelectNext => {}

            // Move selection the previous directory item
            TableEvent::SelectPrev => {}

            _ => {}
        });
    }
}
