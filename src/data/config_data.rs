use std::{collections::HashSet, path::PathBuf};

use ron::de::from_reader;
use serde::{Deserialize, Serialize};
use vizia::prelude::*;

use super::AppEvent;

#[derive(Lens, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub window_size: (u32, u32),
    pub window_position: (u32, u32),

    pub browser_width: f32,
    pub browser_height: f32,
    pub table_height: f32,
    pub waveview_width: f32,

    pub libraries: HashSet<PathBuf>,

    pub recents: Vec<PathBuf>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            browser_width: 300.0,
            table_height: 550.0,
            browser_height: 500.0,
            window_position: (0, 0),
            window_size: (1400, 800),
            ..Default::default()
        }
    }

    pub fn load(&mut self, cx: &mut EventContext) {
        if let Ok(f) = std::fs::File::open("config.ron") {
            let config: Config = from_reader(f).unwrap_or_default();
            *self = config;
            for path in self.libraries.iter() {
                cx.emit(AppEvent::OpenCollection(path.clone()));
            }
        }
    }

    pub fn save(&self) {
        let data = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()).unwrap();
        std::fs::write("config.ron", data).expect("Unable to write file");
    }
}

pub enum ConfigEvent {
    Load,

    SetBrowserWidth(f32),
    SetTableHeight(f32),
    SetBrowserHeight(f32),
    SetWaveviewWidth(f32),
}

impl Model for Config {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.take(|config_event, _| match config_event {
            ConfigEvent::Load => {
                self.load(cx);
            }
            ConfigEvent::SetBrowserWidth(width) => self.browser_width = width,
            ConfigEvent::SetTableHeight(height) => self.table_height = height,
            ConfigEvent::SetBrowserHeight(height) => self.browser_height = height,
            ConfigEvent::SetWaveviewWidth(width) => self.waveview_width = width,
        })
    }
}
