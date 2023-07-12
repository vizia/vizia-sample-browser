use vizia::prelude::*;

fn main() {
    Application::new(|cx| {
        Label::new(cx, "Hello World");
    })
    .title("Vizia Sample Browser")
    .run();
}
