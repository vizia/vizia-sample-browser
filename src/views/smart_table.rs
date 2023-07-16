use vizia::prelude::*;

pub const CELL_MIN_SIZE_PX: f32 = 100.0;

#[derive(Clone, Copy, Debug)]
pub enum SmartTableEvent {
    StartDrag(usize),
    StopDrag,
}

#[derive(Lens)]
pub struct SmartTable {
    dragging: Option<usize>,
    initialized: bool,
    limiters: Vec<f32>,
    sizes: Vec<f32>, // derived
}

impl SmartTable {
    pub fn new<S>(cx: &mut Context, mut rows: Vec<Vec<S>>) -> Handle<Self>
    where
        S: ToString,
    {
        let transformed: Vec<Vec<String>> =
            rows.iter_mut().map(|row| row.iter().map(|d| d.to_string()).collect()).collect();

        let sizes_len = transformed.len();

        Self {
            dragging: None,
            initialized: false,
            limiters: vec![0.0; sizes_len - 1],
            sizes: vec![0.0; sizes_len],
        }
        .build(cx, |cx| {
            for (i, row) in transformed.iter().enumerate() {
                if i != 0 {
                    Element::new(cx).class("smart-table-divisor").toggle_class("accent", i == 1);
                }
                SmartTableRow::new(cx, &row, SmartTable::sizes, i == 0);
            }
        })
    }
}

impl View for SmartTable {
    fn element(&self) -> Option<&'static str> {
        Some("smart-table")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|e, em| match e {
            SmartTableEvent::StartDrag(n) => {
                self.dragging = Some(*n);

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

pub struct SmartTableRow {}

impl SmartTableRow {
    pub fn new<'a, L>(
        cx: &'a mut Context,
        data: &'a Vec<String>,
        sizes: L,
        header: bool,
    ) -> Handle<'a, Self>
    where
        L: Lens<Target = Vec<f32>> + Copy,
    {
        Self {}
            .build(cx, |cx| {
                for (i, d) in data.iter().enumerate() {
                    if i != 0 {
                        let element =
                            Element::new(cx).class("smart-table-divisor").class("vertical");
                        if header {
                            element.class("accent");
                            ResizeHandle::new(cx, i - 1, true)
                                .toggle_class("active", SmartTable::dragging.map(|d| d.is_some()));
                        }
                    }

                    HStack::new(cx, |cx| {
                        Label::new(cx, d).hoverable(false);
                    })
                    .hoverable(false)
                    .class("smart-table-row-data-container")
                    .width(sizes.map(move |v| {
                        if v[i] == 0.0 {
                            Stretch(1.0)
                        } else {
                            Pixels(v[i])
                        }
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
    pub fn new(cx: &mut Context, i: usize, vertical: bool) -> Handle<Self> {
        Self {}
            .build(cx, |cx| {
                Element::new(cx)
                    .position_type(PositionType::SelfDirected)
                    .class("resize-handle")
                    .cursor(CursorIcon::EwResize)
                    .on_press_down(move |cx| cx.emit(SmartTableEvent::StartDrag(i)));
            })
            .toggle_class("vertical", vertical)
    }
}

impl View for ResizeHandle {
    fn element(&self) -> Option<&'static str> {
        Some("resize-handle")
    }
}
