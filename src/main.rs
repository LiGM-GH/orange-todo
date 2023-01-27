mod to_do_app;

use to_do_app::ToDoApp;

fn main() {
    env_logger::init();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Orange ToDo",
        native_options,
        Box::new(|cc| Box::new(ToDoApp::new(cc))),
    )
}
