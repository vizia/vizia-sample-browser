use std::sync::{Arc, Mutex};

use basedrop::Collector;
use vizia::prelude::*;

use crate::{
    database::prelude::{AudioFile, CollectionID, Database, DatabaseAudioFileHandler},
    engine::{SamplePlayerController, Waveform},
    state::{
        browser::{BrowserState, Directory},
        TagsState,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum ChannelMode {
    Left,
    Right,
    Both,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnitsMode {
    Linear,
    Decibel,
}

/// The state of the playhead.
#[derive(Debug, Clone, PartialEq)]
pub enum PlayState {
    Playing,
    Paused,
    Stopped,
}

/// Whether the zoom should be focused at the playback cursor or the mouse.
#[derive(Debug, Clone, PartialEq)]
pub enum ZoomMode {
    Cursor,
    Mouse,
}

#[derive(Lens)]
pub struct AppData {
    // GUI State
    pub browser: BrowserState,
    pub tags: TagsState,
    pub browser_width: f32,
    pub table_height: f32,
    pub table_headers: Vec<String>,
    pub table_rows: Vec<AudioFile>,
    pub search_text: String,
    pub selected_sample: Option<usize>,

    // Database
    #[lens(ignore)]
    pub database: Arc<Mutex<Database>>,

    // Audio Engine
    #[lens(ignore)]
    pub collector: Collector,
    #[lens(ignore)]
    pub controller: SamplePlayerController,

    // Audio GUI State
    pub waveform: Waveform,
    pub zoom_level: usize,
    pub start: usize,
}

pub enum AppEvent {
    SetBrowserWidth(f32),
    SetTableHeight(f32),
    ViewCollection(CollectionID),

    // Audio Control Events
    LoadSample(String),
    Play,
    Pause,
    Stop,
    // SeekLeft,
    // SeekRight,
}

impl Model for AppData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        self.browser.event(cx, event);
        self.tags.event(cx, event);

        event.map(|app_event, _| match app_event {
            AppEvent::SetBrowserWidth(width) => self.browser_width = *width,
            AppEvent::SetTableHeight(height) => self.table_height = *height,
            AppEvent::ViewCollection(id) => {
                if let Ok(db) = self.database.lock() {
                    if let Ok(audio_files) = db.get_child_audio_files(*id) {
                        self.table_rows = audio_files;
                    }
                }
            }

            AppEvent::LoadSample(path) => {
                self.controller.load_file(path);

                if let Some(file) = self.controller.file.as_ref() {
                    // self.num_of_channels = file.num_channels;
                    // self.sample_rate = file.sample_rate;
                    // self.num_of_samples = file.num_samples;
                    // println!("Length: {} ", self.num_of_samples);

                    self.waveform.load(&file.data[0..file.num_samples], 800);
                }
            }

            AppEvent::Play => {
                self.controller.seek(0.0);
                self.controller.play();
            }

            AppEvent::Pause => {
                self.controller.stop();
            }

            AppEvent::Stop => {
                self.controller.stop();
                self.controller.seek(0.0);
            }
        });
    }
}
