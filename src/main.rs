#![allow(unused)] // Disable stupid warnings for now

use app_data::AppData;
use basedrop::Collector;
use cpal::traits::StreamTrait;
use itertools::Itertools;
use rusqlite::Connection;
use std::{
    collections::{HashMap, VecDeque},
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, Mutex},
};
use views::smart_table::SmartTable;
use vizia::{
    icons::{ICON_LIST_SEARCH, ICON_SEARCH},
    prelude::{GenerationalId, *},
};

mod state;
use state::*;

mod database;
use database::*;

mod panels;
use panels::*;

mod views;
use views::*;

mod engine;
use engine::*;

mod popup_menu;

fn main() {
    // Initialize gc
    let collector = Collector::new();

    // Create the sample player and controller
    let (mut player, mut controller) = sample_player(&collector);

    // Initialize state and begin the stream
    std::thread::spawn(move || {
        let stream = audio_stream(move |mut context| {
            player.advance(&mut context);
        });

        // TODO - handle error
        stream.play();

        std::thread::park();
    });

    Application::new(move |cx| {
        // Add resources
        cx.add_stylesheet(include_style!("resources/themes/style.css"))
            .expect("Failed to load stylesheet");

        cx.add_translation(
            langid!("en-US"),
            include_str!("../resources/translations/en-US/browser.ftl"),
        );

        cx.add_translation(langid!("es"), include_str!("../resources/translations/es/browser.ftl"));

        cx.emit(EnvironmentEvent::SetThemeMode(AppTheme::BuiltIn(ThemeMode::DarkMode)));

        // Uncomment to test in Spanish
        // cx.emit(EnvironmentEvent::SetLocale(langid!("es")));

        let headers =
            vec!["Name", "Tags", "Duration", "Sample Rate", "Bit Depth", "BPM", "Key", "Size"]
                .iter_mut()
                .map(|v| v.to_string())
                .collect::<Vec<_>>();

        let mut db =
            Database::from_directory(Path::new("the-libre-sample-pack/").to_path_buf()).unwrap();

        let collections = db.get_all_collections().unwrap();
        let audio_files = db.get_all_audio_files().unwrap();

        let root = collections.iter().find(|v| v.parent_collection().is_none()).unwrap();

        let root = collections_to_directories(&collections, &audio_files, root.clone());

        let audio_files = db.get_all_audio_files().unwrap().len();

        AppData {
            // GUI State
            browser: BrowserState::new(root),
            tags: TagsState::default(),
            browser_width: 300.0,
            table_height: 300.0,
            table_headers: headers,
            table_rows: Vec::new(),
            search_text: String::new(),
            selected_sample: None,

            // Database
            database: Arc::new(Mutex::new(db)),

            // Audio Engine
            collector,
            controller,

            waveform: Waveform::new(),
            zoom_level: 8,
            start: 0,
        }
        .build(cx);

        cx.emit(AppEvent::LoadSample(String::from(
            "/Users/gatkinson/Rust/vizia-sample-browser/the-libre-sample-pack/drums/one shot/kicks/couch kick 1 @TeaBoi.wav",
        )));

        HStack::new(cx, |cx| {
            ResizableStack::new(
                cx,
                AppData::browser_width,
                ResizeStackDirection::Right,
                |cx, width| cx.emit(AppEvent::SetBrowserWidth(width)),
                |cx| {
                    BrowserPanel::new(cx);
                    TagsPanel::new(cx);
                },
            )
            .row_between(Pixels(1.0))
            .class("browser");

            VStack::new(cx, |cx| {
                // Samples Panel
                ResizableStack::new(
                    cx,
                    AppData::table_height,
                    ResizeStackDirection::Bottom,
                    |cx, height| cx.emit(AppEvent::SetTableHeight(height)),
                    |cx| {
                        SamplesPanel::new(cx);
                    },
                );
                // Waveform Panel
                WavePanel::new(cx);
            })
            .row_between(Pixels(1.0));
        })
        .col_between(Pixels(1.0))
        .size(Stretch(1.0));
    })
    .title("Vizia Sample Browser")
    .inner_size((1400, 800))
    .run();
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
