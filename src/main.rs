#![allow(unused)] // Disable stupid warnings for now

use app_data::AppData;
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

mod app_data;
use app_data::*;

mod popup_menu;

fn main() {
    Application::new(|cx| {
        // Add resources
        cx.add_stylesheet(include_style!("resources/themes/style.css"))
            .expect("Failed to load stylesheet");

        cx.add_translation(
            langid!("en-US"),
            include_str!("../resources/translations/en-US/browser.ftl"),
        );

        cx.add_translation(langid!("es"), include_str!("../resources/translations/es/browser.ftl"));

        cx.emit(EnvironmentEvent::SetThemeMode(ThemeMode::DarkMode));

        // Uncomment to test in Spanish
        // cx.emit(EnvironmentEvent::SetLocale(langid!("es")));

        let headers =
            vec!["Name", "Tags", "Duration", "Sample Rate", "Bit Depth", "BPM", "Key", "Size"]
                .iter_mut()
                .map(|v| v.to_string())
                .collect::<Vec<_>>();

        // let rows = (0..20)
        //     .map(|row| {
        //         vec![
        //             &format!("MSL_snare_{:02}", row),
        //             "?",
        //             "5.3 sec",
        //             "44100",
        //             "24",
        //             "?",
        //             "?",
        //             "2.5MB",
        //         ]
        //         .iter_mut()
        //         .map(|v| v.to_string())
        //         .collect::<Vec<_>>()
        //     })
        //     .collect::<Vec<_>>();

        let mut db = Database::from_directory(Path::new("test_files/").to_path_buf()).unwrap();

        let collections = db.get_all_collections().unwrap();
        let audio_files = db.get_all_audio_files().unwrap();
        let root = collections.iter().find(|v| v.parent_collection().is_none()).unwrap();

        let root = collections_to_directories(&collections, &audio_files, root.clone());

        let audio_files = db.get_all_audio_files().unwrap().len();
        println!("num: {}", audio_files);

        AppData {
            browser: BrowserState::new(root),
            browser_width: 300.0,
            table_height: 300.0,
            table_headers: headers,
            table_rows: Vec::new(),
            search_text: String::new(),
            //
            database: Arc::new(Mutex::new(db)),
        }
        .build(cx);

        HStack::new(cx, |cx| {
            ResizableStack::new(
                cx,
                AppData::browser_width,
                ResizeStackDirection::Right,
                |cx, width| cx.emit(AppEvent::SetBrowserWidth(width)),
                |cx| {
                    VStack::new(cx, |cx| {
                        BrowserPanel::new(cx);
                    })
                    .class("panel");
                },
            )
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
                // Sample Player
                Element::new(cx).background_color(Color::from("#323232"));
            })
            .row_between(Pixels(2.0));
        })
        .background_color(Color::from("#181818"))
        .col_between(Pixels(2.0))
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

    println!("{:?} {}", current, afs.len());

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
