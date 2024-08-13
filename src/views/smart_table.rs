use vizia::{
    icons::{ICON_EYE, ICON_EYE_OFF},
    prelude::*,
};

use crate::popup_menu::{
    popup_action::{PopupAction, PopupActionHandle},
    MenuMeta, PopupEvent, PopupMenu,
};

pub const CELL_MIN_SIZE_PX: f32 = 100.0;

#[derive(Clone, Copy, Debug)]
pub enum SmartTableEvent {
    Initialize,
    StartDrag(usize),
    StopDrag,
    ToggleColumn(usize),
    SetColWidth(usize, f32),
    ShowMenu,
}

#[derive(Lens)]
pub struct SmartTable {
    dragging: Option<usize>,
    initialized: bool,
    limiters: Vec<f32>,
    shown: Vec<bool>,
    widths: Vec<Units>, // derived
    show_menu: Option<(f32, f32)>,
}

impl SmartTable {
    pub fn new<L1, L2, R: 'static, T1: 'static, F>(
        cx: &mut Context,
        headers: L1,
        rows: L2,
        content: F,
    ) -> Handle<Self>
    where
        L1: Lens,
        L2: Lens,
        <L1 as Lens>::Target: std::ops::Deref<Target = [T1]>,
        <L2 as Lens>::Target: std::ops::Deref<Target = [R]>,
        R: Data,
        T1: Data + ToStringLocalized,
        // T2: Data + ToStringLocalized,
        F: 'static + Copy + Fn(&mut Context, MapRef<L2, R>, usize),
    {
        let num_cols = headers.map(|h| h.len()).get(cx);

        Self {
            dragging: None,
            initialized: false,
            shown: vec![true; num_cols],
            limiters: vec![0.0; num_cols - 1],
            widths: vec![Pixels(100.0); num_cols],
            // data: data.get(cx),
            show_menu: None,
        }
        .build(cx, |cx| {
            VStack::new(cx, |cx| {
                // Headers
                List::new(cx, headers, |cx, col_index, item| {
                    HStack::new(cx, move |cx| {
                        Label::new(cx, item).class("column-heading").hoverable(false);
                    })
                    .hoverable(false)
                    .width(Self::widths.idx(col_index))
                    .height(Auto);
                })
                .hoverable(true)
                .class("header")
                .width(Stretch(1.0))
                .layout_type(LayoutType::Row);

                // Resize Handles
                List::new(cx, SmartTable::limiters, |cx, idx, limiter| {
                    ResizeHandle::new(cx, limiter, idx, true)
                        // .background_color(Color::red())
                        .height(Pixels(20.0));
                })
                .class("handles")
                .size(Stretch(1.0))
                .position_type(PositionType::SelfDirected);
            })
            .height(Auto);
            //
            List::new(cx, rows, move |cx, row_index, row| {
                //
                List::new(cx, headers, move |cx, col_index, _| {
                    HStack::new(cx, move |cx| {
                        (content)(cx, row, col_index);
                    })
                    .class("column")
                    .overflow(Overflow::Hidden)
                    .width(Self::widths.idx(col_index))
                    .height(Pixels(32.0));
                })
                .class("row")
                .toggle_class("odd", row_index % 2 == 0)
                .width(Stretch(1.0))
                .layout_type(LayoutType::Row);
            })
            .class("row-list")
            .width(Stretch(1.0));
        })
        .on_build(|cx| cx.emit(SmartTableEvent::Initialize))
        .toggle_class("dragging", SmartTable::dragging.map(|dragging| dragging.is_some()))

        // PopupMenu::new(cx, SmartTable::show_menu, |cx| {
        //     List::new(cx, data.map(|data| data[0].clone()), |cx, index, item| {
        //         let shown_lens = SmartTable::shown.index(index);

        //         Binding::new(cx, item, move |cx, name| {
        //             Binding::new(cx, shown_lens, move |cx, shown| {
        //                 let is_shown = shown.get(cx);
        //                 let name = name.get(cx);
        //                 PopupAction::new(
        //                     cx,
        //                     format!("{} {}", if is_shown { "Hide" } else { "Show" }, name),
        //                     Some(if is_shown { ICON_EYE } else { ICON_EYE_OFF }),
        //                 )
        //                 .on_action(move |cx| {
        //                     cx.emit(SmartTableEvent::ToggleShow(index));
        //                 })
        //                 .toggle_class("active", shown_lens)
        //                 .width(Stretch(1.0));
        //             });
        //         });
        //     })
        //     .width(Stretch(1.0));
        // });

        // })
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
                    // let bounds = cx.bounds();

                    // let w = bounds.w / cx.scale_factor();

                    // let stretch_width = w / self.widths.len() as f32;
                    // for size in self.widths.iter_mut() {
                    //     *size = Pixels(stretch_width);
                    // }

                    let mut acc = 0.0;
                    for (i, l) in self.limiters.iter_mut().enumerate() {
                        acc += self.widths[i].to_px(0.0, 0.0);
                        *l = acc;
                    }
                }
            }

            SmartTableEvent::StartDrag(index) => {
                self.dragging = Some(*index);
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

            SmartTableEvent::SetColWidth(index, width) => {
                if *width > CELL_MIN_SIZE_PX {
                    let current_width = self.widths[*index].to_px(0.0, 0.0);
                    if let Some(next_width) = self.widths.as_slice().get(index + 1) {
                        let total_width = current_width + next_width.to_px(0.0, 0.0);
                        let new_next_width = total_width - *width;
                        if new_next_width < CELL_MIN_SIZE_PX {
                            return;
                        }
                        self.widths[index + 1] = Pixels(total_width - *width);
                    }

                    self.widths[*index] = Pixels(*width);
                }
            }

            SmartTableEvent::ShowMenu => {
                self.show_menu = Some(cx.mouse().right.pos_down);
            }

            SmartTableEvent::ToggleColumn(n) => {
                self.shown[*n] = !self.shown[*n];

                if self.shown[*n] {
                    let v_w = cx.cache.get_width(cx.current());
                    let b_w = cx.bounds().x;
                    let w = (v_w - b_w) / cx.scale_factor();

                    // check no oversize
                    // let sum: f32 = self.widths.iter().sum();
                    // let perc = sum / w;
                    // if perc > 1. {
                    //     for (i, size) in self.widths.iter_mut().enumerate() {
                    //         if i != *n {
                    //             *size *= 1. / perc;
                    //         }
                    //     }
                    // }
                }

                // let mut acc = 0.0;
                // for (i, l) in self.limiters.iter_mut().enumerate() {
                //     if self.shown[i] {
                //         acc += self.sizes[i];
                //         *l = acc;
                //     }
                // }

                em.consume();
            }
        });

        event.map(|e, _| match e {
            WindowEvent::MouseMove(x, _) => {
                if let Some(limiter) = self.dragging {
                    let v_w = cx.cache.get_width(cx.current());
                    let b_w = cx.bounds().x;
                    let delta_x = (x - b_w) / cx.scale_factor() - self.limiters[limiter];
                    let prev_limiter = {
                        let mut last = 0.0f32;
                        for i in 0..limiter {
                            if self.shown[i] {
                                last = self.limiters[i];
                            }
                        }
                        last
                    };
                    let next_limiter = {
                        let mut last = v_w;
                        for i in (limiter + 1..self.shown.len() - 1).rev() {
                            if self.shown[i] {
                                last = self.limiters[i];
                            }
                        }
                        last
                    };

                    let scale = cx.scale_factor();

                    // Update new limiter position
                    if delta_x.is_sign_positive() {
                        self.limiters[limiter] += delta_x / scale;
                        // Min width
                        if next_limiter - self.limiters[limiter] < CELL_MIN_SIZE_PX {
                            self.limiters[limiter] = next_limiter - CELL_MIN_SIZE_PX;
                        }
                    } else {
                        // Min width
                        self.limiters[limiter] += delta_x / scale;
                        if self.limiters[limiter] - prev_limiter < CELL_MIN_SIZE_PX {
                            self.limiters[limiter] = prev_limiter + CELL_MIN_SIZE_PX;
                        }
                    }

                    // Set new Sizes
                    if limiter == 0 {
                        self.widths[limiter] = Pixels(self.limiters[limiter]);
                    } else {
                        self.widths[limiter] = Pixels(self.limiters[limiter] - prev_limiter);
                    }
                    if limiter == self.limiters.len() - 1 {
                        self.widths[limiter + 1] = Pixels(v_w - self.limiters[limiter]);
                    } else {
                        self.widths[limiter + 1] = Pixels(next_limiter - self.limiters[limiter]);
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

// #[derive(Lens)]
// pub struct SmartTableRow {}

// impl SmartTableRow {
//     pub fn new<R, T>(
//         cx: &mut Context,
//         row: impl Lens<Target = R>,
//         row_index: usize,
//         content: impl Fn(&mut Context),
//     ) -> Handle<Self>
//     where
//         R: Data + std::ops::Deref<Target = [T]>,
//         T: Data + ToStringLocalized,
//     {
//         Self {}
//             .build(cx, move |cx| {
//                 List::new(cx, row, move |cx, col_index, item| {
//                     HStack::new(cx, move |cx| {
//                         (content)(cx, item);
//                     })
//                     .width(Self::widths.index(col_index))
//                     .height(Auto);
//                 })
//                 .class("row")
//                 .toggle_class("odd", row_index % 2 == 0)
//                 .width(Stretch(1.0))
//                 .layout_type(LayoutType::Row);

//                 // let data_len = data.get(cx).len();
//                 // for i in 0..data_len {
//                 //     if i != 0 {
//                 //         let element = Element::new(cx)
//                 //             .class("smart-table-divisor")
//                 //             .class("vertical")
//                 //             .disabled(SmartTable::shown.map(move |v| !{
//                 //                 //
//                 //                 if i - 1 == data_len - 2 {
//                 //                     v[i] && v[i - 1]
//                 //                 } else {
//                 //                     v[i - 1]
//                 //                 }
//                 //             }));
//                 //         if header {
//                 //             element.class("accent");
//                 //             ResizeHandle::new(cx, i - 1, true)
//                 //                 .toggle_class("active", SmartTable::dragging);
//                 //         }
//                 //     }

//                 //     HStack::new(cx, move |cx| {
//                 //         Label::new(cx, data.map(move |v| v[i].clone())).hoverable(false);
//                 //     })
//                 //     .hoverable(false)
//                 //     .toggle_class("hidden", shown.map(move |v| !v[i]))
//                 //     .class("smart-table-row-data-container");
//                 //     // .width(SmartTable::widths.map(move |v| {
//                 //     //     if v[i] == 0.0 {
//                 //     //         Stretch(1.0)
//                 //     //     } else {
//                 //     //         Pixels(v[i])
//                 //     //     }
//                 //     // }));
//                 // }
//             })
//             .focusable(true)
//             .hoverable(true)
//             .layout_type(LayoutType::Row)
//     }
// }

// impl View for SmartTableRow {
//     fn element(&self) -> Option<&'static str> {
//         Some("smart-table-row")
//     }

//     fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
//         event.map(|e, _| match e {
//             WindowEvent::MouseDown(b) => {
//                 cx.emit(SmartTableEvent::ShowMenu);
//             }

//             _ => {}
//         });
//     }
// }

pub struct ResizeHandle;

impl ResizeHandle {
    pub fn new(
        cx: &mut Context,
        left: impl Lens<Target = f32>,
        index: usize,
        vertical: bool,
    ) -> Handle<Self> {
        Self {}
            .build(cx, |_: &mut Context| {})
            .left(left.map(|l| Pixels(*l)))
            // .left(SmartTable::limiters.map(move |v| Pixels(v[i] - 2.0 + i as f32 * 1.5)))
            .toggle_class("vertical", vertical)
            // .disabled(SmartTable::shown.map(move |v| !v[i]))
            .position_type(PositionType::SelfDirected)
            .class("resize-handle")
            .cursor(CursorIcon::EwResize)
            .on_press_down(move |cx| cx.emit(SmartTableEvent::StartDrag(index)))
            .hoverable(true)
            .focusable(true)
    }
}

impl View for ResizeHandle {
    fn element(&self) -> Option<&'static str> {
        Some("resize-handle")
    }
}
