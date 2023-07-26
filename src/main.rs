use app_data::AppData;
use popup_menu::PopupMenu;
use views::smart_table::SmartTable;
use vizia::prelude::*;

mod app_data;

mod popup_menu;
mod views;

fn main() {
    Application::new(|cx| {
        cx.add_stylesheet(include_style!("styles.css")).unwrap();

        AppData {
            smart_table_data: vec![
                vec!["Col 1", "Col 2", "Col 3", "Col 4"]
                    .iter_mut()
                    .map(|v| v.to_string())
                    .collect(),
                vec!["data 1.1", "data 1.2", "data 1.3", "data 1.4"]
                    .iter_mut()
                    .map(|v| v.to_string())
                    .collect(),
                vec!["data 2.1", "data 2.2", "data 2.3", "data 2.4"]
                    .iter_mut()
                    .map(|v| v.to_string())
                    .collect(),
                vec!["data 3.1", "data 3.2", "data 3.3", "data 3.4"]
                    .iter_mut()
                    .map(|v| v.to_string())
                    .collect(),
            ],
        }
        .build(cx);

        SmartTable::new(cx, AppData::smart_table_data);
    })
    .ignore_default_theme()
    .title("Vizia Sample Browser")
    .run();
}
