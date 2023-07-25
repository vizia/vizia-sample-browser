use vizia::prelude::*;

pub struct PopupDivisor;

impl PopupDivisor {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {}.build(cx, |_| {})
    }
}

impl View for PopupDivisor {
    fn element(&self) -> Option<&'static str> {
        Some("popup-divisor")
    }
}
