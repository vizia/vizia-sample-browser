use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

use basedrop::{Collector, Shared};
use rfd::FileDialog;
use vizia::prelude::*;

use crate::{
    data::{
        browser_data::{BrowserData, Directory},
        TagsData,
    },
    database::prelude::{
        AudioFile, CollectionID, Database, DatabaseAudioFileHandler, DatabaseCollection,
    },
    engine::{SamplePlayerController, Waveform},
    AudioData, Collection, DatabaseTags, Tag,
};

use super::{SettingsData, TableData};

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
    // Dialogs
    pub show_about_dialog: bool,
    pub show_settings_dialog: bool,
    pub show_add_collection_dialog: bool,

    // GUI State
    pub browser: BrowserData,
    pub table: TableData,
    pub tags: TagsData,
    pub browser_width: f32,
    pub browser_height: f32,
    pub table_height: f32,

    pub search_text: String,
    pub selected_sample: Option<usize>,
    pub settings_data: SettingsData,

    // Database
    #[lens(ignore)]
    pub database: Option<Arc<Mutex<Database>>>,

    // Audio Engine
    #[lens(ignore)]
    pub collector: Collector,
    #[lens(ignore)]
    pub controller: SamplePlayerController,

    // Audio GUI State
    pub waveform: Arc<Waveform>,
    pub zoom_level: usize,
    pub start: usize,

    pub should_loop: bool,
    pub should_autoplay: bool,

    pub selected_file_name: String,
    pub selected_file_sample_rate: u32,
    pub selected_file_bit_depth: u32,
    pub selected_file_num_channels: u32,
}

impl AppData {
    pub fn new(collector: Collector, controller: SamplePlayerController) -> Self {
        Self {
            // GUI State
            browser: BrowserData::new(),
            table: TableData::new(),
            tags: TagsData::default(),
            browser_width: 300.0,
            table_height: 550.0,
            browser_height: 500.0,
            search_text: String::new(),
            selected_sample: None,

            // Database
            database: None,

            // Audio Engine
            collector,
            controller,

            waveform: Arc::new(Waveform::new()),
            zoom_level: 4,
            start: 0,
            show_about_dialog: false,
            show_settings_dialog: false,
            show_add_collection_dialog: false,
            settings_data: SettingsData::dummy(),
            should_loop: true,
            should_autoplay: true,
            selected_file_name: String::new(),
            selected_file_sample_rate: 0,
            selected_file_bit_depth: 0,
            selected_file_num_channels: 0,
        }
    }
}

pub enum AppEvent {
    ShowAboutDialog,
    HideAboutDialog,
    ShowSettingsDialog,
    HideSettingsDialog,
    ShowAddCollectionDialog,
    HideAddCollectionDialog,

    ShowOpenCollectionDialog,

    SetBrowserWidth(f32),
    SetTableHeight(f32),
    SetBrowserHeight(f32),
    ViewCollection(CollectionID),
    UpdateTable(Vec<AudioFile>),

    OpenCollection(PathBuf),
    CollectionOpened(Database, Directory, Vec<Tag>),

    SelectSample(CollectionID, String),

    // Audio Control Events
    LoadSample(PathBuf),
    SampleLoaded(Shared<AudioData>),
    Play,
    Pause,
    Stop,
    // SeekLeft,
    // SeekRight,
}

fn view_collection(id: usize, db: &MutexGuard<Database>, rows: &mut Vec<AudioFile>) {
    if let Ok(audio_files) = db.get_child_audio_files(id) {
        rows.extend(audio_files.into_iter());
    }

    if let Ok(child_collections) = db.get_child_collections(id) {
        for child in child_collections {
            view_collection(child.id(), db, rows);
        }
    }
}

