use image::DynamicImage;
use vizia::prelude::*;

use crate::data::{AppData, AppEvent};

pub fn about_dialog(cx: &mut Context, icon: DynamicImage) {
    Binding::new(cx, AppData::show_about_dialog, move |cx, show_about_dialog| {
        if show_about_dialog.get(cx) {
            Window::popup(cx, true, |cx| {
                VStack::new(cx, |cx| {
                    Svg::new(cx, *include_bytes!("../../resources/logo.svg")).size(Pixels(64.0));
                    Label::new(cx, "Name").class("title");
                    Label::new(cx, "Version 0.1").class("title");
                })
                .alignment(Alignment::Center);
            })
            .on_close(|cx| {
                cx.emit(AppEvent::HideAboutDialog);
            })
            .class("dialog")
            .title("About")
            .inner_size((200, 200))
            .resizable(false)
            .anchor(Anchor::Center)
            .enabled_window_buttons(WindowButtons::CLOSE)
            .icon(icon.width(), icon.height(), icon.clone().into_bytes());
        }
    });
}
