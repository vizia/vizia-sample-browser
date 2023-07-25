use std::rc::Rc;

use vizia::icons::ICON_MENU_2;
use vizia::{prelude::*, ICON_CHEVRON_DOWN, ICON_SEARCH};

use crate::app_data::AppData;
use crate::state::browser::file_derived_lenses::children;
use crate::state::browser::*;

#[derive(Lens)]
pub struct Browser {
    search_text: String,
}

impl Browser {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self { search_text: String::new() }.build(cx, |cx| {
            cx.emit(BrowserEvent::ViewAll);

            HStack::new(cx, |cx| {
                // Textbox::new(cx, Browser::search_text)
                //     .placeholder(Localized::new("search"))
                //     .width(Stretch(1.0));
                HStack::new(cx, |cx: &mut Context| {
                    Textbox::new(cx, Browser::search_text)
                        .class("icon-before")
                        .width(Stretch(1.0))
                        .placeholder(Localized::new("search"));
                    // .on_edit(|cx, text| cx.emit(AppDataSetter::EditableText(text)));
                    Icon::new(cx, ICON_SEARCH)
                        .color(Color::gray())
                        .size(Pixels(28.0))
                        .position_type(PositionType::SelfDirected);
                })
                .height(Auto)
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

            treeview(
                cx,
                AppData::browser.then(BrowserState::root_file),
                0,
                directory,
                |cx, item, level| {
                    treeview(cx, item, level, directory, |cx, item, level| {
                        treeview(cx, item, level, directory, |cx, item, level| {
                            treeview(cx, item, level, directory, |cx, item, level| {
                                treeview(cx, item, level, directory, directory);
                            });
                        });
                    });
                },
            );
        })
    }
}

impl View for Browser {
    fn element(&self) -> Option<&'static str> {
        Some("browser")
    }
}

fn directory<L>(cx: &mut Context, root: L, level: u32)
where
    L: Lens<Target = File>,
{
    Binding::new(cx, root.then(File::path), move |cx, file_path| {
        let file_path1 = file_path.get(cx);
        let file_path2 = file_path.get(cx);
        let file_path3 = file_path.get(cx);
        let file_path4 = file_path.get(cx);
        HStack::new(cx, |cx| {
            // Arrow Icon
            Label::new(cx, ICON_CHEVRON_DOWN)
                .class("icon")
                .class("toggle_folder")
                .visibility(root.then(File::children).map(|c| !c.is_empty()))
                .hoverable(root.then(File::children).map(|c| !c.is_empty()))
                .rotate(root.then(File::is_open).map(|flag| {
                    if *flag {
                        Angle::Deg(0.0)
                    } else {
                        Angle::Deg(-90.0)
                    }
                }))
                .on_press(move |cx| {
                    if let Some(file_path) = &file_path4 {
                        cx.emit(BrowserEvent::ToggleOpen(file_path.clone()));
                    }
                });
            // Directory name
            Label::new(cx, root.then(File::name))
                .width(Stretch(1.0))
                .text_wrap(false)
                .hoverable(false);
        })
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
            cx.focus();
            if let Some(file_path) = &file_path3 {
                cx.emit(BrowserEvent::SetSelected(file_path.clone()));
            }
        })
        // .col_between(Pixels(4.0))
        .child_left(Pixels(8.0 * level as f32 + 4.0));
    });
}

fn treeview<L>(
    cx: &mut Context,
    lens: L,
    level: u32,
    header: impl Fn(&mut Context, L, u32),
    content: impl Fn(&mut Context, Index<Then<L, Wrapper<children>>, File>, u32) + 'static,
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
                })
                .height(Auto);
            }
        });
    })
    .height(Auto);
}
