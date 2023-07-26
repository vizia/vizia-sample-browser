use std::sync::Arc;

use vizia::{
    icons::{ICON_EYE, ICON_EYE_OFF},
    prelude::*,
};

use crate::popup_menu::{
    popup_action::{PopupAction, PopupActionHandle},
    MenuMeta, PopupEvent,
};

pub const CELL_MIN_SIZE_PX: f32 = 100.0;

#[derive(Clone, Copy, Debug)]
pub enum SmartTableEvent {
    Initialize,
    StartDrag(usize),
    StopDrag,
    ToggleShow(usize),
}

#[derive(Lens)]
pub struct SmartTable {
    dragging: Option<usize>,
    initialized: bool,
    limiters: Vec<f32>,
    shown: Vec<bool>,
    sizes: Vec<f32>, // derived
    data: Vec<Vec<String>>,
}

impl SmartTable {
    pub fn new<D>(cx: &mut Context, data: D) -> Handle<Self>
    where
        D: Lens<Target = Vec<Vec<String>>>,
    {
        let collumns_len = data.get(cx)[0].len();

        Self {
            dragging: None,
            initialized: false,
            shown: vec![true; collumns_len],
            limiters: vec![0.0; collumns_len - 1],
            sizes: vec![0.0; collumns_len],
            data: data.get(cx),
        }
        .build(cx, move |cx| {
            for i in 0..data.get(cx).len() {
                if i != 0 {
                    Element::new(cx).class("smart-table-divisor").toggle_class("accent", i == 1);
                }
                SmartTableRow::new(
                    cx,
                    data.map(move |v| v[i].clone()),
                    SmartTable::shown,
                    i == 0,
                    cx.current(),
                )
                .toggle_class("even", i % 2 == 0);
            }

            cx.emit(SmartTableEvent::Initialize);
        })
    }
}

impl View for SmartTable {
    fn element(&self) -> Option<&'static str> {
        Some("smart-table")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|e, em| match e {
            SmartTableEvent::Initialize => {
                if !self.initialized {
                    self.initialized = true;

                    // Convert from Stretch units to pixels for each portion.
                    let v_w = cx.cache.get_width(cx.current());
                    let b_w = cx.bounds().x;
                    let w = (v_w - b_w) / cx.scale_factor();

                    let stretch_width = w / self.sizes.len() as f32;
                    for size in self.sizes.iter_mut() {
                        *size = stretch_width;
                    }

                    let mut acc = 0.0;
                    for (i, l) in self.limiters.iter_mut().enumerate() {
                        acc += self.sizes[i];
                        *l = acc;
                    }
                }
            }

            SmartTableEvent::StartDrag(n) => {
                self.dragging = Some(*n);

                cx.capture();
                cx.lock_cursor_icon();
                em.consume();
            }
            SmartTableEvent::StopDrag => {
                self.dragging = None;

                cx.release();
                cx.unlock_cursor_icon();
                em.consume();
            }

            SmartTableEvent::ToggleShow(n) => {
                self.shown[*n] = !self.shown[*n];

                if self.shown[*n] {
                    let v_w = cx.cache.get_width(cx.current());
                    let b_w = cx.bounds().x;
                    let w = (v_w - b_w) / cx.scale_factor();

                    // check no oversize
                    let sum: f32 = self.sizes.iter().sum();
                    let perc = sum / w;
                    if perc > 1. {
                        for (i, size) in self.sizes.iter_mut().enumerate() {
                            if i != *n {
                                *size *= 1. / perc;
                            }
                        }
                    }
                }

                let mut acc = 0.0;
                for (i, l) in self.limiters.iter_mut().enumerate() {
                    if self.shown[i] {
                        acc += self.sizes[i];
                        *l = acc;
                    }
                }

                update_right_click_menu(cx, cx.current(), self.data[0].clone(), self.shown.clone());

                em.consume();
            }
        });

        event.map(|e, _| match e {
            WindowEvent::MouseMove(x, _) => {
                if let Some(i) = self.dragging {
                    let v_w = cx.cache.get_width(cx.current());
                    let b_w = cx.bounds().x;
                    let w = (v_w - b_w) / cx.scale_factor(); // total width

                    let delta_x = (x - b_w) - self.limiters[i];

                    let prev_limiter = {
                        let mut last = 0.0f32;
                        for i in 0..i {
                            if self.shown[i] {
                                last = self.limiters[i];
                            }
                        }
                        last
                    };
                    let next_limiter = {
                        let mut last = w;
                        for i in (i + 1..self.shown.len() - 1).rev() {
                            if self.shown[i] {
                                last = self.limiters[i];
                            }
                        }
                        last
                    };

                    // Update new limiter position

                    if delta_x.is_sign_positive() {
                        self.limiters[i] += delta_x;

                        // Min width
                        if next_limiter - self.limiters[i] < CELL_MIN_SIZE_PX {
                            self.limiters[i] = next_limiter - CELL_MIN_SIZE_PX;
                        }
                    } else {
                        // Min width
                        self.limiters[i] += delta_x;
                        if self.limiters[i] - prev_limiter < CELL_MIN_SIZE_PX {
                            self.limiters[i] = prev_limiter + CELL_MIN_SIZE_PX;
                        }
                    }

                    // Set new Sizes

                    if i == 0 {
                        self.sizes[i] = self.limiters[i];
                    } else {
                        self.sizes[i] = self.limiters[i] - prev_limiter;
                    }

                    if i == self.limiters.len() - 1 {
                        self.sizes[i + 1] = w - self.limiters[i]
                    } else {
                        self.sizes[i + 1] = next_limiter - self.limiters[i];
                    }
                }
            }

            WindowEvent::MouseUp(b) => {
                if *b == MouseButton::Left {
                    cx.emit(SmartTableEvent::StopDrag)
                }
            }
            _ => {}
        });
    }
}

