use std::{collections::HashSet, path::PathBuf};

use ron::de::from_reader;
use serde::{Deserialize, Serialize};
use vizia::prelude::*;

use super::AppEvent;

#[derive(Default, Debug, Clone, PartialEq, Data, Serialize, Deserialize)]
pub enum SidebarView {
    #[default]
    Browser,
    Tags,
}

// The width at which the sidebar is hidden
pub const SIDEBAR_HIDDEN_WIDTH: f32 = 50.0;

// The configuration data of the application, saved to disk and loaded on startup
#[derive(Lens, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    // The size of the application window
    pub window_size: (u32, u32),
    // The position of the application window
    pub window_position: (i32, i32),

    // The width of the browser panel
    pub browser_width: f32,
    // The height of the table panel
    pub table_height: f32,
    // The width of the waveview panel
    pub waveview_width: f32,

    // The view currently displayed in the sidebar
    pub sidebar_view: SidebarView,
    // Whether the sidebar is visible
    pub show_sidebar: bool,
    // Whether the waveview panel is visible
    pub waveview_visible: bool,

    pub libraries: HashSet<PathBuf>,

    pub recents: Vec<PathBuf>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            browser_width: 300.0,
            table_height: 550.0,
            window_position: (0, 0),
            window_size: (1400, 800),

            sidebar_view: SidebarView::Browser,
            waveview_visible: true,

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
    // Load the configuration from disk
    Load,
    // Set the width of the browser panel
    SetBrowserWidth(f32),
    // Set the height of the table panel
    SetTableHeight(f32),
    // Set the width of the waveview panel
    SetWaveviewWidth(f32),
    // Show the specified view in the sidebar
    ShowSidebarView(SidebarView),
    // Toggle the visibility of the waveview panel
    ToggleWaveviewVisibility,
}

impl Model for Config {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.take(|config_event, _| match config_event {
            ConfigEvent::Load => {
                self.load(cx);
            }
            ConfigEvent::SetBrowserWidth(width) => {
                self.browser_width = width;

                // Hide sidebar if width is less than SIDEBAR_HIDDEN_WIDTH
                self.show_sidebar = width >= SIDEBAR_HIDDEN_WIDTH;
            }
            ConfigEvent::SetTableHeight(height) => self.table_height = height,
            ConfigEvent::SetWaveviewWidth(width) => self.waveview_width = width,
            ConfigEvent::ShowSidebarView(view) => {
                // Toggle sidebar visibility if the same view is selected
                if !self.show_sidebar {
                    self.show_sidebar = true;
                } else if view == self.sidebar_view {
                    self.show_sidebar = false;
                }

                self.sidebar_view = view;
            }
            ConfigEvent::ToggleWaveviewVisibility => self.waveview_visible ^= true,
        })
    }
}
