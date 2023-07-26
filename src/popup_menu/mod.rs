use std::sync::Arc;

use vizia::prelude::*;

pub mod popup_action;
pub mod popup_divisor;

const POSITION_OFFSET: f32 = 8.;
const WINDOW_BOUNDS: f32 = 16.;

#[derive(Data, Clone, PartialEq)]
pub struct MenuMeta {
    pub hide_on_click: bool,
}

pub enum PopupEvent {
    Show,
    Hide,
    SetMenu(Entity, Option<Arc<dyn Fn(&mut Context) + Send + Sync>>, MenuMeta),
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
    target_entity: Option<Entity>,
    position: (f32, f32),
    corner: Corner,
    meta: MenuMeta,
}

impl PopupMenu {
    pub fn new(
        cx: &mut Context,
        shown: impl Lens<Target = Option<(f32, f32)>>,
        content: impl Fn(&mut Context),
    ) -> Handle<Self> {
        Self {
            target_entity: None,
            position: (0., 0.),
            corner: Corner::TopLeft,
            meta: MenuMeta { hide_on_click: false },
        }
        .build(cx, |cx| {
            (content)(cx);

            Binding::new(cx, shown, |cx, shown| {
                if let Some((x, y)) = shown.get(cx) {
                    cx.emit(PopupEvent::Show);
                }
            });
        })
        .toggle_class("topleft-corner", PopupMenu::corner.map(|v| *v == Corner::TopLeft))
        .toggle_class("topright-corner", PopupMenu::corner.map(|v| *v == Corner::TopRight))
        .toggle_class("bottomleft-corner", PopupMenu::corner.map(|v| *v == Corner::BottomLeft))
        .toggle_class("bottomright-corner", PopupMenu::corner.map(|v| *v == Corner::BottomRight))
        .toggle_class("hidden", shown.map(|v| v.is_none()))
        .position_type(PositionType::SelfDirected)
        .left(PopupMenu::position.map(|v| Pixels(v.0)))
        .top(PopupMenu::position.map(|v| Pixels(v.1)))
    }
}

impl View for PopupMenu {
    fn element(&self) -> Option<&'static str> {
        Some("popup-menu")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|e, em| match e {
            // PopupEvent::SetMenu(entity, menu, meta) => {
            //     self.target_entity = Some(*entity);
            //     self.meta = meta.clone();

            //     em.consume();
            // }
            PopupEvent::Show => {
                // if self.shown {
                //     cx.emit(PopupEvent::Hide);
                //     return;
                // }

                let window_bounds = cx.cache.get_bounds(Entity::root());

                let width = cx.bounds().width();
                let height = cx.bounds().height();

                let bounds = cx.bounds();

                let cursor = cx.mouse().right.pos_down;
                let mut desired_pos = (
                    cursor.0.floor() - bounds.x + POSITION_OFFSET,
                    cursor.1.floor() - bounds.y + POSITION_OFFSET,
                );

                // Check horizontally
                self.corner = Corner::TopLeft;
                if desired_pos.0 + width > window_bounds.w - WINDOW_BOUNDS {
                    // Right-side overflow
                    desired_pos.0 = cursor.0 - width;
                    self.corner = Corner::TopRight;
                } else if desired_pos.0 < WINDOW_BOUNDS {
                    // Left-side overflow
                    desired_pos.0 = WINDOW_BOUNDS;
                }

                // Check vertically
                if desired_pos.1 + height > window_bounds.h - WINDOW_BOUNDS {
                    // Right-side overflow
                    desired_pos.1 = cursor.1 - height;
                    self.corner = match self.corner {
                        Corner::TopRight => Corner::BottomRight,
                        _ => Corner::BottomLeft,
                    };
                } else if desired_pos.1 < WINDOW_BOUNDS {
                    // Left-side overflow
                    desired_pos.1 = WINDOW_BOUNDS;
                    self.corner = match self.corner {
                        Corner::TopRight => Corner::BottomRight,
                        _ => Corner::BottomLeft,
                    };
                }

                let scale = cx.scale_factor();

                cx.set_translate((Pixels(desired_pos.0 / scale), Pixels(desired_pos.1 / scale)));
                cx.needs_redraw();
                // self.shown = true;
            } // PopupEvent::Hide => {
            //     self.shown = false;
            // }
            _ => {}
        });
    }
}
