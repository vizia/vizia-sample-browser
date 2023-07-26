use std::rc::Rc;

use vizia::icons::{
    ICON_CHEVRON_DOWN, ICON_FOLDER, ICON_FOLDER_FILLED, ICON_FOLDER_OPEN, ICON_LIST,
    ICON_LIST_TREE, ICON_SEARCH,
};
use vizia::prelude::*;

use crate::app_data::AppData;
use crate::state::browser::file_derived_lenses::children;
use crate::state::browser::*;
use crate::views::{ToggleButton, ToggleButtonModifiers};

#[derive(Lens)]
pub struct Browser {
    search_text: String,
    search_shown: bool,
    tree_view: bool,
}

impl Browser {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self { search_text: String::new(), search_shown: true, tree_view: true }.build(cx, |cx| {
            cx.emit(BrowserEvent::ViewAll);

            // Panel icon
            HStack::new(cx, |cx| {
                Icon::new(cx, ICON_FOLDER_OPEN).class("panel-icon");
                HStack::new(cx, |cx| {
                    ToggleButton::new(cx, Browser::tree_view, |cx| Icon::new(cx, ICON_LIST_TREE));
                    ToggleButton::new(cx, Browser::tree_view.map(|flag| !flag), |cx| {
                        Icon::new(cx, ICON_LIST)
                    });
                })
                .class("button-group")
                .width(Auto);
                ToggleButton::new(cx, Browser::search_shown, |cx| Icon::new(cx, ICON_SEARCH))
                    .on_toggle(|cx| cx.emit(BrowserEvent::ToggleShowSearch));
            })
            .class("header");

            // Top bar
            HStack::new(cx, |cx| {
                Textbox::new(cx, Browser::search_text)
                    .width(Stretch(1.0))
                    .placeholder(Localized::new("search"))
                    .bind(Browser::search_shown, |mut handle, shown| {
                        if shown.get(&handle) {
                            handle.context().emit(TextEvent::StartEdit);
                        }
                    })
                    .class("search");
                // .on_edit(|cx, text| cx.emit(AppDataSetter::EditableText(text)));
            })
            .class("searchbar")
            .toggle_class("shown", Browser::search_shown)
            .col_between(Pixels(8.0))
            .height(Auto);

            //
            ScrollView::new(cx, 0.0, 0.0, false, true, |cx| {
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
            });

            // Footer
            HStack::new(cx, |cx| {
                Label::new(cx, "550 samples in 34 folders");
            })
            .class("footer");
        })
    }
}

impl View for Browser {
    fn element(&self) -> Option<&'static str> {
        Some("browser")
    }

    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|browser_event, _| match browser_event {
            BrowserEvent::ToggleShowSearch => self.search_shown ^= true,
            _ => {}
        })
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

        let selected_lens = AppData::browser
            .then(BrowserState::selected)
            .map(move |selected| &file_path1 == selected);

        HStack::new(cx, |cx| {
            // Arrow Icon
            Icon::new(cx, ICON_CHEVRON_DOWN)
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
                    if let Some(file_path) = &file_path3 {
                        cx.emit(BrowserEvent::ToggleOpen(file_path.clone()));
                        cx.emit(BrowserEvent::SetSelected(file_path.clone()));
                    }
                });
            Icon::new(
                cx,
                selected_lens
                    .map(|is_selected| if *is_selected { ICON_FOLDER_FILLED } else { ICON_FOLDER }),
            )
            .class("folder-icon")
            .checked(selected_lens);
            // Directory name
            Label::new(cx, root.then(File::name))
                .width(Stretch(1.0))
                .text_wrap(false)
                .hoverable(false);
        })
        .class("dir-item")
        // .toggle_class(
        //     "focused",
        //     AppData::browser.then(BrowserState::selected).map(move |selected| {
        //         match (&file_path1, selected) {
        //             (Some(fp), Some(s)) => s.starts_with(fp),
        //             _ => false,
        //         }
        //     }),
        // )
        .toggle_class("selected", selected_lens)
        .on_press(move |cx| {
            cx.focus();
            if let Some(file_path) = &file_path2 {
                cx.emit(BrowserEvent::SetSelected(file_path.clone()));
            }
        })
        // .col_between(Pixels(4.0))
        .child_left(Pixels(10.0 * level as f32 + 4.0));
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

                    // Element::new(cx)
                    //     .left(Pixels(10.0 * (level + 1) as f32))
                    //     .height(Stretch(1.0))
                    //     .width(Pixels(1.0))
                    //     .position_type(PositionType::SelfDirected)
                    //     .display(lens.then(File::is_open))
                    //     .class("dir-line");
                    // .toggle_class(
                    //     "focused",
                    //     AppData::browser.then(BrowserState::selected).map(move |selected| {
                    //         if let Some(path) = &file_path1 {
                    //             if let Some(selected) = selected {
                    //                 if let Some(dir) = dir_path(selected) {
                    //                     dir == path
                    //                 } else {
                    //                     false
                    //                 }
                    //             } else {
                    //                 false
                    //             }
                    //         } else {
                    //             false
                    //         }
                    //     }),
                    // );
                })
                .height(Auto);
            }
        });
    })
    .height(Auto);
}
