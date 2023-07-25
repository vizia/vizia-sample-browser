use std::sync::Arc;

use vizia::{
    context::TreeProps,
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
        }
        .build(cx, move |cx| {
            Binding::new(cx, data, move |cx, d| {
                Binding::new(cx, SmartTable::shown, move |cx, _| {
                    let mut data = d.get(cx);

                    let transformed: Vec<Vec<String>> = data
                        .iter_mut()
                        .map(|row| row.iter().map(|d| d.to_string()).collect())
                        .collect();

                    for (i, row) in transformed.iter().enumerate() {
                        if i != 0 {
                            Element::new(cx)
                                .class("smart-table-divisor")
                                .toggle_class("accent", i == 1);
                        }
                        SmartTableRow::new(cx, row.clone(), i == 0)
                            .toggle_class("even", i % 2 == 0);
                    }
                });
            });

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
                println!("GOT TOGGLE {:?}", n);
                self.shown[*n] = !self.shown[*n];
                //
            }
        });

        event.map(|e, _| match e {
            WindowEvent::MouseMove(x, _) => {
                if let Some(i) = self.dragging {
                    let v_w = cx.cache.get_width(cx.current());
                    let b_w = cx.bounds().x;
                    let w = (v_w - b_w) / cx.scale_factor(); // total width

                    let delta_x = (x - b_w) - self.limiters[i];

                    let prev_limiter = if i == 0 { 0.0 } else { self.limiters[i - 1] };
                    let next_limiter =
                        if i == self.limiters.len() - 1 { w } else { self.limiters[i + 1] };

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
pub struct SmartTableRow {
    is_header: bool,
    data: Vec<String>,
}

impl SmartTableRow {
    pub fn new(cx: &mut Context, data: Vec<String>, header: bool) -> Handle<Self> {
        Self { is_header: header, data: data.clone() }
            .build(cx, move |cx| {
                let shown = SmartTable::shown.get(cx);
                for (i, d) in data.iter().enumerate() {
                    let s = shown[i];
                    if s {
                        if i != 0 {
                            let element =
                                Element::new(cx).class("smart-table-divisor").class("vertical");
                            if header {
                                element.class("accent");
                                ResizeHandle::new(cx, i - 1, true).toggle_class(
                                    "active",
                                    SmartTable::dragging
                                        .map(move |d| d.is_some() && d.unwrap() == i - 1),
                                );
                            }
                        }

                        HStack::new(cx, |cx| {
                            Label::new(cx, d).hoverable(false);
                        })
                        .hoverable(false)
                        .class("smart-table-row-data-container")
                        .width(SmartTable::sizes.map(move |v| {
                            if v[i] == 0.0 {
                                Stretch(1.0)
                            } else {
                                Pixels(v[i])
                            }
                        }));
                    }
                }
            })
            .focusable(true)
            .hoverable(true)
            .toggle_class("header", header)
            .layout_type(LayoutType::Row)
    }
}

impl View for SmartTableRow {
    fn element(&self) -> Option<&'static str> {
        Some("smart-table-row")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|e, _| match e {
            WindowEvent::MouseDown(b) => {
                if self.is_header && *b == MouseButton::Right {
                    let prop = Propagation::Subtree;

                    let data = self.data.clone();
                    let shown = SmartTable::shown.get(cx);
                    let callback = move |cx: &mut Context| {
                        let mut i = 0;
                        for h in data.clone() {
                            PopupAction::new(
                                cx,
                                &h,
                                Some(if shown[i] { ICON_EYE } else { ICON_EYE_OFF }),
                            )
                            .on_action(move |cx| {
                                println!("{:?}", i);
                                cx.emit(
                                    Event::new(SmartTableEvent::ToggleShow(i)).direct(cx.parent()),
                                );
                            });
                            i += 1;
                        }
                    };

                    cx.emit_custom(
                        Event::new(PopupEvent::SetMenu(
                            cx.current(),
                            Some(Arc::new(callback)),
                            MenuMeta { hide_on_click: false },
                        ))
                        .propagate(prop),
                    );

                    cx.emit_custom(Event::new(PopupEvent::Show).propagate(prop));
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
            .position_type(PositionType::SelfDirected)
            .class("resize-handle")
            .cursor(CursorIcon::EwResize)
            .on_press_down(move |cx| cx.emit(SmartTableEvent::StartDrag(i)))
    }
}

impl View for ResizeHandle {
    fn element(&self) -> Option<&'static str> {
        Some("resize-handle")
    }
}