#[derive(Lens)]
pub struct SmartTableRow<D: Lens> {
    parent: Entity,
    is_header: bool,
    data: D,
}

impl<D> SmartTableRow<D>
where
    D: Lens<Target = Vec<String>>,
{
    pub fn new<S>(cx: &mut Context, data: D, shown: S, header: bool, parent: Entity) -> Handle<Self>
    where
        S: Lens<Target = Vec<bool>>,
    {
        Self { parent, is_header: header, data: data.clone() }
            .build(cx, move |cx| {
                let data_len = data.get(cx).len();
                for i in 0..data_len {
                    if i != 0 {
                        let element = Element::new(cx)
                            .class("smart-table-divisor")
                            .class("vertical")
                            .disabled(SmartTable::shown.map(move |v| !{
                                //
                                if i - 1 == data_len - 2 {
                                    v[i] && v[i - 1]
                                } else {
                                    v[i - 1]
                                }
                            }));
                        if header {
                            element.class("accent");
                            ResizeHandle::new(cx, i - 1, true).toggle_class(
                                "active",
                                SmartTable::dragging
                                    .map(move |d| d.is_some() && d.unwrap() == i - 1),
                            );
                        }
                    }

                    HStack::new(cx, move |cx| {
                        Label::new(cx, data.map(move |v| v[i].clone())).hoverable(false);
                    })
                    .hoverable(false)
                    .toggle_class("hidden", shown.map(move |v| !v[i]))
                    .class("smart-table-row-data-container")
                    .width(SmartTable::sizes.map(move |v| {
                        if v[i] == 0.0 {
                            Stretch(1.0)
                        } else {
                            Pixels(v[i])
                        }
                    }));
                }
            })
            .focusable(true)
            .hoverable(true)
            .toggle_class("header", header)
            .layout_type(LayoutType::Row)
    }
}

impl<D> View for SmartTableRow<D>
where
    D: Lens<Target = Vec<String>>,
{
    fn element(&self) -> Option<&'static str> {
        Some("smart-table-row")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|e, _| match e {
            WindowEvent::MouseDown(b) => {
                if self.is_header && *b == MouseButton::Right {
                    update_right_click_menu(
                        cx,
                        self.parent,
                        self.data.get(cx),
                        SmartTable::shown.get(cx),
                    );
                    open_right_click_menu(cx);
                }
            }

            _ => {}
        });
    }
}

pub struct ResizeHandle;

impl ResizeHandle {
    pub fn new(cx: &mut Context, i: usize, vertical: bool) -> Handle<Self> {
        Self {}
            .build(cx, |_: &mut Context| {})
            .left(SmartTable::limiters.map(move |v| Pixels(v[i] - 2.0 + i as f32 * 1.5)))
            .toggle_class("vertical", vertical)
            .disabled(SmartTable::shown.map(move |v| !v[i]))
            .position_type(PositionType::SelfDirected)
            .class("resize-handle")
            .cursor(CursorIcon::EwResize)
            .on_press_down(move |cx| cx.emit(SmartTableEvent::StartDrag(i)))
            .hoverable(true)
            .focusable(true)
    }
}

impl View for ResizeHandle {
    fn element(&self) -> Option<&'static str> {
        Some("resize-handle")
    }
}

pub fn open_right_click_menu(cx: &mut EventContext) {
    cx.emit_custom(Event::new(PopupEvent::Show).propagate(Propagation::Subtree));
}

pub fn update_right_click_menu(
    cx: &mut EventContext,
    target: Entity,
    data: Vec<String>,
    shown: Vec<bool>,
) {
    let callback = move |cx: &mut Context| {
        let mut i = 0;
        for h in data.clone() {
            PopupAction::new(
                cx,
                format!("{} {}", if shown[i] { "Hide" } else { "Show" }, h),
                Some(if shown[i] { ICON_EYE } else { ICON_EYE_OFF }),
            )
            .on_action(move |cx| {
                cx.emit_to(target, SmartTableEvent::ToggleShow(i));
            })
            .toggle_class("active", shown[i]);
            i += 1;
        }
    };

    cx.emit_custom(
        Event::new(PopupEvent::SetMenu(
            cx.current(),
            Some(Arc::new(callback)),
            MenuMeta { hide_on_click: false },
        ))
        .propagate(Propagation::Subtree),
    );
}
