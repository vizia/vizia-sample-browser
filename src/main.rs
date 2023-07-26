use app_data::AppData;
use views::smart_table::SmartTable;
use vizia::prelude::*;

mod state;
use state::*;

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

        AppData {
            browser: BrowserState::default(),
            browser_width: 300.0,
            table_height: 300.0,
            smart_table_data: vec![
                vec!["Col 1", "Col 2", "Col 3", "Col 4"]
                    .iter_mut()
                    .map(|v| v.to_string())
                    .collect(),
                vec!["data 1.1", "data 1.2", "data 1.3", "data 1.4"]
                    .iter_mut()
                    .map(|v| v.to_string())
                    .collect(),
                vec!["data 2.1", "data 2.2", "data 2.3", "data 2.4"]
                    .iter_mut()
                    .map(|v| v.to_string())
                    .collect(),
                vec!["data 3.1", "data 3.2", "data 3.3", "data 3.4"]
                    .iter_mut()
                    .map(|v| v.to_string())
                    .collect(),
            ],
        }
        .build(cx);

        HStack::new(cx, |cx| {
            // TODO: Place this in resizable stack
            ResizableStack::new(
                cx,
                AppData::browser_width,
                ResizeStackDirection::Right,
                |cx, width| cx.emit(AppEvent::SetBrowserWidth(width)),
                |cx| {
                    Browser::new(cx);
                },
            );
            VStack::new(cx, |cx| {
                // Table View
                ResizableStack::new(
                    cx,
                    AppData::table_height,
                    ResizeStackDirection::Bottom,
                    |cx, height| cx.emit(AppEvent::SetTableHeight(height)),
                    |cx| {
                        SmartTable::new(cx, AppData::smart_table_data);
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
    .run();
}
