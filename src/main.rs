#![allow(unused)] // Disable stupid warnings for now

use app_data::AppData;
use basedrop::Collector;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use image::Pixels;
use itertools::Itertools;
use menus::menu_bar;
use rusqlite::Connection;
use std::{
    collections::{HashMap, VecDeque},
    error::Error,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, Mutex},
};
use thiserror::Error;

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

    let host = cpal::default_host();
    let output_device = host.default_output_device().expect("no output found");
    let config = output_device.default_output_config().expect("no default output config").config();

    let sample_rate = config.sample_rate.0 as f64;
    let num_channels = config.channels as usize;

    // Create the sample player and controller
    let (mut player, mut controller) = sample_player(&collector);

    let callback = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        let buffer_size = data.len() / num_channels;
        let context =
            PlaybackContext { buffer_size, num_channels, sample_rate, output_buffer: data };

        player.process(context);
    };

    let stream = output_device
        .build_output_stream(&config, callback, |err| eprintln!("{}", err), None)
        .expect("failed to open stream");

    stream.play();

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

        let timer =
            cx.add_timer(Duration::from_millis(10), None, |cx, action| cx.emit(AppEvent::Tick));

        // Uncomment to test in Spanish
        // cx.emit(EnvironmentEvent::SetLocale(langid!("es")));

        AppData::new(collector, controller, timer).build(cx);

        cx.emit(ConfigEvent::Load);

        Keymap::from(vec![
            (
                KeyChord::new(Modifiers::empty(), Code::Space),
                KeymapEntry::new((), |cx| cx.emit(AppEvent::Play)),
            ),
            (
                KeyChord::new(Modifiers::empty(), Code::Escape),
                KeymapEntry::new((), |cx| cx.emit(AppEvent::Stop)),
            ),
        ])
        .build(cx);

        about_dialog(cx, icon_clone.clone());
        settings_dialog(cx, AppData::settings_data, icon_clone.clone());

        HStack::new(cx, |cx| {
            menu_bar(cx);
        })
        .class("top-bar");

        HStack::new(cx, |cx| {
            Sidebar::new(cx);
            ResizableStack::new(
                cx,
                AppData::config.then(Config::browser_width).map(|w| Pixels(*w)),
                ResizeStackDirection::Right,
                |cx, width| cx.emit(ConfigEvent::SetBrowserWidth(width)),
                |cx| {
                    Binding::new(
                        cx,
                        AppData::config.then(Config::sidebar_view),
                        |cx, sidebar_view| match sidebar_view.get(cx) {
                            SidebarView::Browser => {
                                BrowserPanel::new(cx);
                            }
                            SidebarView::Tags => {
                                TagsPanel::new(cx);
                            }
                        },
                    );
                    // ResizableStack::new(
                    //     cx,
                    //     AppData::config.map(|config| {
                    //         if config.tags_visible {
                    //             Pixels(config.browser_height)
                    //         } else {
                    //             Stretch(1.0)
                    //         }
                    //     }),
                    //     ResizeStackDirection::Bottom,
                    //     |cx, height| cx.emit(ConfigEvent::SetBrowserHeight(height)),
                    //     |cx| {
                    //         BrowserPanel::new(cx);
                    //     },
                    // )
                    // .display(AppData::config.then(Config::browser_visible))
                    // .class("browser");
                    // TagsPanel::new(cx).display(AppData::config.then(Config::tags_visible));
                },
            )
            //.max_width(Pixels(20.0))
            //.display(AppData::config.then(Config::show_sidebar))
            .class("side-bar")
            .toggle_class("hidden", AppData::config.then(Config::show_sidebar).map(|b| !b));

            VStack::new(cx, |cx| {
                // Samples Panel
                ResizableStack::new(
                    cx,
                    AppData::config.map(|config| {
                        if config.waveview_visible {
                            Pixels(config.table_height)
                        } else {
                            Stretch(1.0)
                        }
                    }),
                    ResizeStackDirection::Bottom,
                    |cx, height| cx.emit(ConfigEvent::SetTableHeight(height)),
                    |cx| {
                        SamplesPanel::new(cx);
                    },
                )
                .class("table");
                // Waveform Panel
                WavePanel::new(cx).display(AppData::config.then(Config::waveview_visible));
            })
            .vertical_gap(Pixels(1.0));
        })
        .class("content")
        .horizontal_gap(Pixels(1.0))
        .size(Stretch(1.0));

        HStack::new(cx, |cx| {}).class("bottom-bar");
    })
    .title("Vizia Sample Browser")
    .inner_size(AppData::config.then(Config::window_size))
    .position(AppData::config.then(Config::window_position))
    .icon(icon.width(), icon.height(), icon.into_bytes())
    .run()?;

    Ok(())
}
