use std::sync::atomic::Ordering;

use vizia::{
    icons::{
        ICON_ARROW_BACK_UP, ICON_ARROW_FORWARD_UP, ICON_CHECK, ICON_FILE_DATABASE, ICON_FOLDER,
        ICON_MENU_2, ICON_PLAYER_PLAY, ICON_PLAYER_SKIP_BACK, ICON_PLAYER_SKIP_FORWARD,
        ICON_PLAYER_STOP, ICON_SECTION_SIGN, ICON_SETTINGS,
    },
    prelude::*,
};

use crate::{
    data::AppData, data::SidebarView, AppEvent, Config, ConfigEvent, SampleEvent,
    SamplePlayerController, SamplesData, SettingsEvent,
};

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
                        Svg::new(cx, ICON_FILE_DATABASE).class("icon");
                        Label::new(cx, Localized::new("add-collection"));
                        Spacer::new(cx);
                        Label::new(cx, "Ctrl+N").class("shortcut");
                    })
                    .width(Stretch(1.0))
                    .min_width(Auto)
                },
            );
            MenuButton::new(
                cx,
                |cx| cx.emit(AppEvent::ShowOpenCollectionDialog),
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_FOLDER).class("icon");
                        Label::new(cx, Localized::new("open-collection"));
                        Spacer::new(cx);
                        Label::new(cx, "Ctrl+O").class("shortcut");
                    })
                },
            );
            Submenu::new(
                cx,
                |cx| {
                    HStack::new(cx, |cx| {
                        Element::new(cx).class("icon");
                        Label::new(cx, Localized::new("open-recent"));
                    })
                    .width(Auto)
                },
                |cx| {
                    // List::new(cx, AppData::config.then(Config::recents), |cx, index, recent| {
                    //     MenuButton::new(
                    //         cx,
                    //         |_| {},
                    //         move |cx| {
                    //             HStack::new(cx, |cx| {
                    //                 Element::new(cx).class("icon");
                    //                 Label::new(
                    //                     cx,
                    //                     recent.map(|path| path.to_str().unwrap().to_owned()),
                    //                 );
                    //             })
                    //         },
                    //     );
                    // });
                    // MenuDivider::new(cx).display(
                    //     AppData::config.then(Config::recents).map(|recents| !recents.is_empty()),
                    // );
                    MenuButton::new(
                        cx,
                        |_| {},
                        |cx| {
                            HStack::new(cx, |cx| {
                                Element::new(cx).class("icon");
                                Label::new(cx, Localized::new("clear-recents"));
                            })
                        },
                    );
                },
            );
            MenuDivider::new(cx);
            MenuButton::new(
                cx,
                |cx| cx.emit(AppEvent::ShowSettingsDialog),
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_SETTINGS).class("icon");
                        Label::new(cx, Localized::new("settings"));
                        Spacer::new(cx);
                        Label::new(cx, "Ctrl+,").class("shortcut");
                    })
                },
            );
            MenuDivider::new(cx);
            MenuButton::new(
                cx,
                |cx| cx.emit(WindowEvent::WindowClose),
                |cx| {
                    HStack::new(cx, |cx| {
                        Element::new(cx).class("icon");
                        Label::new(cx, Localized::new("quit"));
                        Spacer::new(cx);
                        Label::new(cx, "Ctrl+Q").class("shortcut");
                    })
                },
            );
        },
    );
}