impl Model for AppData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        self.browser.event(cx, event);
        self.table.event(cx, event);
        self.tags.event(cx, event);
        self.settings_data.event(cx, event);

        event.take(|app_event, _| match app_event {
            AppEvent::ShowAboutDialog => self.show_about_dialog = true,
            AppEvent::HideAboutDialog => self.show_about_dialog = false,
            AppEvent::ShowSettingsDialog => self.show_settings_dialog = true,
            AppEvent::HideSettingsDialog => self.show_settings_dialog = false,
            AppEvent::ShowAddCollectionDialog => self.show_add_collection_dialog = true,
            AppEvent::HideAddCollectionDialog => self.show_add_collection_dialog = false,
            AppEvent::SetBrowserWidth(width) => self.browser_width = width,
            AppEvent::SetTableHeight(height) => self.table_height = height,
            AppEvent::SetBrowserHeight(height) => self.browser_height = height,
            AppEvent::ViewCollection(id) => {
                if let Some(database) = &self.database {
                    let database = database.clone();
                    cx.spawn(move |cx| {
                        let mut audio_files = Vec::with_capacity(500);
                        if let Ok(db) = database.lock() {
                            view_collection(id, &db, &mut audio_files);
                        }
                        cx.emit(AppEvent::UpdateTable(audio_files));
                    });
                }
                self.table.selected = None;
            }

            AppEvent::UpdateTable(audio_files) => {
                self.table.table_rows = audio_files;
            }

            AppEvent::LoadSample(path) => {
                let collector_handle = self.collector.handle();
                cx.spawn(move |cx| {
                    let audio_file = Shared::new(
                        &collector_handle,
                        AudioData::open(path).expect("file does not exist"),
                    );
                    cx.emit(AppEvent::SampleLoaded(audio_file));
                });
            }

            AppEvent::SampleLoaded(audio_file) => {
                self.controller.load_file(audio_file);

                if let Some(file) = self.controller.file.as_ref() {
                    self.selected_file_num_channels = file.num_channels as u32;
                    self.selected_file_sample_rate = file.sample_rate as u32;
                    self.selected_file_bit_depth = file.bits_per_sample as u32;
                    // self.num_of_samples = file.num_samples;
                    // println!("Length: {} ", self.num_of_samples);

                    let wf = Arc::make_mut(&mut self.waveform);

                    wf.load(&file.data[0..file.num_samples], 800);

                    //self.waveform = *wf;
                }
                self.controller.stop();
                self.controller.seek(0.0);
                self.controller.play();
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

            AppEvent::ShowOpenCollectionDialog => {
                if let Some(folder) = FileDialog::new().set_directory("/").pick_folder() {
                    cx.emit(AppEvent::OpenCollection(folder));
                }
            }
            AppEvent::OpenCollection(path) => {
                cx.spawn(|cx| {
                    if let Ok(db) = Database::from_directory(path) {
                        let collections = db.get_all_collections().unwrap();
                        let audio_files = db.get_all_audio_files().unwrap();
                        let mut tags = db.get_all_tags().unwrap();
                        tags.sort_by_cached_key(|tag| tag.name.clone());
                        let root =
                            collections.iter().find(|v| v.parent_collection().is_none()).unwrap();

                        let root =
                            collections_to_directories(&collections, &audio_files, root.clone());

                        cx.emit(AppEvent::CollectionOpened(db, root, tags));
                    }
                });
            }

            AppEvent::CollectionOpened(database, root, tags) => {
                self.database = Some(Arc::new(Mutex::new(database)));
                self.browser.libraries.push(root);
                self.tags.tags = tags;
            }

            AppEvent::SelectSample(collection_id, name) => {
                if let Some(database) = &self.database {
                    //let database = database.clone();
                    if let Ok(db) = database.lock() {
                        if let Ok(collection) = db.get_collection(collection_id) {
                            let path: PathBuf = collection.path().join(&name);

                            self.selected_file_name = name;
                            cx.emit(AppEvent::LoadSample(path));
                        }
                    }
                }
            }
        });
    }
}

fn collections_to_directories(
    collections: &Vec<Collection>,
    audio_files: &Vec<AudioFile>,
    current: Collection,
) -> Directory {
    let children: Vec<Directory> = collections
        .iter()
        .filter(|v| v.parent_collection() == Some(current.id()))
        .map(|v| collections_to_directories(collections, audio_files, v.clone()))
        .collect();

    let afs: Vec<&AudioFile> =
        audio_files.iter().filter(|v| v.collection == current.id()).collect();

    Directory {
        id: current.id(),
        parent_id: current.parent_collection(),
        name: current.name().to_string(),
        path: current.path().clone(),
        shown: true,
        is_open: false,
        num_files: children.iter().map(|v| v.num_files).sum::<usize>() + afs.len(),
        children,
        ..Default::default()
    }
}
