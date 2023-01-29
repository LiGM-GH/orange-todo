mod todo_app;

use todo_app::{TodoApp, Todo};

fn main() {
    env_logger::init();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Orange To Do",
        native_options,
        Box::new(|cc| {
            Box::new({
                let mut app = TodoApp::new(cc);

                app.insert_todo(Todo {
                    heading: "Hello world!".to_string(),
                    body: "".to_string(),
                    checked: true,
                    tags: Vec::new(),
                });

                app
            })
        }),
    )
}
