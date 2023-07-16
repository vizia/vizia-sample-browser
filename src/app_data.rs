use vizia::prelude::*;

use crate::state::browser::BrowserState;

#[derive(Lens)]
pub struct AppData {
    pub browser: BrowserState,
}

impl Model for AppData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        self.browser.event(cx, event);
    }
}
