mod db;
mod todo_app;

use db::Config;
use postgres::{Client, NoTls};
use todo_app::{todo::Todo, TodoApp};

fn main() {
    env_logger::init();
    match init_db() {
        Ok(_) => log::trace!("Db initialized successfully"),
        Err(err) => log::error!("Error occured while initializing db: {}", err.to_string()),
    }
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Orange To Do",
        native_options,
        Box::new(|cc| Box::new(TodoApp::new(cc))),
    )
}

fn init_db() -> Result<(), String> {
    let password = std::fs::read_to_string("./secrets.toml").map_err(|err| err.to_string())?;
    let config: Config = toml::de::from_str(&password).map_err(|err| err.to_string())?;

    let connect_config = &format!(
        "host=localhost port=5432 user={} password={}",
        config.db.user, config.db.password
    );

    let mut client = Client::connect(&connect_config, NoTls)
        .map_err(|err| format!("ClientConnect: {}", err.to_string()))?;

    client
        .batch_execute(
            "
                CREATE TABLE IF NOT EXISTS todo (
                    id SERIAL PRIMARY KEY,
                    heading TEXT NOT NULL,
                    body TEXT NOT NULL,
                    checked BOOLEAN NOT NULL,
                    alarm_date DATE,
                    alarm_time TIME,
                    tags TEXT[]
                )
            ",
        )
        .map_err(|err| err.to_string())?;
    Ok(())
}
