use vizia::prelude::*;

use vizia::icons::{
    ICON_CHEVRON_DOWN, ICON_FILTER, ICON_FOLDER, ICON_FOLDER_FILLED, ICON_FOLDER_OPEN,
    ICON_LETTER_CASE, ICON_LIST, ICON_LIST_TREE, ICON_SEARCH, ICON_TAG, ICON_WAVE_SINE,
};

use crate::app_data::AppData;
use crate::state::browser::{BrowserEvent, BrowserState};

#[derive(Lens)]
pub struct WavePanel {}

impl WavePanel {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {}.build(cx, |cx| {
            // Header
            HStack::new(cx, |cx| {
                // Panel Icon
                Icon::new(cx, ICON_WAVE_SINE).class("panel-icon");

                Label::new(cx, "Sample Name");
            })
            .class("header");

            // Waveform
            ScrollView::new(cx, 0.0, 0.0, false, true, |cx| {}).class("waveform");

            // Footer
            HStack::new(cx, |cx| {
                Label::new(cx, "24 bit");
                Label::new(cx, "44100 Hz");
                Label::new(cx, "2 channels");
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
