use vizia::icons::{ICON_LIST_SEARCH, ICON_SEARCH};
use vizia::prelude::*;

use crate::app_data::AppData;
use crate::views::SmartTable;

pub struct SamplesPanel {}

impl SamplesPanel {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {}.build(cx, |cx| {
            HStack::new(cx, |cx| {
                Icon::new(cx, ICON_LIST_SEARCH).class("panel-icon");

                HStack::new(cx, |cx| {
                    Textbox::new(cx, AppData::search_text)
                        .class("icon-before")
                        .width(Stretch(1.0))
                        .class("search")
                        .placeholder(Localized::new("search"));
                    // .on_edit(|cx, text| cx.emit(AppDataSetter::EditableText(text)));
                    Icon::new(cx, ICON_SEARCH)
                        .color(Color::gray())
                        .size(Pixels(28.0))
                        .position_type(PositionType::SelfDirected);
                })
                .height(Auto)
                .width(Stretch(1.0));
            })
            .col_between(Pixels(8.0))
            .height(Auto)
            .class("header");

            SmartTable::new(cx, AppData::table_headers, AppData::table_rows, |cx, item| {
                Label::new(cx, item)
                    .width(Stretch(1.0))
                    .border_color(Color::bisque())
                    // .border_width(Pixels(1.0))
                    .child_space(Stretch(1.0))
                    .child_left(if item.idx() == 0 { Pixels(4.0) } else { Stretch(1.0) });
            });
        })
    }
}

impl View for SamplesPanel {
    fn element(&self) -> Option<&'static str> {
        Some("samples-panel")
    }
}