pub fn edit_menu(cx: &mut Context) {
    Submenu::new(
        cx,
        |cx| Label::new(cx, "Edit"),
        |cx| {
            MenuButton::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_ARROW_BACK_UP).class("icon");
                        Label::new(cx, Localized::new("undo"));
                        Spacer::new(cx);
                        Label::new(cx, "Ctrl+Z").class("shortcut");
                    })
                },
            );
            MenuButton::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_ARROW_FORWARD_UP).class("icon");
                        Label::new(cx, Localized::new("redo"));
                        Spacer::new(cx);
                        Label::new(cx, "Ctrl+Y").class("shortcut");
                    })
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
            MenuButton::new(
                cx,
                |cx| cx.emit(AppEvent::Play),
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_PLAYER_PLAY).class("icon");
                        Label::new(cx, Localized::new("play-selected"));
                        Spacer::new(cx);
                        Label::new(cx, "Space").class("shortcut");
                    })
                },
            );
            MenuButton::new(
                cx,
                |cx| cx.emit(AppEvent::Stop),
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_PLAYER_STOP).class("icon");
                        Label::new(cx, Localized::new("stop"));
                        Spacer::new(cx);
                        Label::new(cx, "Esc").class("shortcut");
                    })
                },
            );
            MenuDivider::new(cx);
            MenuButton::new(
                cx,
                |cx| cx.emit(AppEvent::ToggleLooping),
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_CHECK)
                            .visibility(
                                AppData::controller
                                    .then(SamplePlayerController::should_loop)
                                    .map(|sl| sl.load(Ordering::SeqCst)),
                            )
                            .class("icon");
                        Label::new(cx, Localized::new("toggle-loop"));
                        Spacer::new(cx);
                        Label::new(cx, "Ctrl+L").class("shortcut");
                    })
                },
            );
            MenuDivider::new(cx);
            MenuButton::new(
                cx,
                |cx| cx.emit(AppEvent::ToggleAutoplay),
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_CHECK).visibility(AppData::should_autoplay).class("icon");
                        Label::new(cx, Localized::new("toggle-autoplay"));
                        Spacer::new(cx);
                        Label::new(cx, "").class("shortcut");
                    })
                },
            );
            MenuDivider::new(cx);
            MenuButton::new(
                cx,
                |cx| cx.emit(SampleEvent::SelectPrev),
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_PLAYER_SKIP_BACK).class("icon");
                        Label::new(cx, Localized::new("prev-sample"));
                        Spacer::new(cx);
                        Label::new(cx, "").class("shortcut");
                    })
                },
            );
            MenuButton::new(
                cx,
                |cx| cx.emit(SampleEvent::SelectNext),
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_PLAYER_SKIP_FORWARD).class("icon");
                        Label::new(cx, Localized::new("next-sample"));
                        Spacer::new(cx);
                        Label::new(cx, "").class("shortcut");
                    })
                },
            );
        },
    );
}

pub fn view_menu(cx: &mut Context) {
    Submenu::new(
        cx,
        |cx| Label::new(cx, "View"),
        |cx| {
            MenuButton::new(
                cx,
                |cx| cx.emit(ConfigEvent::ShowSidebarView(SidebarView::Browser)),
                |cx| {
                    HStack::new(cx, |cx| {
                        Element::new(cx).class("icon");
                        Label::new(cx, Localized::new("show-collections"));
                        Spacer::new(cx);
                        Label::new(cx, "Ctrl+Shift+E").class("shortcut");
                    })
                },
            );
            MenuButton::new(
                cx,
                |cx| cx.emit(ConfigEvent::ShowSidebarView(SidebarView::Tags)),
                |cx| {
                    HStack::new(cx, |cx| {
                        Element::new(cx).class("icon");
                        Label::new(cx, Localized::new("show-tags"));
                        Spacer::new(cx);
                        Label::new(cx, "Ctrl+Shift+T").class("shortcut");
                    })
                },
            );
            MenuButton::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_CHECK)
                            .visibility(AppData::config.then(Config::waveview_visible))
                            .class("icon");
                        Label::new(cx, Localized::new("show-waveview"));
                        Spacer::new(cx);
                        Label::new(cx, "Ctrl+Shift+W").class("shortcut");
                    })
                },
            )
            .on_press(|cx| cx.emit(ConfigEvent::ToggleWaveviewVisibility));

            MenuDivider::new(cx);

            columns_menu(cx);

            MenuDivider::new(cx);

            MenuButton::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Element::new(cx).class("icon");
                        Label::new(cx, Localized::new("toggle-fullscreen"));
                        Spacer::new(cx);
                        Label::new(cx, "F11").class("shortcut");
                    })
                },
            );
            MenuButton::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Element::new(cx).class("icon");
                        Label::new(cx, Localized::new("reset-view"));
                        Spacer::new(cx);
                        Label::new(cx, "").class("shortcut");
                    })
                },
            );
        },
    );
}

