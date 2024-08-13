use vizia::icons::{ICON_LIST_SEARCH, ICON_SEARCH};
use vizia::prelude::*;

use crate::app_data::AppData;
use crate::database::prelude::AudioFile;
use crate::views::SmartTable;

pub struct SamplesPanel {}

impl SamplesPanel {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {}.build(cx, |cx| {
            HStack::new(cx, |cx| {
                Svg::new(cx, ICON_LIST_SEARCH).class("panel-icon");

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
            })
            .col_between(Pixels(8.0))
            .height(Auto)
            .class("header");

            SmartTable::new(
                cx,
                AppData::table_headers,
                AppData::table_rows,
                |cx, row, col_index| {
                    match col_index {
                        // Name
                        0 => {
                            Label::new(cx, row.then(AudioFile::name))
                                .size(Stretch(1.0))
                                .child_space(Stretch(1.0))
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
                            Label::new(cx, row.then(AudioFile::duration))
                                .child_space(Stretch(1.0))
                                .size(Stretch(1.0));
                        }
                        // Sample Rate
                        3 => {
                            Label::new(cx, row.then(AudioFile::sample_rate))
                                .child_space(Stretch(1.0))
                                .size(Stretch(1.0));
                        }
                        // Bit Depth
                        4 => {
                            Label::new(cx, row.then(AudioFile::bit_depth))
                                .child_space(Stretch(1.0))
                                .size(Stretch(1.0));
                        }
                        // BPM
                        5 => {
                            Label::new(cx, row.then(AudioFile::bpm).map(|k| format!("{:?}", k)))
                                .child_space(Stretch(1.0))
                                .size(Stretch(1.0));
                        }
                        // Key
                        6 => {
                            Label::new(cx, row.then(AudioFile::key).map(|k| format!("{:?}", k)))
                                .child_space(Stretch(1.0))
                                .size(Stretch(1.0));
                        }
                        // Size
                        _ => {
                            Label::new(cx, row.then(AudioFile::size))
                                .child_space(Stretch(1.0))
                                .size(Stretch(1.0));
                        }
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
