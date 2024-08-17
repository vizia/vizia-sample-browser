use vizia::prelude::*;

use vizia::icons::{
    ICON_CHEVRON_DOWN, ICON_FILTER, ICON_FOLDER, ICON_FOLDER_FILLED, ICON_FOLDER_OPEN,
    ICON_LETTER_CASE, ICON_LIST, ICON_LIST_TREE, ICON_PLAYER_PLAY, ICON_PLAYER_SKIP_BACK,
    ICON_PLAYER_SKIP_FORWARD, ICON_PLAYER_STOP, ICON_SEARCH, ICON_TAG, ICON_WAVE_SINE,
};

use crate::app_data::AppData;
use crate::data::browser_data::{BrowserData, BrowserEvent};
use crate::data::AppEvent;
use crate::views::Waveview;

#[derive(Lens)]
pub struct WavePanel {
    // Todo - move this
    playing: bool,
}

impl WavePanel {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self { playing: false }.build(cx, |cx| {
            // Header
            HStack::new(cx, |cx| {
                // Panel Icon
                Svg::new(cx, ICON_WAVE_SINE).class("panel-icon");

                Label::new(cx, AppData::selected_file_name).right(Stretch(1.0));

                Chip::new(cx, AppData::selected_file_bit_depth.map(|bd| format!("{} bit", bd)));
                Chip::new(cx, AppData::selected_file_sample_rate.map(|bd| format!("{} Hz", bd)));
                Chip::new(
                    cx,
                    AppData::selected_file_num_channels.map(|bd| format!("{} channel", bd)),
                );
            })
            .class("header");

            // Waveform
            HStack::new(cx, |cx| {
                Waveview::new(cx, AppData::waveform, AppData::zoom_level, AppData::start);
            })
            .class("waveform");

            // Footer
            HStack::new(cx, |cx| {
                // toolbar here
                HStack::new(cx, |cx| {
                    Button::new(cx, |cx| Svg::new(cx, ICON_PLAYER_SKIP_BACK));
                    Button::new(cx, |cx| Svg::new(cx, ICON_PLAYER_PLAY))
                        .on_press(|cx| cx.emit(AppEvent::Play));
                    Button::new(cx, |cx| Svg::new(cx, ICON_PLAYER_STOP))
                        .on_press(|cx| cx.emit(AppEvent::Stop));
                    Button::new(cx, |cx| Svg::new(cx, ICON_PLAYER_SKIP_FORWARD));
                })
                .class("transport-controls");
            })
            .class("footer");
        })
    }
}

impl View for WavePanel {
    fn element(&self) -> Option<&'static str> {
        Some("wave-panel")
    }
}
