#[derive(Default, Debug, Clone)]
pub struct Tag(pub String);

#[derive(Default, Debug, Clone)]
pub struct Todo {
    pub heading: String,
    pub body: String,
    pub checked: bool,
    pub tags: Vec<Tag>,
}

