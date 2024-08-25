use vizia::icons::{ICON_LIST_SEARCH, ICON_SEARCH};
use vizia::prelude::*;

use crate::app_data::AppData;
use crate::database::prelude::AudioFile;
use crate::menus::samples_panel_menu;
use crate::views::SmartTable;
use crate::{SampleEvent, SamplesData};

pub struct SamplesPanel {}

impl SamplesPanel {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {}.build(cx, |cx| {
            Keymap::from(vec![
                (
                    KeyChord::new(Modifiers::empty(), Code::ArrowDown),
                    KeymapEntry::new((), |cx| cx.emit(SampleEvent::SelectNext)),
                ),
                (
                    KeyChord::new(Modifiers::empty(), Code::ArrowUp),
                    KeymapEntry::new((), |cx| cx.emit(SampleEvent::SelectPrev)),
                ),
            ])
            .build(cx);

            HStack::new(cx, |cx| {
                HStack::new(cx, |cx| {
                    Textbox::new(cx, AppData::search_text)
                        .class("icon-before")
                        .width(Stretch(1.0))
                        .class("search")
                        .placeholder(Localized::new("search"));
                    // .on_edit(|cx, text| cx.emit(AppDataSetter::EditableText(text)));
                    Svg::new(cx, ICON_SEARCH)
                        .class("icon")
                        .size(Pixels(20.0))
                        .position_type(PositionType::SelfDirected);
                })
                .height(Auto)
                .child_top(Stretch(1.0))
                .child_bottom(Stretch(1.0))
                .width(Stretch(1.0));

                samples_panel_menu(cx);
            })
            .col_between(Pixels(8.0))
            .height(Auto)
            .class("header");

            SmartTable::new(
                cx,
                AppData::samples_data.then(SamplesData::table_headers),
                AppData::samples_data.then(SamplesData::table_rows),
                |cx, row, col_index| {
                    match col_index {
                        // Name
                        0 => {
                            Label::new(cx, row.then(AudioFile::name))
                                .size(Stretch(1.0))
                                .child_left(if col_index == 0 {
                                    Pixels(4.0)
                                } else {
                                    Stretch(1.0)
                                });
                        }
                        // Tags
                        1 => {}
                        // Duration
                        2 => {
                            Label::new(
                                cx,
                                row.then(AudioFile::duration).map(|duration| {
                                    let d = Duration::from_secs_f32(*duration);
                                    let secs = d.as_millis() / 1000;
                                    let h = secs / (60 * 60);
                                    let m = (secs / 60) % 60;
                                    let s = secs % 60;
                                    format!("{:0>2}:{:06.3}", m, duration)
                                }),
                            )
                            .text_align(TextAlign::Center)
                            .size(Stretch(1.0))
                            .hoverable(false);
                        }
                        // Sample Rate
                        3 => {
                            Label::new(cx, row.then(AudioFile::sample_rate))
                                .text_align(TextAlign::Center)
                                .size(Stretch(1.0))
                                .hoverable(false);
                        }
                        // Bit Depth
                        4 => {
                            Label::new(cx, row.then(AudioFile::bit_depth))
                                .text_align(TextAlign::Center)
                                .size(Stretch(1.0))
                                .hoverable(false);
                        }
                        // Num Channels
                        5 => {
                            Label::new(cx, row.then(AudioFile::num_channels))
                                .text_align(TextAlign::Center)
                                .size(Stretch(1.0))
                                .hoverable(false);
                        }
                        // BPM
                        6 => {
                            Label::new(
                                cx,
                                row.then(AudioFile::bpm).map(|k| {
                                    k.map(|k| format!("{}", k)).unwrap_or(String::from("-"))
                                }),
                            )
                            .text_align(TextAlign::Center)
                            .hoverable(false)
                            .size(Stretch(1.0));
                        }
                        // Key
                        7 => {
                            Label::new(
                                cx,
                                row.then(AudioFile::key).map(|k| {
                                    k.map(|k| format!("{}", k)).unwrap_or(String::from("-"))
                                }),
                            )
                            .text_align(TextAlign::Center)
                            .hoverable(false)
                            .size(Stretch(1.0));
                        }
                        // Size
                        8 => {
                            Label::new(cx, row.then(AudioFile::size))
                                .text_align(TextAlign::Center)
                                .hoverable(false)
                                .size(Stretch(1.0));
                        }

                        _ => {}
                    }
                },
            );
        })
    }
}

impl View for SamplesPanel {
    fn element(&self) -> Option<&'static str> {
        Some("samples-panel")
    }
}
