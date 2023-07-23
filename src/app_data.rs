use vizia::prelude::*;

#[derive(Lens)]
pub struct AppData {
    pub smart_table_data: Vec<Vec<String>>,
}

impl Model for AppData {}
