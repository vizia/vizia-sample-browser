use vizia::prelude::*;

const POSITION_OFFSET: f32 = 8.;
const WINDOW_BOUNDS: f32 = 16.;

#[derive(Data, Clone, PartialEq)]
pub enum PopupType {
    Empty,
    SmartTableHeader(Vec<(String, bool)>),
}

pub enum PopupEvent {
    Show,
    Hide,
    SetMenu(Entity, PopupType),
}

#[derive(Clone, Copy, Debug, Data, PartialEq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Lens)]
pub struct PopupMenu {
    shown: bool,
    target_entity: Option<Entity>,
    current_menu: PopupType,
    position: (f32, f32),
    corner: Corner,
}

impl PopupMenu {
    pub fn new(cx: &mut Context) {
        Self {
            target_entity: None,
            current_menu: PopupType::Empty,
            shown: false,
            position: (0., 0.),
            corner: Corner::TopLeft,
        }
        .build(cx, |cx| {
            Binding::new(cx, PopupMenu::shown, |cx, s| {
                if s.get(cx) {
                    Binding::new(cx, PopupMenu::current_menu, |cx, cm| match cm.get(cx) {
                        PopupType::Empty => {}

                        PopupType::SmartTableHeader(sth) => {
                            // Binding::new(cx, PopupMenu::corner, |cx, c| {
                            //     let corner = c.get(cx);
                            //     println!("{:?}", corner);
                            //     Label::new(
                            //         cx,
                            //         match corner {
                            //             Corner::TopLeft => "TopLeft",
                            //             Corner::TopRight => "TopRight",
                            //             Corner::BottomLeft => "BottomLeft",
                            //             Corner::BottomRight => "BottomRight",
                            //         },
                            //     );
                            // });
                            for (header, shown) in sth {
                                Label::new(cx, &header);
                            }
                        }
                    })
                }
            })
        })
        .class("right-click-menu")
        // .bind(PopupMenu::corner, |mut h, c| {
        //     let txt = match c.get(h.context()) {
        //         Corner::TopLeft => "topleft-corner",
        //         Corner::TopRight => "topright-corner",
        //         Corner::BottomLeft => "bottomleft-corner",
        //         Corner::BottomRight => "bottomright-corner",
        //     };
        //     h.text(txt);
        // })
        .toggle_class("hidden", PopupMenu::shown.map(|v| !v))
        .position_type(PositionType::SelfDirected)
        .left(PopupMenu::position.map(|v| Pixels(v.0)))
        .top(PopupMenu::position.map(|v| Pixels(v.1)));
    }
}

impl View for PopupMenu {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|e, _| match e {
            PopupEvent::SetMenu(entity, menu) => {
                self.target_entity = Some(*entity);
                self.current_menu = menu.clone();
            }

            PopupEvent::Show => {
                self.shown = true;
            }

            PopupEvent::Hide => {
                self.shown = false;
            }
        });

        event.map(|e, _| match e {
            WindowEvent::MouseDown(m) => {
                if *m == MouseButton::Left {
                    cx.emit(PopupEvent::Hide)
                }
            }

            WindowEvent::GeometryChanged(e) => {
                let window_bounds = cx.cache.get_bounds(Entity::root());

                let cursor = (cx.mouse().cursorx, cx.mouse().cursory);

                let mut desired_pos = (cursor.0 + POSITION_OFFSET, cursor.1 + POSITION_OFFSET);

                let width = cx.bounds().width();
                let height = cx.bounds().height();

                println!("{:?} - {:?} {:?}", desired_pos, width, height);

                // check horizontally

                if desired_pos.0 + width > window_bounds.w - WINDOW_BOUNDS {
                    desired_pos.0 = cursor.0 - width;
                    self.corner = Corner::TopRight;
                } else if desired_pos.0 < WINDOW_BOUNDS {
                    desired_pos.0 = WINDOW_BOUNDS;
                    self.corner = Corner::TopLeft;
                }

                // check vertically

                if desired_pos.1 + height > window_bounds.h - WINDOW_BOUNDS {
                    desired_pos.1 = cursor.1 - height;
                    self.corner = match self.corner {
                        Corner::TopRight => Corner::BottomRight,
                        _ => Corner::BottomLeft,
                    };
                } else if desired_pos.1 < WINDOW_BOUNDS {
                    desired_pos.1 = WINDOW_BOUNDS;
                    self.corner = match self.corner {
                        Corner::TopRight => Corner::BottomRight,
                        _ => Corner::BottomLeft,
                    };
                }

                self.position = desired_pos;
            }

            _ => {}
        })
    }
}

// pub trait PopupMenuHandle: Sized + DataContext {
//     fn on_blur<F>(self, f: F) -> Self
//     where
//         F: 'static + Fn(&mut EventContext);
// }

// impl<'a> PopupMenuHandle for Handle<'a, PopupMenu> {
//     fn on_blur<F>(mut self, f: F) -> Self
//     where
//         F: 'static + Fn(&mut EventContext),
//     {
//         let focus_event = Box::new(f);
//         let cx = self.context();
//         let entity = self.entity();
//         cx.with_current(entity, |cx| {
//             cx.add_listener(move |popup: &mut PopupMenu, cx, event| {
//                 event.map(|window_event, meta| match window_event {
//                     WindowEvent::MouseDown(_) => {
//                         if popup.shown && meta.origin != cx.current() {
//                             // Check if the mouse was pressed outside of any descendants
//                             if !cx.hovered().is_descendant_of(cx.tree, cx.current()) {
//                                 (focus_event)(cx);
//                                 meta.consume();
//                             }
//                         }
//                     }

//                     WindowEvent::KeyDown(code, _) => {
//                         if popup.shown && *code == Code::Escape {
//                             (focus_event)(cx);
//                         }
//                     }

//                     _ => {}
//                 });
//             });
//         });

//         self
//     }
// }
