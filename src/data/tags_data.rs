//! GUI state for the tags panel

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use vizia::prelude::*;

use crate::Tag;

#[derive(Debug, Lens, Clone, Default)]
pub struct TagsData {
    pub search_text: String,
    pub filter_search: bool,
    pub search_case_sensitive: bool,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TagsEvent {
    Search(String),
    ToggleShowSearch,
    ToggleSearchFilter,
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
