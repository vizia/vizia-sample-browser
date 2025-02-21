use vizia::icons::{ICON_LIST_SEARCH, ICON_SEARCH};
use vizia::prelude::*;

use crate::app_data::AppData;
use crate::database::prelude::AudioFile;
use crate::menus::samples_panel_menu;
use crate::{SampleEvent, SamplesData};

pub struct SamplesPanel {}

impl SamplesPanel {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {}.build(cx, |cx| {
            HStack::new(cx, |cx| {
                HStack::new(cx, |cx| {
                    Textbox::new(cx, AppData::samples_data.then(SamplesData::search_text))
                        .class("icon-before")
                        .width(Stretch(1.0))
                        .class("search")
                        .placeholder(Localized::new("search"));
                    // .on_edit(|cx, text| cx.emit(AppDataSetter::EditableText(text)));
                    Svg::new(cx, ICON_SEARCH)
                        .class("icon")
                        .size(Pixels(20.0))
                        .position_type(PositionType::Absolute);
                })
                .height(Auto)
                .alignment(Alignment::Center)
                .width(Stretch(1.0));

                samples_panel_menu(cx);
            })
            .horizontal_gap(Pixels(8.0))
            .height(Auto)
            .class("header");

            VirtualTable::new(
                cx,
                AppData::samples_data.then(SamplesData::table_headers),
                AppData::samples_data.then(SamplesData::table_rows),
                30.0,
                |cx, _, item| {
                    Label::new(cx, item.map_ref(|(n, _)| n));
                },
                |cx, index, item| match index {
                    // Name
                    0 => {
                        Label::new(cx, item.then(AudioFile::name));
                    }
                    // Tags
                    1 => {}
                    // Duration
                    2 => {
                        Label::new(
                            cx,
                            item.then(AudioFile::duration).map(|duration| {
                                let d = Duration::from_secs_f32(*duration);
                                let secs = d.as_millis() / 1000;
                                let h = secs / (60 * 60);
                                let m = (secs / 60) % 60;
                                let s = secs % 60;
                                format!("{:0>2}:{:06.3}", m, duration)
                            }),
                        );
                    }
                    // Sample Rate
                    3 => {
                        Label::new(cx, item.then(AudioFile::sample_rate));
                    }
                    // Bit Depth
                    4 => {
                        Label::new(cx, item.then(AudioFile::bit_depth));
                    }
                    // Num Channels
                    5 => {
                        Label::new(cx, item.then(AudioFile::num_channels));
                    }
                    // BPM
                    6 => {
                        Label::new(
                            cx,
                            item.then(AudioFile::bpm)
                                .map(|k| k.map(|k| format!("{}", k)).unwrap_or(String::from("-"))),
                        );
                    }
                    // Key
                    7 => {
                        Label::new(
                            cx,
                            item.then(AudioFile::key)
                                .map(|k| k.map(|k| format!("{}", k)).unwrap_or(String::from("-"))),
                        );
                    }
                    // Size
                    8 => {
                        Label::new(cx, item.then(AudioFile::size));
                    }
                    _ => {}
                },
            )
            .selectable(Selectable::Single)
            .selected(AppData::samples_data.then(SamplesData::selected).map(|selected| {
                if let Some(selected) = selected {
                    vec![*selected]
                } else {
                    vec![]
                }
            }))
            .selection_follows_focus(true)
            .on_select(|cx, index| cx.emit(SampleEvent::Select(index)));
        })
    }
}

impl View for SamplesPanel {
    fn element(&self) -> Option<&'static str> {
        Some("samples-panel")
    }
}
