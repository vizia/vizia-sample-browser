use vizia::prelude::*;

use crate::state::browser::BrowserState;

#[derive(Lens)]
pub struct AppData {
    pub browser: BrowserState,
    pub browser_width: f32,
}

pub enum AppEvent {
    SetBrowserWidth(f32),
}

impl Model for AppData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        self.browser.event(cx, event);

        event.map(|app_event, _| match app_event {
            AppEvent::SetBrowserWidth(width) => self.browser_width = *width,
        });
    }
}
