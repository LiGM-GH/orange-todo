#[derive(serde::Deserialize)]
pub struct Config {
    pub db: Db,
}

#[derive(serde::Deserialize)]
pub struct Db {
    pub user: String,
    pub password: String,
}