pub fn columns_menu(cx: &mut Context) {
    Submenu::new(
        cx,
        |cx| {
            HStack::new(cx, |cx| {
                Element::new(cx).class("icon");
                Label::new(cx, Localized::new("columns"));
            })
            .width(Auto)
        },
        |cx| {
            MenuButton::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Element::new(cx).class("icon");
                        Label::new(cx, Localized::new("restore-default-columns"));
                        Spacer::new(cx);
                        Label::new(cx, "").class("shortcut");
                    })
                },
            );
            MenuButton::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Element::new(cx).class("icon");
                        Label::new(cx, Localized::new("show-all-columns"));
                        Spacer::new(cx);
                        Label::new(cx, "").class("shortcut");
                    })
                },
            );
            MenuDivider::new(cx);
            MenuButton::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_CHECK)
                            .visibility(
                                AppData::samples_data
                                    .then(SamplesData::table_headers)
                                    .map(|headers| headers[0].1),
                            )
                            .class("icon");
                        Label::new(cx, Localized::new("name"));
                        Spacer::new(cx);
                        Label::new(cx, "").class("shortcut");
                    })
                },
            )
            .on_press(|cx| cx.emit(SampleEvent::ToggleColumn(0)));
            MenuButton::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_CHECK)
                            .visibility(
                                AppData::samples_data
                                    .then(SamplesData::table_headers)
                                    .map(|headers| headers[1].1),
                            )
                            .class("icon");
                        Label::new(cx, Localized::new("tags"));
                        Spacer::new(cx);
                        Label::new(cx, "").class("shortcut");
                    })
                },
            )
            .on_press(|cx| cx.emit(SampleEvent::ToggleColumn(1)));
            MenuButton::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_CHECK)
                            .visibility(
                                AppData::samples_data
                                    .then(SamplesData::table_headers)
                                    .map(|headers| headers[2].1),
                            )
                            .class("icon");
                        Label::new(cx, Localized::new("duration"));
                        Spacer::new(cx);
                        Label::new(cx, "").class("shortcut");
                    })
                },
            )
            .on_press(|cx| cx.emit(SampleEvent::ToggleColumn(2)));
            MenuButton::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_CHECK)
                            .visibility(
                                AppData::samples_data
                                    .then(SamplesData::table_headers)
                                    .map(|headers| headers[3].1),
                            )
                            .class("icon");
                        Label::new(cx, Localized::new("sample-rate"));
                        Spacer::new(cx);
                        Label::new(cx, "").class("shortcut");
                    })
                },
            )
            .on_press(|cx| cx.emit(SampleEvent::ToggleColumn(3)));
            MenuButton::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_CHECK)
                            .visibility(
                                AppData::samples_data
                                    .then(SamplesData::table_headers)
                                    .map(|headers| headers[4].1),
                            )
                            .class("icon");
                        Label::new(cx, Localized::new("bit-depth"));
                        Spacer::new(cx);
                        Label::new(cx, "").class("shortcut");
                    })
                },
            )
            .on_press(|cx| cx.emit(SampleEvent::ToggleColumn(4)));
            MenuButton::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_CHECK)
                            .visibility(
                                AppData::samples_data
                                    .then(SamplesData::table_headers)
                                    .map(|headers| headers[5].1),
                            )
                            .class("icon");
                        Label::new(cx, Localized::new("num-channels"));
                        Spacer::new(cx);
                        Label::new(cx, "").class("shortcut");
                    })
                },
            )
            .on_press(|cx| cx.emit(SampleEvent::ToggleColumn(5)));
            MenuButton::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_CHECK)
                            .visibility(
                                AppData::samples_data
                                    .then(SamplesData::table_headers)
                                    .map(|headers| headers[6].1),
                            )
                            .class("icon");
                        Label::new(cx, Localized::new("bpm"));
                        Spacer::new(cx);
                        Label::new(cx, "").class("shortcut");
                    })
                },
            )
            .on_press(|cx| cx.emit(SampleEvent::ToggleColumn(6)));
            MenuButton::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_CHECK)
                            .visibility(
                                AppData::samples_data
                                    .then(SamplesData::table_headers)
                                    .map(|headers| headers[7].1),
                            )
                            .class("icon");
                        Label::new(cx, Localized::new("key"));
                        Spacer::new(cx);
                        Label::new(cx, "").class("shortcut");
                    })
                },
            )
            .on_press(|cx| cx.emit(SampleEvent::ToggleColumn(7)));
            MenuButton::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_CHECK).class("icon");
                        Label::new(cx, Localized::new("size"));
                        Spacer::new(cx);
                        Label::new(cx, "").class("shortcut");
                    })
                },
            )
            .on_press(|cx| cx.emit(SampleEvent::ToggleColumn(8)));
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
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Element::new(cx).class("icon");
                        Label::new(cx, Localized::new("show-logs"));
                        Spacer::new(cx);
                        Label::new(cx, "").class("shortcut");
                    })
                },
            );
            MenuDivider::new(cx);
            MenuButton::new(
                cx,
                |cx| cx.emit(AppEvent::ShowAboutDialog),
                |cx| {
                    HStack::new(cx, |cx| {
                        Element::new(cx).class("icon");
                        Label::new(cx, Localized::new("about"));
                        Spacer::new(cx);
                        Label::new(cx, "").class("shortcut");
                    })
                },
            );
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

