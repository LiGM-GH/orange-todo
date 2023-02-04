mod db;
mod todo_app;

use db::Config;
use postgres::{Client, NoTls};
use todo_app::TodoApp;

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

fn init_db() -> Result<(), anyhow::Error> {
    let password = std::fs::read_to_string("./secrets.toml")?;
    let config: Config = toml::de::from_str(&password)?;

    let connect_config = &format!(
        "host=localhost port=5432 user={} password={}",
        config.db.user, config.db.password
    );

    let mut client = Client::connect(&connect_config, NoTls)?;

    client.batch_execute(
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
    )?;
    Ok(())
}
