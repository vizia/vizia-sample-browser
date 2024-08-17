use vizia::prelude::*;

use vizia::icons::{
    ICON_CHEVRON_DOWN, ICON_FILTER, ICON_FOLDER, ICON_FOLDER_FILLED, ICON_FOLDER_OPEN,
    ICON_LETTER_CASE, ICON_LIST, ICON_LIST_TREE, ICON_SEARCH, ICON_TAG,
};

use crate::app_data::AppData;
use crate::data::{TagsData, TagsEvent};
use crate::Tag;

#[derive(Lens)]
pub struct TagsPanel {
    search_shown: bool,
}

impl TagsPanel {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self { search_shown: false }.build(cx, |cx| {
            Keymap::from(vec![(
                KeyChord::new(Modifiers::CTRL, Code::KeyF),
                KeymapEntry::new((), |cx| cx.emit(TagsEvent::ToggleShowSearch)),
            )])
            .build(cx);

            // Header
            HStack::new(cx, |cx| {
                // Panel Icon
                Svg::new(cx, ICON_TAG).class("panel-icon");

                Label::new(cx, "TAGS").class("title");

                // Search Toggle Button
                ToggleButton::new(cx, TagsPanel::search_shown, |cx| Svg::new(cx, ICON_SEARCH))
                    .on_toggle(|cx| cx.emit(TagsEvent::ToggleShowSearch))
                    .class("toggle-search")
                    .tooltip(|cx| {
                        Tooltip::new(cx, |cx| {
                            Label::new(cx, Localized::new("toggle-search"));
                        })
                    });
            })
            .class("header");

            // Search Box
            HStack::new(cx, |cx| {
                Textbox::new(cx, AppData::tags.then(TagsData::search_text))
                    .on_edit(|cx, text| cx.emit(TagsEvent::Search(text.clone())))
                    .placeholder(Localized::new("search"))
                    .width(Stretch(1.0))
                    .bind(TagsPanel::search_shown, |mut handle, shown| {
                        if shown.get(&handle) {
                            handle.context().emit(TextEvent::StartEdit);
                        }
                    })
                    .class("search");

                HStack::new(cx, |cx| {
                    // Match Case Toggle Button
                    ToggleButton::new(
                        cx,
                        AppData::tags.then(TagsData::search_case_sensitive),
                        |cx| Svg::new(cx, ICON_LETTER_CASE),
                    )
                    .on_toggle(|cx| cx.emit(TagsEvent::ToggleSearchCaseSensitivity))
                    .size(Pixels(20.0))
                    .class("filter-search")
                    .tooltip(|cx| {
                        Tooltip::new(cx, |cx| {
                            Label::new(cx, Localized::new("match-case"));
                        })
                    });

                    // Filter Results Toggle Button
                    ToggleButton::new(cx, AppData::tags.then(TagsData::filter_search), |cx| {
                        Svg::new(cx, ICON_FILTER)
                    })
                    .on_toggle(|cx| cx.emit(TagsEvent::ToggleSearchFilter))
                    .size(Pixels(20.0))
                    .class("filter-search")
                    .tooltip(|cx| {
                        Tooltip::new(cx, |cx| {
                            Label::new(cx, Localized::new("filter"));
                        })
                    });
                })
                .position_type(PositionType::SelfDirected)
                .space(Stretch(1.0))
                .right(Pixels(4.0))
                .col_between(Pixels(2.0))
                .size(Auto);
            })
            .class("searchbar")
            .toggle_class("shown", TagsPanel::search_shown)
            .col_between(Pixels(8.0))
            .height(Auto);

            // Tags List
            // TODO - List of tags
            VirtualList::new(cx, AppData::tags.then(TagsData::tags), 30.0, |cx, index, tag| {
                HStack::new(cx, |cx| {
                    Element::new(cx)
                        .background_color(tag.then(Tag::color).map(|col| Color::from(col.as_str())))
                        .class("tag-color");
                    Label::new(cx, tag.then(Tag::name)).class("tag-name");
                    Label::new(cx, tag.then(Tag::number)).class("tag-num");
                })
                .class("tag")
            });

            // Footer
            HStack::new(cx, |cx| {
                Label::new(
                    cx,
                    AppData::tags
                        .then(TagsData::tags)
                        .map(|tags| format!("{} total tags", tags.len())),
                );
            })
            .class("footer");
        })
    }
}

impl View for TagsPanel {
    fn element(&self) -> Option<&'static str> {
        Some("tags-panel")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|tags_event, _| match tags_event {
            TagsEvent::ToggleShowSearch => self.search_shown ^= true,
            _ => {}
        });
    }
}
