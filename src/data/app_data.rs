use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

use basedrop::{Collector, Owned, Shared};
use creek::{Decoder, ReadDiskStream, ReadStreamOptions, SymphoniaDecoder};
use rfd::FileDialog;
use ron::de::from_reader;
use serde::{Deserialize, Serialize};
use vizia::prelude::*;

use crate::{
    data::{
        browser_data::{BrowserData, Directory},
        TagsData,
    },
    database::{
        prelude::{
            AudioFile, CollectionID, Database, DatabaseAudioFileHandler, DatabaseCollection,
        },
        DatabaseTags,
    },
    engine::{SamplePlayerController, Waveform},
    AudioData, Collection, PlayerState, Tag,
};

use super::{Config, SamplesData, SettingsData};

#[derive(Debug, Clone, PartialEq)]
pub enum ChannelMode {
    Left,
    Right,
    Both,
}

/// The units mode for the waveform.
#[derive(Debug, Clone, PartialEq)]
pub enum UnitsMode {
    // The waveform is displayed in linear samples.
    Linear,
    // The waveform is displayed in decibels.
    Decibel,
}

/// The state of the playhead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayState {
    // The audio is currently playing.
    Playing,
    // The audio is currently paused.
    Paused,
    // The audio is currently stopped.
    Stopped,
}

/// Whether the zoom should be focused at the playback cursor or the mouse.
#[derive(Debug, Clone, PartialEq)]
pub enum ZoomMode {
    // The zoom is focused at the playback cursor.
    Cursor,
    // The zoom is focused at the mouse.
    Mouse,
}

#[derive(Lens)]
pub struct AppData {
    // Timer
    timer: Timer,
    // Dialogs
    // Whether the about dialog should be shown.
    pub show_about_dialog: bool,
    // Whether the settings dialog should be shown.
    pub show_settings_dialog: bool,
    // Whether the add collection dialog should be shown.
    pub show_add_collection_dialog: bool,

    // GUI State
    // The data model for the browser panel.
    pub browser_data: BrowserData,
    // The data model for the samples view.
    pub samples_data: SamplesData,
    // The data model for the tags panel.
    pub tags_data: TagsData,
    // The data model for the settings dialog.
    pub settings_data: SettingsData,

    // The configuration data of the application
    pub config: Config,

    // Database
    #[lens(ignore)]
    pub database: Option<Arc<Mutex<Database>>>,

    // Audio Engine
    #[lens(ignore)]
    pub collector: Collector,
    pub controller: SamplePlayerController,

    // Audio GUI State
    pub waveform: Option<Arc<Waveform>>,
    pub zoom_level: usize,
    pub start: usize,

    pub should_autoplay: bool,

    pub selected_file_name: String,
    pub selected_file_sample_rate: u32,
    pub selected_file_bit_depth: u32,
    pub selected_file_num_channels: u32,
}

impl AppData {
    pub fn new(collector: Collector, controller: SamplePlayerController, timer: Timer) -> Self {
        Self {
            timer,
            // GUI State
            browser_data: BrowserData::new(),
            samples_data: SamplesData::new(),
            tags_data: TagsData::default(),

            config: Config::new(),

            // Database
            database: None,

            // Audio Engine
            collector,
            controller,

            waveform: None,
            zoom_level: 9,
            start: 0,
            show_about_dialog: false,
            show_settings_dialog: false,
            show_add_collection_dialog: false,
            settings_data: SettingsData::dummy(),
            should_autoplay: true,
            selected_file_name: String::new(),
            selected_file_sample_rate: 0,
            selected_file_bit_depth: 0,
            selected_file_num_channels: 0,
        }
    }
}

pub struct Testy(pub Owned<ReadDiskStream<SymphoniaDecoder>>);
unsafe impl Send for Testy {}
unsafe impl Sync for Testy {}

pub enum AppEvent {
    // Show the about dialog.
    ShowAboutDialog,
    // Hide the about dialog.
    HideAboutDialog,
    // Show the settings dialog.
    ShowSettingsDialog,
    // Hide the settings dialog.
    HideSettingsDialog,
    // Show the add collection dialog.
    ShowAddCollectionDialog,
    // Hide the add collection dialog.
    HideAddCollectionDialog,

