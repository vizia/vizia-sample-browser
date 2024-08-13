//! GUI state for the tags panel

use vizia::prelude::*;

#[derive(Debug, Lens, Clone, Default)]
pub struct TagsState {
    pub search_text: String,
    pub filter_search: bool,
    pub search_case_sensitive: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TagsEvent {
    Search(String),
    ToggleShowSearch,
    ToggleSearchFilter,
    ToggleSearchCaseSensitivity,
}

impl Model for TagsState {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|tags_event, meta| match tags_event {
            TagsEvent::Search(search_text) => {
                //
            }

            _ => {}
        })
    }
}
