use vizia::prelude::*;

use crate::svg::SvgDisplay;

#[derive(Lens)]
pub struct PopupAction {
    on_action: Option<Box<dyn Fn(&mut EventContext) + Send + Sync>>,
}

#[derive(Clone, Debug)]
pub enum PopupActionEvent {
    Action,
}

impl PopupAction {
    pub fn new<'a, S>(cx: &'a mut Context, label: S, icon: Option<crate::svg::Icon>) -> Handle<Self>
    where
        S: ToString,
    {
        let has_icon = icon.is_some();

        Self { on_action: None }
            .build(cx, |cx| {
                match icon {
                    Some(i) => {
                        SvgDisplay::new(cx, i).hoverable(false);
                    }
                    None => {}
                }

                Label::new(cx, &label.to_string()).hoverable(false);
            })
            .cursor(CursorIcon::Hand)
            .toggle_class("with-icon", has_icon)
            .layout_type(LayoutType::Row)
            .on_press_down(|cx| cx.emit(PopupActionEvent::Action))
    }
}

pub trait PopupActionHandle {
    fn on_action<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut EventContext) + Send + Sync;
}

impl<'a> PopupActionHandle for Handle<'a, PopupAction> {
    fn on_action<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut EventContext) + Send + Sync,
    {
        self.modify(|popup_action: &mut PopupAction| {
            popup_action.on_action = Some(Box::new(callback))
        })
    }
}

impl View for PopupAction {
    fn element(&self) -> Option<&'static str> {
        Some("popup-action")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|e, _| match e {
            PopupActionEvent::Action => {
                if let Some(action) = &self.on_action {
                    (action)(cx);
                }
            }
        })
    }
}
