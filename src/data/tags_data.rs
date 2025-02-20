//! GUI state for the tags panel

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use vizia::prelude::*;

use crate::Tag;

// The data model for the tags panel
#[derive(Debug, Lens, Clone, Default)]
pub struct TagsData {
    // The search text in the search box
    pub search_text: String,
    // Whether the search results should be filtered
    pub filter_search: bool,
    // Whether the search should be case sensitive
    pub search_case_sensitive: bool,
    // The tags to display
    pub tags: Vec<Tag>,
}

// The event types for the tags panel
#[derive(Debug, Clone, PartialEq)]
pub enum TagsEvent {
    // Search for a tag
    Search(String),
    // Toggle the visibility of the search box
    ToggleShowSearch,
    // Toggle the filtering of the search results
    ToggleSearchFilter,
    // Toggle the case sensitivity of the search
    ToggleSearchCaseSensitivity,
}

impl Model for TagsData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|tags_event, meta| match tags_event {
            TagsEvent::Search(search_text) => {
                let mut matcher = SkimMatcherV2::default();

                if !self.search_case_sensitive {
                    matcher = matcher.ignore_case()
                } else {
                    matcher = matcher.respect_case()
                }

                for tag in self.tags.iter() {
                    if let Some((_, indices)) = matcher.fuzzy_indices(&tag.name, search_text) {}
                }
            }

            _ => {}
        })
    }
}
