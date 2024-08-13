use vizia::prelude::*;

use vizia::icons::{
    ICON_CHEVRON_DOWN, ICON_FILTER, ICON_FOLDER, ICON_FOLDER_FILLED, ICON_FOLDER_OPEN,
    ICON_LETTER_CASE, ICON_LIST, ICON_LIST_TREE, ICON_PLAYER_PLAY, ICON_PLAYER_SKIP_BACK,
    ICON_PLAYER_SKIP_FORWARD, ICON_PLAYER_STOP, ICON_SEARCH, ICON_TAG, ICON_WAVE_SINE,
};

use crate::app_data::AppData;
use crate::data::browser::{BrowserEvent, BrowserState};
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

                Label::new(cx, "Sample Name").right(Stretch(1.0));

                Chip::new(cx, "24 bit");
                Chip::new(cx, "44100 Hz");
                Chip::new(cx, "2 channels");
            })
            .class("header");

            // Waveform
            // ScrollView::new(cx, 0.0, 0.0, false, true, |cx| {}).class("waveform");
            HStack::new(cx, |cx| {
                Waveview::new(cx, AppData::waveform, AppData::zoom_level, AppData::start);
            })
            .class("waveform");

            // Footer
            HStack::new(cx, |cx| {
                // toolbar here
                ButtonGroup::new(cx, |cx| {
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