    // Show the open collection dialog.
    ShowOpenCollectionDialog,

    // View a collection.
    ViewCollection(CollectionID),
    // Update the samples table with the given audio files.
    UpdateTable(Vec<AudioFile>),

    // Open a collection from the given path.
    OpenCollection(PathBuf),
    // The collection has been opened.
    CollectionOpened(Database, Directory, Vec<Tag>),

    // Select a sample from the given collection.
    SelectSample(CollectionID, String),

    // Audio Control Events
    LoadSample(PathBuf),
    SampleLoaded(Testy),
    AppendWaveform(Vec<f32>, usize),
    Play,
    Pause,
    Stop,
    // SeekLeft,
    // SeekRight,
    Tick,
    ToggleLooping,
    ToggleAutoplay,
}

fn view_collection(id: usize, db: &MutexGuard<Database>, rows: &mut Vec<AudioFile>) {
    if let Ok(audio_files) = db.get_child_audio_files(id) {
        rows.extend(audio_files.into_iter());
    }

    for row in rows.iter() {
        if let Ok(tags) = db.get_tags_for_audio_file(row.id) {
            // Add tags to the audio file
        }
    }

    if let Ok(child_collections) = db.get_child_collections(id) {
        for child in child_collections {
            view_collection(child.id(), db, rows);
        }
    }
}

