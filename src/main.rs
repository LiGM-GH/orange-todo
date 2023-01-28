mod to_do_app;

use to_do_app::{ToDoApp, Todo};

fn main() {
    env_logger::init();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Orange ToDo",
        native_options,
        Box::new(|cc| {
            Box::new({
                let mut app = ToDoApp::new(cc);

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
