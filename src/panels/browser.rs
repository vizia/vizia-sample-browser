use std::path::PathBuf;
use std::rc::Rc;

use vizia::icons::{
    ICON_CHEVRON_DOWN, ICON_FILTER, ICON_FOLDER, ICON_FOLDER_FILLED, ICON_FOLDER_OPEN,
    ICON_LETTER_CASE, ICON_LIST, ICON_LIST_TREE, ICON_SEARCH,
};
use vizia::prelude::*;

use crate::app_data::AppData;
use crate::database::prelude::CollectionID;
use crate::state::browser::directory_derived_lenses::children;
use crate::state::browser::*;
use crate::views::{ToggleButton, ToggleButtonModifiers};

#[derive(Lens)]
pub struct BrowserPanel {
    search_shown: bool,
    tree_view: bool,
}

impl BrowserPanel {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self { search_shown: true, tree_view: true }.build(cx, |cx| {
            Keymap::from(vec![(
                KeyChord::new(Modifiers::CTRL, Code::KeyF),
                KeymapEntry::new((), |cx| cx.emit(BrowserEvent::ToggleShowSearch)),
            )])
            .build(cx);

            // Header
            HStack::new(cx, |cx| {
                // Panel Icon
                Icon::new(cx, ICON_FOLDER_OPEN).class("panel-icon");

                // List/Tree Toggle Buttons
                HStack::new(cx, |cx| {
                    Button::new(
                        cx,
                        |cx| cx.emit(BrowserEvent::ShowTree),
                        |cx| Icon::new(cx, ICON_LIST_TREE),
                    )
                    .checked(BrowserPanel::tree_view);

                    Button::new(
                        cx,
                        |cx| cx.emit(BrowserEvent::ShowList),
                        |cx| Icon::new(cx, ICON_LIST),
                    )
                    .checked(BrowserPanel::tree_view.map(|flag| !flag));
                })
                .class("button-group")
                .width(Auto);

                // Search Toggle Button
                ToggleButton::new(cx, BrowserPanel::search_shown, |cx| Icon::new(cx, ICON_SEARCH))
                    .on_toggle(|cx| cx.emit(BrowserEvent::ToggleShowSearch));
            })
            .class("header");

            // Search Box
            HStack::new(cx, |cx| {
                Textbox::new(cx, AppData::browser.then(BrowserState::search_text))
                    .on_edit(|cx, text| cx.emit(BrowserEvent::Search(text)))
                    .placeholder(Localized::new("search"))
                    .width(Stretch(1.0))
                    .bind(BrowserPanel::search_shown, |mut handle, shown| {
                        if shown.get(&handle) {
                            handle.context().emit(TextEvent::StartEdit);
                        }
                    })
                    .class("search");

                HStack::new(cx, |cx| {
                    // Match Case Toggle Button
                    ToggleButton::new(
                        cx,
                        AppData::browser.then(BrowserState::search_case_sensitive),
                        |cx| Icon::new(cx, ICON_LETTER_CASE),
                    )
                    .on_toggle(|cx| cx.emit(BrowserEvent::ToggleSearchCaseSensitivity))
                    .size(Pixels(20.0))
                    .class("filter-search")
                    .tooltip(|cx| {
                        Label::new(cx, Localized::new("match-case"));
                    });

                    // Filter Results Toggle Button
                    ToggleButton::new(
                        cx,
                        AppData::browser.then(BrowserState::filter_search),
                        |cx| Icon::new(cx, ICON_FILTER),
                    )
                    .on_toggle(|cx| cx.emit(BrowserEvent::ToggleSearchFilter))
                    .size(Pixels(20.0))
                    .class("filter-search")
                    .tooltip(|cx| {
                        Label::new(cx, Localized::new("filter"));
                    });
                })
                .position_type(PositionType::SelfDirected)
                .space(Stretch(1.0))
                .right(Pixels(4.0))
                .col_between(Pixels(2.0))
                .size(Auto);
            })
            .class("searchbar")
            .toggle_class("shown", BrowserPanel::search_shown)
            .col_between(Pixels(8.0))
            .height(Auto);

            // Folder TreeView
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

            // Footer
            HStack::new(cx, |cx| {
                Label::new(cx, "550 samples in 34 folders");
            })
            .class("footer");
        })
    }
}

