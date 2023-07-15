use vizia::prelude::*;

#[derive(Lens)]
pub struct SmartTable {
    override_sizes: Vec<Option<f32>>,
}

impl<'a> SmartTable {
    pub fn new<S>(cx: &'a mut Context, mut rows: Vec<Vec<S>>) -> Handle<'a, Self>
    where
        S: Into<&'a str> + Clone,
    {
        let transformed: Vec<Vec<&str>> =
            rows.iter_mut().map(|row| row.iter().map(|d| d.clone().into()).collect()).collect();

        drop(rows);

        Self { override_sizes: vec![None; transformed.len()] }.build(cx, |cx| {
            for (i, row) in transformed.iter().enumerate() {
                SmartTableRow::new(cx, &row, SmartTable::override_sizes, i == 0);
            }
        })
    }
}

impl View for SmartTable {
    fn element(&self) -> Option<&'static str> {
        Some("smart-table")
    }
}

pub struct SmartTableRow {}

impl SmartTableRow {
    pub fn new<'a, L>(
        cx: &'a mut Context,
        data: &'a Vec<&'a str>,
        sizes: L,
        header: bool,
    ) -> Handle<'a, Self>
    where
        L: Lens<Target = Vec<Option<f32>>>,
    {
        Self {}
            .build(cx, |cx| {
                for (i, d) in data.iter().enumerate() {
                    if i != 0 {
                        Element::new(cx).class("smart-table-divisor");

                        if header {
                            ResizeHandle::new(cx, true);
                        }
                    }

                    HStack::new(cx, |cx| {
                        Label::new(cx, *d).hoverable(false);
                    })
                    .hoverable(false)
                    .class("smart-table-row-data-container")
                    .width(sizes.clone().map(move |v| match v[i] {
                        None => Stretch(1.0),
                        Some(s) => Pixels(s),
                    }));
                }
            })
            .toggle_class("header", header)
            .layout_type(LayoutType::Row)
    }
}

impl View for SmartTableRow {
    fn element(&self) -> Option<&'static str> {
        Some("smart-table-row")
    }
}

pub struct ResizeHandle;

impl ResizeHandle {
    pub fn new(cx: &mut Context, vertical: bool) -> Handle<Self> {
        Self {}
            .build(cx, |cx| {
                Element::new(cx)
                    .position_type(PositionType::SelfDirected)
                    .class("resize-handle")
                    .cursor(CursorIcon::EwResize);
            })
            .toggle_class("vertical", vertical)
    }
}

impl View for ResizeHandle {
    fn element(&self) -> Option<&'static str> {
        Some("resize-handle")
    }
}
