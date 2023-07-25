use vizia::prelude::*;

mod state;
use state::*;

mod panels;
use panels::*;

mod views;
use views::*;

mod app_data;
use app_data::*;

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

        AppData { browser: BrowserState::default(), browser_width: 300.0 }.build(cx);

        HStack::new(cx, |cx| {
            // TODO: Place this in resizable stack
            ResizableStack::new(
                cx,
                AppData::browser_width,
                |cx, width| cx.emit(AppEvent::SetBrowserWidth(width)),
                |cx| {
                    Browser::new(cx);
                },
            );
            VStack::new(cx, |cx| {
                // Sample Player
                Element::new(cx).background_color(Color::from("#323232"));
                // Table View
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
