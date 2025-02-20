use vizia::icons::{ICON_SETTINGS, ICON_TAG};
use vizia::prelude::*;
use vizia::{
    icons::{ICON_FOLDER_OPEN, ICON_SEARCH, ICON_SELECT_ALL},
    view::View,
};

use crate::data::{AppData, AppEvent, Config, ConfigEvent, SidebarView};

pub struct Sidebar {}

impl Sidebar {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {}.build(cx, |cx| {
            VStack::new(cx, |cx| {
                Button::new(cx, |cx| Svg::new(cx, ICON_FOLDER_OPEN))
                    .checked(
                        AppData::config
                            .then(Config::sidebar_view)
                            .map(|view| *view == SidebarView::Browser),
                    )
                    .on_press(|cx| cx.emit(ConfigEvent::ShowSidebarView(SidebarView::Browser)));

                Button::new(cx, |cx| Svg::new(cx, ICON_SEARCH));
                Button::new(cx, |cx| Svg::new(cx, ICON_TAG))
                    .checked(
                        AppData::config
                            .then(Config::sidebar_view)
                            .map(|view| *view == SidebarView::Tags),
                    )
                    .on_press(|cx| cx.emit(ConfigEvent::ShowSidebarView(SidebarView::Tags)));
                Spacer::new(cx);
                Button::new(cx, |cx| Svg::new(cx, ICON_SETTINGS))
                    .on_press(|cx| cx.emit(AppEvent::ShowSettingsDialog));
            });
        })
    }
}

impl View for Sidebar {
    fn element(&self) -> Option<&'static str> {
        Some("sidebar")
    }
}