impl Model for AppData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        // Handle events for all the data models
        self.browser_data.event(cx, event);
        self.samples_data.event(cx, event);
        self.tags_data.event(cx, event);
        self.settings_data.event(cx, event);
        self.config.event(cx, event);

        event.take(|app_event, _| match app_event {
            AppEvent::ShowAboutDialog => self.show_about_dialog = true,
            AppEvent::HideAboutDialog => self.show_about_dialog = false,
            AppEvent::ShowSettingsDialog => self.show_settings_dialog = true,
            AppEvent::HideSettingsDialog => self.show_settings_dialog = false,
            AppEvent::ShowAddCollectionDialog => self.show_add_collection_dialog = true,
            AppEvent::HideAddCollectionDialog => self.show_add_collection_dialog = false,

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
                self.samples_data.selected = None;
            }

            AppEvent::UpdateTable(audio_files) => {
                self.samples_data.table_rows = audio_files;
            }

            AppEvent::LoadSample(path) => {
                let collector_handle = self.collector.handle();
                let path2 = path.clone();
                cx.spawn(move |cx| {
                    let opts = ReadStreamOptions {
                        // The number of prefetch blocks in a cache block. This will cause a cache to be
                        // used whenever the stream is seeked to a frame in the range:
                        //
                        // `[cache_start, cache_start + (num_cache_blocks * block_size))`
                        //
                        // If this is 0, then the cache is only used when seeked to exactly `cache_start`.
                        num_cache_blocks: 20,

                        // The maximum number of caches that can be active in this stream. Keep in mind each
                        // cache uses some memory (but memory is only allocated when the cache is created).
                        //
                        // The default is `1`.
                        num_caches: 2,
                        ..Default::default()
                    };

                    // This is how to calculate the total size of a cache block.
                    let cache_size = opts.num_cache_blocks * SymphoniaDecoder::DEFAULT_BLOCK_SIZE;

                    // Open the read stream.
                    let mut read_stream =
                        ReadDiskStream::<SymphoniaDecoder>::new(path, 0, opts).unwrap();

                    // Cache the start of the file into cache with index `0`.
                    let _ = read_stream.cache(0, 0);

                    // Tell the stream to seek to the beginning of file. This will also alert the stream to the existence
                    // of the cache with index `0`.
                    read_stream.seek(0, Default::default()).unwrap();

                    // Wait until the buffer is filled before sending it to the process thread.
                    read_stream.block_until_ready().unwrap();
                    let audio_file = Owned::new(&collector_handle, read_stream);
                    cx.emit(AppEvent::SampleLoaded(Testy(audio_file)));
                });

                self.waveform = Some(Arc::new(Waveform::new()));

                cx.spawn(move |cx| {
                    let opts = ReadStreamOptions {
                        // The number of prefetch blocks in a cache block. This will cause a cache to be
                        // used whenever the stream is seeked to a frame in the range:
                        //
                        // `[cache_start, cache_start + (num_cache_blocks * block_size))`
                        //
                        // If this is 0, then the cache is only used when seeked to exactly `cache_start`.
                        num_cache_blocks: 20,

                        // The maximum number of caches that can be active in this stream. Keep in mind each
                        // cache uses some memory (but memory is only allocated when the cache is created).
                        //
                        // The default is `1`.
                        num_caches: 2,
                        ..Default::default()
                    };

                    // This is how to calculate the total size of a cache block.
                    let cache_size = opts.num_cache_blocks * SymphoniaDecoder::DEFAULT_BLOCK_SIZE;

                    // Open the read stream.
                    let mut read_stream =
                        ReadDiskStream::<SymphoniaDecoder>::new(path2, 0, opts).unwrap();

                    // Cache the start of the file into cache with index `0`.
                    let _ = read_stream.cache(0, 0);

                    // Tell the stream to seek to the beginning of file. This will also alert the stream to the existence
                    // of the cache with index `0`.
                    read_stream.seek(0, Default::default()).unwrap();

                    let mut pos = 0usize;

                    while pos < read_stream.info().num_frames {
                        if let Ok(_) = read_stream.block_until_ready() {
                            if let Ok(ready) = read_stream.is_ready() {
                                if ready {
                                    cx.emit(AppEvent::AppendWaveform(
                                        read_stream.read(8192).unwrap().read_channel(0).to_owned(),
                                        read_stream.info().num_frames,
                                    ));
                                    pos = read_stream.playhead();
                                }
                            }
                        }
                    }
                });
            }

            AppEvent::SampleLoaded(audio_file) => {
                self.controller.load_file(audio_file.0);
                self.controller.seek(0);

                if self.should_autoplay {
                    self.controller.play();
                } else {
                    self.controller.stop();
                }

                cx.start_timer(self.timer);
            }

            AppEvent::AppendWaveform(data, total_frames) => {
                if let Some(waveform) = &mut self.waveform {
                    let wf = Arc::make_mut(waveform);
                    let samples_per_pixel =
                        (total_frames as f32 / self.config.waveview_width).ceil() as usize;
                    wf.append(data.as_slice(), samples_per_pixel);
                }
            }

            AppEvent::Play => {
                if self.controller.play_state == PlayerState::Playing {
                    self.controller.stop();
                    cx.stop_timer(self.timer);
                } else {
                    self.controller.play();
                    cx.start_timer(self.timer);
                }
            }

            AppEvent::Pause => {
                self.controller.pause();
                cx.stop_timer(self.timer);
            }

            AppEvent::Stop => {
                self.controller.stop();
                cx.stop_timer(self.timer);
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
                self.config.libraries.insert(root.path.clone());
                self.config.recents.push(root.path.clone());
                self.browser_data.libraries.push(root);
                self.tags_data.tags = tags;
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
            AppEvent::Tick => {}
            AppEvent::ToggleLooping => self.controller.toggle_looping(),
            AppEvent::ToggleAutoplay => self.should_autoplay = !self.should_autoplay,
        });

        event.map(|window_event, _| match window_event {
            WindowEvent::WindowClose => {
                if let Some(window) = cx.window() {
                    let size: (u32, u32) = window.inner_size().into();
                    self.config.window_size = (
                        (size.0 as f32 / cx.scale_factor()) as u32,
                        (size.1 as f32 / cx.scale_factor()) as u32,
                    );
                    let position: (i32, i32) = window.outer_position().unwrap_or_default().into();
                    self.config.window_position = (
                        (position.0 as f32 / cx.scale_factor()) as i32,
                        (position.1 as f32 / cx.scale_factor()) as i32,
                    );
                }

                self.config.save();
            }

            _ => {}
        })
    }
}

/// Recursively convert a list of collections into a tree of directories.
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
