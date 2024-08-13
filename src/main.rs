#![allow(unused)] // Disable stupid warnings for now

use app_data::AppData;
use basedrop::Collector;
use cpal::traits::StreamTrait;
use itertools::Itertools;
use rusqlite::Connection;
use thiserror::Error;
use std::{
    collections::{HashMap, VecDeque}, error::Error, path::{Path, PathBuf}, rc::Rc, sync::{Arc, Mutex}
};
use views::smart_table::SmartTable;
use vizia::{
    icons::{ICON_LIST_SEARCH, ICON_SEARCH},
    prelude::{GenerationalId, *},
};

mod data;
use data::*;

mod database;
use database::*;

mod panels;
use panels::*;

mod dialogs;
use dialogs::*;

mod views;
use views::*;

mod engine;
use engine::*;

mod menus;
use menus::*;

mod popup_menu;

#[derive(Debug, Error)]
#[error("App Error: ")]
pub enum AppError {
    ApplicationError(#[from] vizia::ApplicationError),
    IOError(#[from] std::io::Error),
    ImageError(#[from] image::ImageError),
}

fn main() -> Result<(), AppError> {
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

    // let icon = vizia::vg::Image::from_encoded(vizia::vg::Data::new_bytes(include_bytes!("../resources/icons/icon_256.png")));
    let icon = image::ImageReader::new(std::io::Cursor::new(include_bytes!("../resources/icons/icon_32.png"))).with_guessed_format()?.decode()?;

    Application::new(move |cx| {
        // Add resources
        cx.add_stylesheet(include_style!("resources/themes/style.css"))
            .expect("Failed to load stylesheet");

        cx.add_translation(
            langid!("en-GB"),
            include_str!("../resources/translations/en-GB/strings.ftl"),
        );

        cx.load_image("logo", include_bytes!("../resources/icons/icon_32.png"), ImageRetentionPolicy::Forever);

        cx.add_translation(langid!("es"), include_str!("../resources/translations/es/strings.ftl"));

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
            show_about_dialog: false,
            show_settings_dialog: false,
            show_add_collection_dialog: false,
            settings_data: SettingsData::dummy(),
        }
        .build(cx);

        cx.emit(AppEvent::LoadSample(String::from(
            "C:/Rust/vizia-sample-browser/the-libre-sample-pack/drums/one shot/kicks/couch kick 1 @TeaBoi.wav",
        )));

        about_dialog(cx);
        settings_dialog(cx, AppData::settings_data);

        VStack::new(cx, |cx|{
            HStack::new(cx, |cx|{
                menu_bar(cx);
            }).class("top-bar");
           
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
        });


    })
    .title("Vizia Sample Browser")
    .inner_size((1400, 800))
    .icon(icon.width(), icon.height(), icon.into_bytes())
    .run()?;

    Ok(())
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
