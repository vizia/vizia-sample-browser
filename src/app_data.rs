use std::sync::{Arc, Mutex};

use vizia::prelude::*;

use crate::{
    database::prelude::Database,
    state::browser::{BrowserState, Directory},
};

#[derive(Lens)]
pub struct AppData {
    pub database: Arc<Mutex<Database>>,
    pub browser: BrowserState,
    pub browser_width: f32,
    pub table_height: f32,
    pub table_headers: Vec<String>,
    pub table_rows: Vec<Vec<String>>,
    pub search_text: String,
}

pub enum AppEvent {
    SetBrowserWidth(f32),
    SetTableHeight(f32),
}

impl Model for AppData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        self.browser.event(cx, event);

        event.map(|app_event, _| match app_event {
            AppEvent::SetBrowserWidth(width) => self.browser_width = *width,
            AppEvent::SetTableHeight(height) => self.table_height = *height,
        });
    }
}
