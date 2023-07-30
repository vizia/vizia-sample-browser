use std::path::PathBuf;
use std::rc::Rc;

use vizia::icons::{
    ICON_CHEVRON_DOWN, ICON_FOLDER, ICON_FOLDER_FILLED, ICON_FOLDER_OPEN, ICON_LIST,
    ICON_LIST_TREE, ICON_SEARCH,
};
use vizia::prelude::*;

use crate::app_data::AppData;
use crate::state::browser::directory_derived_lenses::children;
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

            // Panel Header
            HStack::new(cx, |cx| {
                // Panel Icon
                Icon::new(cx, ICON_FOLDER_OPEN).class("panel-icon");

                // List/Tree Toggle Buttons
                HStack::new(cx, |cx| {
                    ToggleButton::new(cx, Browser::tree_view, |cx| Icon::new(cx, ICON_LIST_TREE));
                    ToggleButton::new(cx, Browser::tree_view.map(|flag| !flag), |cx| {
                        Icon::new(cx, ICON_LIST)
                    });
                })
                .class("button-group")
                .width(Auto);

                // Search Toggle Button
                ToggleButton::new(cx, Browser::search_shown, |cx| Icon::new(cx, ICON_SEARCH))
                    .on_toggle(|cx| cx.emit(BrowserEvent::ToggleShowSearch));
            })
            .class("header");

            // Search Box
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

            // Folder Treeview
            ScrollView::new(cx, 0.0, 0.0, false, true, |cx| {
                treeview(
                    cx,
                    AppData::browser.then(BrowserState::libraries.index(0)),
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

            // Panel Footer
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

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|browser_event, _| match browser_event {
            BrowserEvent::ToggleShowSearch => self.search_shown ^= true,
            _ => {}
        });

        event.map(|window_event, _| match window_event {
            WindowEvent::KeyDown(code, _) => match code {
                Code::ArrowLeft => cx.emit(BrowserEvent::CollapseDirectory),
                Code::ArrowRight => cx.emit(BrowserEvent::ExpandDirectory),
                Code::ArrowDown => {
                    cx.emit(BrowserEvent::FocusNext);
                }
                Code::ArrowUp => cx.emit(BrowserEvent::FocusPrev),
                _ => {}
            },

            WindowEvent::FocusOut => {
                BrowserEvent::SetFocused(None);
            }

            _ => {}
        });
    }
}

fn directory<L>(cx: &mut Context, root: L, level: u32)
where
    L: Lens<Target = Directory>,
{
    Binding::new(cx, root.then(Directory::path), move |cx, file_path| {
        let file_path = file_path.get(cx);
        let file_path2 = file_path.clone();

        let selected_lens = AppData::browser
            .then(BrowserState::focused)
            .map(move |selected| Some(file_path.clone()) == *selected);

        DirectoryItem::new(cx, root, selected_lens, file_path2)
            .child_left(Pixels(10.0 * level as f32 + 4.0));
    });
}

pub struct DirectoryItem {
    path: PathBuf,
}

impl DirectoryItem {
    pub fn new(
        cx: &mut Context,
        root: impl Lens<Target = Directory>,
        selected: impl Lens<Target = bool>,
        path: PathBuf,
    ) -> Handle<Self> {
        let file_path2 = path.clone();
        Self { path: path.clone() }
            .build(cx, |cx| {
                // Arrow Icon
                Icon::new(cx, ICON_CHEVRON_DOWN)
                    .class("toggle_folder")
                    .visibility(root.then(Directory::children).map(|c| !c.is_empty()))
                    .hoverable(root.then(Directory::children).map(|c| !c.is_empty()))
                    .rotate(root.then(Directory::is_open).map(|flag| {
                        if *flag {
                            Angle::Deg(0.0)
                        } else {
                            Angle::Deg(-90.0)
                        }
                    }))
                    .on_press(move |cx| {
                        cx.emit(BrowserEvent::ToggleDirectory(path.clone()));
                        cx.emit(BrowserEvent::SetFocused(Some(path.clone())));
                    });

                // Folder Icon
                Icon::new(
                    cx,
                    selected.map(
                        |is_selected| if *is_selected { ICON_FOLDER_FILLED } else { ICON_FOLDER },
                    ),
                )
                .class("folder-icon")
                .checked(selected);

                // Directory name
                Label::new(cx, root.then(Directory::name))
                    .width(Stretch(1.0))
                    .text_wrap(false)
                    .hoverable(false);

                // Number of Files
                Label::new(cx, root.then(Directory::num_files))
                    .width(Auto)
                    .left(Stretch(1.0))
                    .text_wrap(false)
                    .hoverable(false);
            })
            .navigable(true)
            .class("dir-item")
            .layout_type(LayoutType::Row)
            .toggle_class("selected", selected)
            .on_press(move |cx| {
                cx.focus();
                cx.emit(BrowserEvent::SetFocused(Some(file_path2.clone())));
            })
    }
}

impl View for DirectoryItem {
    fn element(&self) -> Option<&'static str> {
        Some("directory-item")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, _| match window_event {
            // WindowEvent::KeyDown(code, _) => match code {
            //     Code::ArrowLeft => cx.emit(BrowserEvent::CollapseDirectory),
            //     Code::ArrowRight => cx.emit(BrowserEvent::ExpandDirectory),
            //     Code::ArrowDown => {
            //         cx.emit(BrowserEvent::FocusNext);
            //     }
            //     Code::ArrowUp => cx.emit(BrowserEvent::FocusPrev),
            //     _ => {}
            // },
            WindowEvent::FocusIn => {
                cx.emit(BrowserEvent::SetFocused(Some(self.path.clone())));
            }

            _ => {}
        });
    }
}

fn treeview<L>(
    cx: &mut Context,
    lens: L,
    level: u32,
    header: impl Fn(&mut Context, L, u32),
    content: impl Fn(&mut Context, Index<Then<L, Wrapper<children>>, Directory>, u32) + 'static,
) where
    L: Lens<Target = Directory>,
    L::Source: Model,
{
    let content = Rc::new(content);
    VStack::new(cx, |cx| {
        (header)(cx, lens, level);
        Binding::new(cx, lens.then(Directory::is_open), move |cx, is_open| {
            if is_open.get(cx) {
                let content1 = content.clone();
                VStack::new(cx, |cx| {
                    List::new(cx, lens.then(Directory::children), move |cx, _, item| {
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
