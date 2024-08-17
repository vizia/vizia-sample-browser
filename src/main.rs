#![allow(unused)] // Disable stupid warnings for now

use app_data::AppData;
use basedrop::Collector;
use cpal::traits::StreamTrait;
use itertools::Itertools;
use rusqlite::Connection;
use std::{
    collections::{HashMap, VecDeque},
    error::Error,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, Mutex},
};
use thiserror::Error;
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
    let icon = image::ImageReader::new(std::io::Cursor::new(include_bytes!(
        "../resources/icons/icon_32.png"
    )))
    .with_guessed_format()?
    .decode()?;

    let icon_clone = icon.clone();

    Application::new(move |cx| {
        // Add resources
        cx.add_stylesheet(include_style!("resources/themes/style.css"))
            .expect("Failed to load stylesheet");

        cx.add_translation(
            langid!("en-GB"),
            include_str!("../resources/translations/en-GB/strings.ftl"),
        );

        cx.load_image(
            "logo",
            include_bytes!("../resources/icons/icon_32.png"),
            ImageRetentionPolicy::Forever,
        );

        cx.add_translation(langid!("es"), include_str!("../resources/translations/es/strings.ftl"));

        cx.emit(EnvironmentEvent::SetThemeMode(AppTheme::BuiltIn(ThemeMode::DarkMode)));

        // Uncomment to test in Spanish
        // cx.emit(EnvironmentEvent::SetLocale(langid!("es")));

        AppData::new(collector, controller).build(cx);

        about_dialog(cx, icon_clone.clone());
        settings_dialog(cx, AppData::settings_data, icon_clone.clone());

        HStack::new(cx, |cx| {
            menu_bar(cx);
        })
        .class("top-bar");

        HStack::new(cx, |cx| {
            ResizableStack::new(
                cx,
                AppData::browser_width,
                ResizeStackDirection::Right,
                |cx, width| cx.emit(AppEvent::SetBrowserWidth(width)),
                |cx| {
                    ResizableStack::new(
                        cx,
                        AppData::browser_height,
                        ResizeStackDirection::Bottom,
                        |cx, height| cx.emit(AppEvent::SetBrowserHeight(height)),
                        |cx| {
                            BrowserPanel::new(cx);
                        },
                    )
                    .class("browser");
                    TagsPanel::new(cx);
                },
            )
            .class("side-bar");

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
                )
                .class("table");
                // Waveform Panel
                WavePanel::new(cx);
            })
            .row_between(Pixels(1.0));
        })
        .class("content")
        .col_between(Pixels(1.0))
        .size(Stretch(1.0));

        HStack::new(cx, |cx| {}).class("bottom-bar");
    })
    .title("Vizia Sample Browser")
    .inner_size((1400, 800))
    .icon(icon.width(), icon.height(), icon.into_bytes())
    .run()?;

    Ok(())
}
