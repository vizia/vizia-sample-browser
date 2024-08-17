use vizia::{icons::ICON_CHECK, prelude::*};

use crate::{data::AppData, AppEvent, SettingsEvent};

pub fn file_menu(cx: &mut Context) {
    Submenu::new(
        cx,
        |cx| Label::new(cx, Localized::new("file")),
        |cx| {
            MenuButton::new(
                cx,
                |cx| cx.emit(AppEvent::ShowAddCollectionDialog),
                |cx| {
                    HStack::new(cx, |cx| {
                        Label::new(cx, Localized::new("add-collection"));
                        Label::new(cx, "Ctrl + N").class("shortcut");
                    })
                },
            );
            MenuButton::new(
                cx,
                |cx| cx.emit(AppEvent::ShowOpenCollectionDialog),
                |cx| {
                    HStack::new(cx, |cx| {
                        Label::new(cx, Localized::new("open-collection"));
                        Label::new(cx, "Ctrl + O").class("shortcut");
                    })
                },
            );
            Submenu::new(
                cx,
                |cx| Label::new(cx, "Open Recent"),
                |cx| {
                    MenuButton::new(cx, |_| println!("Doc 1"), |cx| Label::new(cx, "Doc 1"));
                    Submenu::new(
                        cx,
                        |cx| Label::new(cx, "Doc 2"),
                        |cx| {
                            MenuButton::new(
                                cx,
                                |_| println!("Version 1"),
                                |cx| Label::new(cx, "Version 1"),
                            );
                            MenuButton::new(
                                cx,
                                |_| println!("Version 2"),
                                |cx| Label::new(cx, "Version 2"),
                            );
                            MenuButton::new(
                                cx,
                                |_| println!("Version 3"),
                                |cx| Label::new(cx, "Version 3"),
                            );
                        },
                    );
                    MenuButton::new(cx, |_| println!("Doc 3"), |cx| Label::new(cx, "Doc 3"));
                },
            );
            Divider::new(cx);
            MenuButton::new(
                cx,
                |cx| cx.emit(AppEvent::ShowSettingsDialog),
                |cx| Label::new(cx, Localized::new("settings")),
            );
            Divider::new(cx);
            MenuButton::new(cx, |_| println!("Exit"), |cx| Label::new(cx, Localized::new("quit")));
        },
    );
}

pub fn help_menu(cx: &mut Context) {
    Submenu::new(
        cx,
        |cx| Label::new(cx, Localized::new("help")),
        |cx| {
            MenuButton::new(
                cx,
                |cx| cx.emit(AppEvent::ShowAboutDialog),
                |cx| Label::new(cx, Localized::new("about")),
            );
        },
    );
}

pub fn edit_menu(cx: &mut Context) {
    Submenu::new(
        cx,
        |cx| Label::new(cx, "Edit"),
        |cx| {
            MenuButton::new(cx, |_| println!("Cut"), |cx| Label::new(cx, "Cut"));
            MenuButton::new(cx, |_| println!("Copy"), |cx| Label::new(cx, "Copy"));
            MenuButton::new(cx, |_| println!("Paste"), |cx| Label::new(cx, "Paste"));
        },
    );
}

pub fn view_menu(cx: &mut Context) {
    Submenu::new(
        cx,
        |cx| Label::new(cx, "View"),
        |cx| {
            MenuButton::new(cx, |_| println!("Zoom In"), |cx| Label::new(cx, "Zoom In"));
            MenuButton::new(cx, |_| println!("Zoom Out"), |cx| Label::new(cx, "Zoom Out"));
            Submenu::new(
                cx,
                |cx| Label::new(cx, "Zoom Level"),
                |cx| {
                    MenuButton::new(cx, |_| println!("10%"), |cx| Label::new(cx, "10%"));
                    MenuButton::new(cx, |_| println!("20%"), |cx| Label::new(cx, "20%"));
                    MenuButton::new(cx, |_| println!("50%"), |cx| Label::new(cx, "50%"));
                    MenuButton::new(cx, |_| println!("100%"), |cx| Label::new(cx, "100%"));
                    MenuButton::new(cx, |_| println!("150%"), |cx| Label::new(cx, "150%"));
                    MenuButton::new(cx, |_| println!("200%"), |cx| Label::new(cx, "200%"));
                },
            );
        },
    );
}

pub fn playback_menu(cx: &mut Context) {
    Submenu::new(
        cx,
        |cx| Label::new(cx, Localized::new("playback")),
        |cx| {
            MenuButton::new(cx, |_| {}, |cx| Label::new(cx, Localized::new("play-selected")));
            MenuButton::new(cx, |_| {}, |cx| Label::new(cx, Localized::new("stop")));
            Divider::new(cx);
            MenuButton::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_CHECK).visibility(AppData::should_loop);
                        Label::new(cx, Localized::new("toggle-loop"));
                    })
                },
            );
            Divider::new(cx);
            MenuButton::new(cx, |_| {}, |cx| Label::new(cx, Localized::new("toggle-autoplay")));
            Divider::new(cx);
            MenuButton::new(cx, |_| {}, |cx| Label::new(cx, Localized::new("prev-sample")));
            MenuButton::new(cx, |_| {}, |cx| Label::new(cx, Localized::new("next-sample")));
        },
    );
}

pub fn menu_bar(cx: &mut Context) {
    MenuBar::new(cx, |cx| {
        file_menu(cx);
        edit_menu(cx);
        playback_menu(cx);
        view_menu(cx);
        help_menu(cx);
    });
}
