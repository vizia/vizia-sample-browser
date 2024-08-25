use std::sync::atomic::Ordering;

use vizia::prelude::*;

use vizia::icons::{
    ICON_CHEVRON_DOWN, ICON_FILTER, ICON_FOLDER, ICON_FOLDER_FILLED, ICON_FOLDER_OPEN,
    ICON_LETTER_CASE, ICON_LIST, ICON_LIST_TREE, ICON_MENU_2, ICON_PLAYER_PAUSE, ICON_PLAYER_PLAY,
    ICON_PLAYER_SKIP_BACK, ICON_PLAYER_SKIP_FORWARD, ICON_PLAYER_STOP, ICON_RELOAD, ICON_SEARCH,
    ICON_TAG, ICON_WAVE_SINE,
};

use crate::app_data::AppData;
use crate::data::browser_data::{BrowserData, BrowserEvent};
use crate::data::AppEvent;
use crate::menus::wave_panel_menu;
use crate::views::Waveview;
use crate::{ConfigEvent, PlayerState, SampleEvent, SamplePlayerController};

#[derive(Lens)]
pub struct WavePanel {}

impl WavePanel {
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

            // Header
            HStack::new(cx, |cx| {
                // Panel Icon
                Svg::new(cx, ICON_WAVE_SINE).class("panel-icon");

                Label::new(cx, AppData::selected_file_name).right(Stretch(1.0)).class("title");

                Chip::new(cx, AppData::selected_file_bit_depth.map(|bd| format!("{} bit", bd)));
                Chip::new(cx, AppData::selected_file_sample_rate.map(|bd| format!("{} Hz", bd)));
                Chip::new(
                    cx,
                    AppData::selected_file_num_channels.map(|bd| format!("{} channel", bd)),
                );

                wave_panel_menu(cx);
            })
            .class("header");

            // Waveform

            Waveview::new(
                cx,
                AppData::waveform,
                AppData::zoom_level,
                AppData::start,
                AppData::controller
                    .then(SamplePlayerController::playhead)
                    .map(|p| p.load(Ordering::SeqCst)),
            );

            // Footer
            HStack::new(cx, |cx| {
                // toolbar here
                HStack::new(cx, |cx| {
                    Button::new(cx, |cx| Svg::new(cx, ICON_PLAYER_SKIP_BACK))
                        .on_press(|cx| cx.emit(SampleEvent::SelectPrev));
                    ToggleButton::new(
                        cx,
                        AppData::controller
                            .then(SamplePlayerController::should_loop)
                            .map(|sl| sl.load(Ordering::SeqCst)),
                        |cx| Svg::new(cx, ICON_RELOAD),
                    )
                    .on_press(|cx| cx.emit(AppEvent::ToggleLooping));
                    ToggleButton::new(
                        cx,
                        AppData::controller
                            .then(SamplePlayerController::play_state)
                            .map(|ps| *ps == PlayerState::Playing),
                        |cx| {
                            Svg::new(
                                cx,
                                AppData::controller.then(SamplePlayerController::play_state).map(
                                    |ps| {
                                        if *ps == PlayerState::Stopped {
                                            ICON_PLAYER_PLAY
                                        } else {
                                            ICON_PLAYER_PAUSE
                                        }
                                    },
                                ),
                            )
                        },
                    )
                    .on_press(|cx| cx.emit(AppEvent::Play));
                    Button::new(cx, |cx| Svg::new(cx, ICON_PLAYER_STOP))
                        .on_press(|cx| cx.emit(AppEvent::Stop));
                    Button::new(cx, |cx| Svg::new(cx, ICON_PLAYER_SKIP_FORWARD))
                        .on_press(|cx| cx.emit(SampleEvent::SelectNext));
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

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, _| match window_event {
            WindowEvent::GeometryChanged(geo_changed)
                if geo_changed.contains(GeoChanged::WIDTH_CHANGED) =>
            {
                cx.emit(ConfigEvent::SetWaveviewWidth(cx.bounds().width()));
            }

            _ => {}
        })
    }
}
