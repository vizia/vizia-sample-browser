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

        let root = collections_to_directories(&mut db.get_all_collections().unwrap());

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

#[derive(Clone)]
struct RecursiveInner {
    id: CollectionID,
    parent_id: Option<CollectionID>,
    name: String,
    path: PathBuf,
    children: Vec<Rc<Mutex<RecursiveInner>>>,
}

impl RecursiveInner {
    fn to_directory(inner: &mut RecursiveInner) -> Directory {
        let children = inner
            .children
            .iter_mut()
            .map(|child| RecursiveInner::to_directory(&mut child.lock().unwrap()))
            .collect();

        Directory {
            id: inner.id,
            parent_id: inner.parent_id,
            name: inner.name.clone(),
            path: inner.path.clone(),
            children,
            is_open: false,
            shown: true,
            ..Default::default()
        }
    }
}

fn collections_to_directories(collections: &mut Vec<Collection>) -> Directory {
    let mut hm: HashMap<CollectionID, Rc<Mutex<RecursiveInner>>> = HashMap::new();

    for coll in collections {
        hm.insert(
            coll.id(),
            Rc::new(Mutex::new(RecursiveInner {
                id: coll.id(),
                parent_id: coll.parent_collection(),
                name: coll.name().to_string(),
                path: coll.path().clone(),
                children: Vec::new(),
            })),
        );
    }

    fn children_of_collection(
        map: &HashMap<CollectionID, Rc<Mutex<RecursiveInner>>>,
        coll: CollectionID,
    ) -> VecDeque<CollectionID> {
        map.values()
            .filter(|v| v.lock().unwrap().parent_id == Some(coll))
            .map(|v| v.lock().unwrap().id)
            .collect()
    }

    let mut root_dir = hm.values().find(|v| v.lock().unwrap().parent_id.is_none()).unwrap();
    let mut directory_stack: VecDeque<Rc<Mutex<RecursiveInner>>> = VecDeque::new();
    directory_stack.push_back(root_dir.clone());

    while let Some(mut coll) = directory_stack.pop_front() {
        let id: usize = coll.lock().unwrap().id;
        let mut children = children_of_collection(&hm, id);
        let mut children_dir: VecDeque<Rc<Mutex<RecursiveInner>>> = VecDeque::new();
        children.iter_mut().for_each(|v| children_dir.push_back(hm.get(&v).unwrap().clone()));

        for mut child_ref in children_dir {
            let mut child = child_ref.lock().unwrap();

            // Each child inside the current focused directory appends to the recursive structure
            coll.lock().unwrap().children.push(child_ref.clone());

            // Reference each of those children to iterate in the stack
            directory_stack.push_back(child_ref.clone());
        }
    }

    // Transform root dir to Directory
    let mut root_directory = root_dir.lock().unwrap().clone();
    let directory = RecursiveInner::to_directory(&mut root_directory);

    directory
}
