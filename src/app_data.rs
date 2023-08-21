use crate::collections_to_directories;
use crate::{
    database::prelude::*,
    state::browser::{BrowserState, Directory},
};
use std::sync::{Arc, Mutex};
use vizia::prelude::*;

#[derive(Lens)]
pub struct AppData {
    pub database: Arc<Mutex<Database>>,
    pub browser: BrowserState,
    pub browser_width: f32,
    pub table_height: f32,
    pub table_headers: Vec<String>,
    pub table_rows: Vec<AudioFile>,
    pub search_text: String,
}

pub enum AppEvent {
    SetBrowserWidth(f32),
    SetTableHeight(f32),
    ViewCollection(CollectionID),
    UpdateDirectories,
    UpdateDirectoriesError(Vec<notify::Error>),
}

impl Model for AppData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        self.browser.event(cx, event);

        event.map(|app_event, _| match app_event {
            AppEvent::SetBrowserWidth(width) => self.browser_width = *width,
            AppEvent::SetTableHeight(height) => self.table_height = *height,
            AppEvent::ViewCollection(id) => {
                println!("selected: {}", id);
                if let Ok(db) = self.database.lock() {
                    if let Ok(audio_files) = db.get_all_audio_files() {
                        println!("ALL {}", audio_files.len());
                        self.table_rows = audio_files;
                    }
                }
                println!("num rows: {}", self.table_rows.len());
            }
            AppEvent::UpdateDirectories => {
                let mut db = self.database.lock().unwrap();

                db.update_database();

                let collections = db.get_all_collections().unwrap();
                let audio_files = db.get_all_audio_files().unwrap();

                let root = collections.iter().find(|v| v.parent_collection().is_none()).unwrap();

                let root = collections_to_directories(&collections, &audio_files, root.clone());

                let audio_files = db.get_all_audio_files().unwrap().len();

                self.browser.libraries = vec![root];

                println!("Update directories");
            }
            AppEvent::UpdateDirectoriesError(e) => {
                println!("Update directories ERROR {:?}", e);
            }
        });
    }
}
