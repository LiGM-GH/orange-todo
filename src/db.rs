#[derive(serde::Deserialize)]
pub struct Config {
    pub db: Db,
}

#[derive(serde::Deserialize)]
pub struct Db {
    pub user: String,
    pub password: String,
}

pub mod set {
    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error("Couldn't setup db properly")]
        SetupDb,
        #[error("Couldn't read config from file 'secrets.toml' properly")]
        ReadConfig,
    }
}
