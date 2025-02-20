use std::str::FromStr;

use image::DynamicImage;
use strum::VariantNames;
use vizia::prelude::*;

use crate::data::{AppData, AppEvent, SettingsData, SettingsEvent, SettingsPage};

pub fn settings_dialog<L: Lens<Target = SettingsData>>(
    cx: &mut Context,
    lens: L,
    icon: DynamicImage,
) {
    Binding::new(cx, AppData::show_settings_dialog, move |cx, show_settings_dialog| {
        if show_settings_dialog.get(cx) {
            Window::popup(cx, true, move |cx| {
                HStack::new(cx, move |cx| {
                    VStack::new(cx, |cx| {
                        List::new(cx, StaticLens::new(&SettingsPage::VARIANTS), |cx, _, item| {
                            Button::new(cx, |cx| {
                                Label::new(cx, item.map(|key| Localized::new(key)))
                            })
                            .on_press_down(move |cx| {
                                cx.emit(match SettingsPage::from_str(item.get(cx)).unwrap() {
                                    SettingsPage::General => SettingsEvent::ShowGeneral,
                                    SettingsPage::UserInterface => SettingsEvent::ShowUserInterface,
                                    SettingsPage::Audio => SettingsEvent::ShowAudio,
                                })
                            });
                        });
                    });

                    Binding::new(
                        cx,
                        lens.then(SettingsData::selected_page),
                        move |cx, selected_page| match selected_page.get(cx) {
                            SettingsPage::General => {
                                ScrollView::new(cx, |cx| {
                                    //
                                })
                                .class("settings");
                            }

                            SettingsPage::UserInterface => {
                                ScrollView::new(cx, |cx| {
                                    //
                                })
                                .class("settings");
                            }

                            SettingsPage::Audio => {
                                ScrollView::new(cx, move |cx| {
                                    //
                                    HStack::new(cx, move |cx| {
                                        Label::new(cx, Localized::new("audio-driver"));
                                        PickList::new(
                                            cx,
                                            lens.then(SettingsData::audio_driver),
                                            lens.then(SettingsData::selected_audio_driver),
                                            true,
                                        )
                                        .width(Pixels(150.0));
                                    })
                                    .class("panel");

                                    HStack::new(cx, move |cx| {
                                        Label::new(cx, Localized::new("input-device"));
                                        PickList::new(
                                            cx,
                                            lens.then(SettingsData::input_device),
                                            lens.then(SettingsData::selected_input_device),
                                            true,
                                        )
                                        .width(Pixels(150.0));
                                    })
                                    .class("panel");

                                    HStack::new(cx, move |cx| {
                                        Label::new(cx, Localized::new("output-device"));
                                        PickList::new(
                                            cx,
                                            lens.then(SettingsData::output_device),
                                            lens.then(SettingsData::selected_output_device),
                                            true,
                                        )
                                        .width(Pixels(150.0));
                                    })
                                    .class("panel");
                                })
                                .class("settings");
                            }
                        },
                    );

                    // TabView::new(
                    //     cx,
                    //     StaticLens::new(&SettingsPage::VARIANTS),
                    //     move |cx, item| {
                    //         match item.get(cx) {
                    //             "general" => TabPair::new(
                    //                 move |cx| {
                    //                     Label::new(cx, item.map(|key| Localized::new(key)))
                    //                         .class("tab-name")
                    //                         .hoverable(false);
                    //                 },
                    //                 |cx| {
                    //                     ScrollView::new(cx, 0.0, 0.0, false, true, |cx| {
                    //                         //
                    //                     })
                    //                     .class("settings");
                    //                 },
                    //             ),

                    //             "user-interface" => TabPair::new(
                    //                 move |cx| {
                    //                     Label::new(cx, item.map(|key| Localized::new(key)))
                    //                         .class("tab-name")
                    //                         .hoverable(false);
                    //                 },
                    //                 |cx| {
                    //                     ScrollView::new(cx, 0.0, 0.0, false, true, |cx| {
                    //                         //
                    //                     })
                    //                     .class("settings");
                    //                 },
                    //             ),

                    //             "audio" => TabPair::new(
                    //                 move |cx| {
                    //                     Label::new(cx, item.map(|key| Localized::new(key)))
                    //                         .class("tab-name")
                    //                         .hoverable(false);
                    //                 },
                    //                 move |cx| {
                    //                     ScrollView::new(cx, 0.0, 0.0, false, true, move |cx| {
                    //                         //
                    //                         HStack::new(cx, move |cx| {
                    //                             Label::new(cx, Localized::new("audio-driver"));
                    //                             PickList::new(
                    //                                 cx,
                    //                                 lens.then(SettingsData::audio_driver),
                    //                                 lens.then(
                    //                                     SettingsData::selected_audio_driver,
                    //                                 ),
                    //                                 true,
                    //                             )
                    //                             .width(Pixels(150.0));
                    //                         })
                    //                         .class("panel");

                    //                         HStack::new(cx, move |cx| {
                    //                             Label::new(cx, Localized::new("input-device"));
                    //                             PickList::new(
                    //                                 cx,
                    //                                 lens.then(SettingsData::input_device),
                    //                                 lens.then(
                    //                                     SettingsData::selected_input_device,
                    //                                 ),
                    //                                 true,
                    //                             )
                    //                             .width(Pixels(150.0));
                    //                         })
                    //                         .class("panel");

                    //                         HStack::new(cx, move |cx| {
                    //                             Label::new(cx, Localized::new("output-device"));
                    //                             PickList::new(
                    //                                 cx,
                    //                                 lens.then(SettingsData::output_device),
                    //                                 lens.then(
                    //                                     SettingsData::selected_output_device,
                    //                                 ),
                    //                                 true,
                    //                             )
                    //                             .width(Pixels(150.0));
                    //                         })
                    //                         .class("panel");
                    //                     })
                    //                     .class("settings");
                    //                 },
                    //             ),

                    //             _ => TabPair::new(|_| {}, |_| {}),
                    //         }
                    //     },
                    // )
                    // .class("settings")
                    // .vertical();
                });
            })
            .on_close(|cx| {
                cx.emit(AppEvent::HideSettingsDialog);
            })
            .anchor(Anchor::Center)
            .class("dialog")
            .title("Settings")
            .inner_size((800, 600))
            .icon(icon.width(), icon.height(), icon.clone().into_bytes());
        }
    });
}
