use std::path::PathBuf;

use super::app_data::{AppData, AppEvent};
use crate::database::prelude::*;

use vizia::prelude::*;

// The data model for the samples view
#[derive(Debug, Lens, Clone, Default)]
pub struct SamplesData {
    // The headers of the table
    pub table_headers: Vec<(String, bool)>,
    // The rows of the table
    pub table_rows: Vec<AudioFile>,
    // The currently selected row
    pub selected: Option<usize>,
    // The search text in the search box
    pub search_text: String,
}

impl SamplesData {
    pub fn new() -> Self {
        let headers = vec![
            "Name",
            "Tags",
            "Duration",
            "Sample Rate",
            "Bit Depth",
            "Num Channels",
            "BPM",
            "Key",
            "Size",
            "",
        ]
        .iter_mut()
        .map(|v| (v.to_string(), true))
        .collect::<Vec<_>>();

        Self { table_headers: headers, ..Default::default() }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SampleEvent {
    // Select a row in the table
    Select(usize),
    // Deselect the currently selected row
    Deselect,
    // Move selection to the next row
    SelectNext,
    // Move selection to the previous row
    SelectPrev,
    // Hide a column
    HideColumn(usize),
    // Show a column
    ShowColumn(usize),
    // Toggle the visibility of a column
    ToggleColumn(usize),
}

impl Model for SamplesData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|table_event, _| match table_event {
            SampleEvent::Select(row_index) => {
                if *row_index < self.table_rows.len() {
                    self.selected = Some(*row_index);
                    if let Some(audio_file) = self.table_rows.get(*row_index) {
                        cx.emit(AppEvent::SelectSample(
                            audio_file.collection,
                            audio_file.name.clone(),
                        ));
                    }
                }
            }

            SampleEvent::Deselect => {
                self.selected = None;
            }

            SampleEvent::SelectNext => {
                if let Some(selected) = self.selected {
                    cx.emit(SampleEvent::Select(selected + 1));
                }
            }

            SampleEvent::SelectPrev => {
                if let Some(selected) = self.selected {
                    cx.emit(SampleEvent::Select(selected.saturating_sub(1)));
                }
            }

            SampleEvent::ToggleColumn(index) => {
                self.table_headers[*index].1 ^= true;
            }

            _ => {}
        });
    }
}