pub fn wave_panel_menu(cx: &mut Context) {
    Submenu::new(
        cx,
        |cx| Svg::new(cx, ICON_MENU_2),
        |cx| {
            Submenu::new(
                cx,
                |cx| {
                    HStack::new(cx, |cx| {
                        Element::new(cx).class("icon");
                        Label::new(cx, Localized::new("display-mode"));
                    })
                },
                |cx| {
                    MenuButton::new(
                        cx,
                        |_| {},
                        |cx| {
                            HStack::new(cx, |cx| {
                                Svg::new(cx, ICON_CHECK).class("icon");
                                Label::new(cx, Localized::new("linear"));
                                Label::new(cx, "").class("shortcut");
                            })
                        },
                    );
                    MenuButton::new(
                        cx,
                        |_| {},
                        |cx| {
                            HStack::new(cx, |cx| {
                                Svg::new(cx, ICON_CHECK).class("icon");
                                Label::new(cx, Localized::new("decibel"));
                                Label::new(cx, "").class("shortcut");
                            })
                        },
                    );
                },
            );
        },
    )
    .class("panel-menu");
}

pub fn tags_panel_menu(cx: &mut Context) {
    Submenu::new(
        cx,
        |cx| Svg::new(cx, ICON_MENU_2),
        |cx| {
            MenuButton::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Label::new(cx, Localized::new("add-tag"));
                        Label::new(cx, "Ctrl + N").class("shortcut");
                    })
                },
            );
        },
    )
    .class("panel-menu");
}

pub fn collections_panel_menu(cx: &mut Context) {
    Submenu::new(
        cx,
        |cx| Svg::new(cx, ICON_MENU_2),
        |cx| {
            MenuButton::new(
                cx,
                |cx| cx.emit(AppEvent::ShowAddCollectionDialog),
                |cx| {
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_FILE_DATABASE).class("icon");
                        Label::new(cx, Localized::new("add-collection"));
                        Label::new(cx, "Ctrl+N").class("shortcut");
                    })
                },
            );
        },
    )
    .class("panel-menu");
}

pub fn samples_panel_menu(cx: &mut Context) {
    Submenu::new(
        cx,
        |cx| Svg::new(cx, ICON_MENU_2),
        |cx| {
            columns_menu(cx);
        },
    )
    .class("panel-menu");
}
