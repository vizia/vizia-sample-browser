use views::smart_table::SmartTable;
use vizia::prelude::*;

mod views;

fn main() {
    Application::new(|cx| {
        cx.add_stylesheet(include_style!("styles.css")).unwrap();

        SmartTable::new(
            cx,
            vec![
                vec!["Col 1", "Col 2", "Col 3"],
                vec!["data 1.1", "data 1.2", "data 1.3"],
                vec!["data 2.1", "data 2.2", "data 2.3"],
            ],
        );
    })
    .ignore_default_theme()
    .title("Vizia Sample Browser")
    .run();
}
