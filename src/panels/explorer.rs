use std::path::Path;
use std::rc::Rc;

use vizia::icons::ICON_MENU_2;
use vizia::{prelude::*, ICON_CHEVRON_DOWN};

use crate::app_data::AppData;
use crate::state::browser::file_derived_lenses::children;
use crate::state::browser::*;

#[derive(Lens)]
pub struct Explorer {
    search_text: String,
}

impl Explorer {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self { search_text: String::new() }.build(cx, |cx| {
            cx.emit(BrowserEvent::ViewAll);

            HStack::new(cx, |cx| {
                Textbox::new(cx, Explorer::search_text)
                    .placeholder(Localized::new("search"))
                    .width(Stretch(1.0));
                // Menu button
                Button::new(
                    cx,
                    |_cx| {
                        // Open burger menu
                    },
                    |cx| Icon::new(cx, ICON_MENU_2).size(Stretch(1.0)),
                )
                .class("menu-button");
            })
            .col_between(Pixels(8.0))
            .height(Auto);

            // treeview(cx, AppData::browser.then(BrowserState::root_file), 0, directory_header, file);

            treeview(
                cx,
                AppData::browser.then(BrowserState::root_file),
                0,
                directory_header,
                |cx, item, level| {
                    treeview(cx, item, level, directory_header, |cx, item, level| {
                        treeview(cx, item, level, directory_header, |cx, item, level| {
                            treeview(cx, item, level, directory_header, |cx, item, level| {
                                treeview(cx, item, level, directory_header, file);
                            });
                        });
                    });
                },
            );
        })
    }
}

impl View for Explorer {
    fn element(&self) -> Option<&'static str> {
        Some("explorer")
    }
}

fn directory_header<L>(cx: &mut Context, lens: L, level: u32)
where
    L: Lens<Target = File>,
{
    Binding::new(cx, lens.then(File::is_dir), move |cx, is_dir| {
        if is_dir.get(cx) {
            directory(cx, lens, level);
        } else {
            file(cx, lens, level);
        }
    });
}

fn directory<L>(cx: &mut Context, root: L, level: u32)
where
    L: Lens<Target = File>,
{
    Binding::new(cx, root.then(File::file_path), move |cx, file_path| {
        let file_path1 = file_path.get(cx);
        let file_path2 = file_path.get(cx);
        let file_path3 = file_path.get(cx);
        HStack::new(cx, |cx| {
            //Icon::new(cx, IconCode::Dropdown, 24.0, 23.0)
            // Arrow Icon
            Label::new(cx, ICON_CHEVRON_DOWN)
                .class("icon")
                .height(Stretch(1.0))
                .width(Pixels(16.0))
                .child_space(Stretch(1.0))
                .hoverable(false)
                .visibility(root.then(File::children).map(|c| !c.is_empty()))
                .rotate(root.then(File::is_open).map(|flag| {
                    if *flag {
                        Angle::Deg(0.0)
                    } else {
                        Angle::Deg(-90.0)
                    }
                }));
            // File or directory name
            Label::new(cx, root.then(File::name))
                .width(Stretch(1.0))
                .text_wrap(false)
                .hoverable(false);
        })
        .cursor(CursorIcon::Hand)
        .class("dir-file")
        .toggle_class(
            "focused",
            AppData::browser.then(BrowserState::selected).map(move |selected| {
                match (&file_path1, selected) {
                    (Some(fp), Some(s)) => s.starts_with(fp),

                    _ => false,
                }
            }),
        )
        .toggle_class(
            "selected",
            AppData::browser
                .then(BrowserState::selected)
                .map(move |selected| &file_path2 == selected),
        )
        .on_press(move |cx| {
            println!("press");
            cx.focus();
            if let Some(file_path) = &file_path3 {
                cx.emit(BrowserEvent::SetSelected(file_path.clone()));
                cx.emit(BrowserEvent::ToggleOpen);
            }
        })
        .col_between(Pixels(4.0))
        .child_left(Pixels(16.0 * level as f32 + 8.0));
    });
}

fn file<L>(cx: &mut Context, item: L, level: u32)
where
    L: Lens<Target = File>,
{
    Binding::new(cx, item.then(File::file_path), move |cx, file_path| {
        let file_path1 = file_path.get(cx);
        let file_path2 = file_path.get(cx);
        let file_path3 = file_path.get(cx);
        let is_selected = AppData::browser
            .then(BrowserState::selected)
            .map(move |selected| &file_path2 == selected);
        Label::new(cx, item.then(File::name))
            .class("dir-file")
            .width(Stretch(1.0))
            .text_wrap(false)
            .cursor(CursorIcon::Hand)
            .child_left(Pixels(15.0 * level as f32 + 5.0))
            .toggle_class(
                "focused",
                AppData::browser.then(BrowserState::selected).map(move |selected| {
                    match (&file_path1, selected) {
                        (Some(fp), Some(s)) => s.starts_with(fp),
                        _ => false,
                    }
                }),
            )
            .toggle_class("selected", is_selected)
            .on_press(move |cx| {
                cx.focus();
                if let Some(file_path) = &file_path3 {
                    // cx.emit(UiEvent::BrowserFileClicked(file_path.clone()));
                    cx.emit(BrowserEvent::SetSelected(file_path.clone()));
                }
            });
    });
}

fn treeview<L>(
    cx: &mut Context,
    lens: L,
    level: u32,
    header: impl Fn(&mut Context, L, u32),
    content: impl Fn(&mut Context, Then<Then<L, Wrapper<children>>, Index<Vec<File>, File>>, u32)
        + 'static,
) where
    L: Lens<Target = File>,
    L::Source: Model,
{
    let content = Rc::new(content);
    VStack::new(cx, |cx| {
        (header)(cx, lens, level);
        Binding::new(cx, lens.then(File::is_open), move |cx, is_open| {
            if is_open.get(cx) {
                let content1 = content.clone();
                VStack::new(cx, |cx| {
                    List::new(cx, lens.then(File::children), move |cx, _, item| {
                        (content1)(cx, item, level + 1);
                    })
                    .width(Stretch(1.0))
                    .height(Auto);

                    let file_path1 = lens.get(cx).file_path.clone();
                    Element::new(cx)
                        .left(Pixels(16.0 * (level + 1) as f32))
                        .height(Stretch(1.0))
                        .width(Pixels(1.0))
                        .position_type(PositionType::SelfDirected)
                        .display(lens.then(File::is_open))
                        .class("dir-line")
                        .toggle_class(
                            "focused",
                            AppData::browser.then(BrowserState::selected).map(move |selected| {
                                if let Some(path) = &file_path1 {
                                    if let Some(selected) = selected {
                                        if let Some(dir) = dir_path(selected) {
                                            dir == path
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            }),
                        );
                })
                .height(Auto);
                //.display(root.clone().then(File::is_open));
            }
        });
    })
    .height(Auto);
}

fn dir_path(path: &Path) -> Option<&Path> {
    if path.is_dir() {
        Some(path)
    } else {
        path.parent()
    }
}