impl View for BrowserPanel {
    fn element(&self) -> Option<&'static str> {
        Some("browser-panel")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|browser_event, _| match browser_event {
            BrowserEvent::ToggleShowSearch => self.search_shown ^= true,

            BrowserEvent::ShowTree => self.tree_view = true,
            BrowserEvent::ShowList => self.tree_view = false,
            _ => {}
        });

        event.map(|window_event, _| match window_event {
            WindowEvent::KeyDown(code, _) => match code {
                Code::ArrowLeft => cx.emit(BrowserEvent::CollapseDirectory),
                Code::ArrowRight => cx.emit(BrowserEvent::ExpandDirectory),
                Code::ArrowDown => cx.emit(BrowserEvent::FocusNext),
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
        let file_path3 = file_path.clone();

        let selected_lens = AppData::browser
            .then(BrowserState::selected)
            .map(move |selected| selected.contains(&file_path));

        let focused_lens = AppData::browser
            .then(BrowserState::focused)
            .map(move |focused| focused == &Some(file_path2.clone()));

        DirectoryItem::new(cx, root, selected_lens, focused_lens, file_path3)
            .child_left(Pixels(10.0 * level as f32 + 4.0));
    });
}

pub struct DirectoryItem {
    path: PathBuf,
    collection: CollectionID,
}

impl DirectoryItem {
    pub fn new(
        cx: &mut Context,
        root: impl Lens<Target = Directory>,
        selected: impl Lens<Target = bool>,
        focused: impl Lens<Target = bool>,
        path: PathBuf,
    ) -> Handle<Self> {
        let id = root.get(cx).id;
        Self { path: path.clone(), collection: id }
            .build(cx, |cx| {
                // Arrow Icon
                Icon::new(cx, ICON_CHEVRON_DOWN)
                    .class("dir-arrow")
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
                        cx.emit(BrowserEvent::Select(path.clone(), id));
                    })
                    .cursor(CursorIcon::Hand);

                // Folder Icon
                Icon::new(
                    cx,
                    selected.map(
                        |is_selected| if *is_selected { ICON_FOLDER_FILLED } else { ICON_FOLDER },
                    ),
                )
                .class("dir-icon")
                .cursor(CursorIcon::Hand)
                .checked(selected);

                // Directory name
                Label::new(cx, root.then(Directory::name))
                    .width(Stretch(1.0))
                    .text_wrap(false)
                    .hoverable(false)
                    .overflow(Overflow::Hidden)
                    .class("dir-name");

                // Number of Files
                Label::new(cx, root.then(Directory::num_files))
                    .text_wrap(false)
                    .hoverable(false)
                    .class("dir-num");
            })
            .navigable(true)
            .focused(focused)
            .class("dir-item")
            .layout_type(LayoutType::Row)
            .toggle_class("selected", selected)
            .toggle_class(
                "search-match",
                root.then(Directory::match_indices).map(|idx| !idx.is_empty()),
            )
        // TODO
        // .tooltip(|cx| {
        //     Tooltip::new(cx, |cx| {
        //         Label::new(
        //             cx,
        //             root.then(Directory::path)
        //                 .map(|path| path.as_os_str().to_str().unwrap().to_owned()),
        //         );
        //     });
        // })
    }
}

impl View for DirectoryItem {
    fn element(&self) -> Option<&'static str> {
        Some("directory-item")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, _| match window_event {
            WindowEvent::KeyDown(code, _) => match code {
                Code::Escape => cx.emit(BrowserEvent::Deselect),
                _ => {}
            },
            WindowEvent::Press { mouse: _ } => {
                cx.emit(BrowserEvent::SetFocused(Some(self.path.clone())));

                if cx.modifiers().contains(Modifiers::CTRL) {
                    cx.emit(BrowserEvent::AddSelection(self.path.clone()));
                } else {
                    cx.emit(BrowserEvent::Select(self.path.clone(), self.collection));
                }
            }

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
    header: impl Fn(&mut Context, L, u32) + 'static,
    content: impl Fn(&mut Context, Index<Then<L, Wrapper<children>>, Directory>, u32) + 'static,
) where
    L: Lens<Target = Directory>,
    L::Source: Model,
{
    let content = Rc::new(content);
    VStack::new(cx, |cx| {
        Binding::new(cx, lens.then(Directory::shown), move |cx, shown| {
            if shown.get(cx) {
                (header)(cx, lens, level);
                let content = content.clone();
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
                            //     .left(Pixels(10.0 * (level + 1) as f32 + 4.0))
                            //     .height(Stretch(1.0))
                            //     .width(Pixels(1.0))
                            //     .position_type(PositionType::SelfDirected)
                            //     .display(lens.then(Directory::is_open))
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
            }
        });
    })
    .height(Auto);
}
