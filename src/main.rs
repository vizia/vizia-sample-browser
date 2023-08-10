#![allow(unused)] // Disable stupid warnings for now

use app_data::AppData;
use rusqlite::Connection;
use views::smart_table::SmartTable;
use vizia::{
    icons::{ICON_LIST_SEARCH, ICON_SEARCH},
    prelude::*,
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
    let database_handle = Database::from_connection("", Some(Connection::open(".vsb").unwrap()));

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

        let rows = (0..20)
            .map(|row| {
                vec![
                    &format!("MSL_snare_{:02}", row),
                    "?",
                    "5.3 sec",
                    "44100",
                    "24",
                    "?",
                    "?",
                    "2.5MB",
                ]
                .iter_mut()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        AppData {
            browser: BrowserState::default(),
            browser_width: 300.0,
            table_height: 300.0,
            table_headers: headers,
            table_rows: rows,
            search_text: String::new(),
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
