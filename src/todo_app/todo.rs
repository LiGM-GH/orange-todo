use thiserror::Error;

type Heading = String;
type Body = String;
type Tag = String;

#[derive(Default, Debug, Clone)]
pub struct Todo {
    heading: Heading,
    body: Body,
    pub checked: bool,
    pub tags: Vec<Tag>,
}

impl Todo {
    pub fn new(heading: String, body: String) -> Result<Self, TodoError> {
        let mut instance = Self {
            heading: String::new(),
            body: String::new(),
            checked: false,
            tags: Vec::new(),
        };

        instance.set_body(body)?;
        instance.set_heading(heading)?;

        Ok(instance)
    }

    pub fn set_heading(&mut self, heading: String) -> Result<(), TodoError> {
        if heading.is_empty() {
            Err(TodoError::EmptyHeading)
        } else {
            self.heading = heading;
            Ok(())
        }
    }

    pub fn set_body(&mut self, body: String) -> Result<(), TodoError> {
        if body.is_empty() {
            Err(TodoError::EmptyBody)
        } else {
            self.body = body;
            Ok(())
        }
    }

    pub fn heading(&self) -> &str {
        &self.heading
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn check(&mut self) {
        self.checked = true;
    }

    pub fn add_tags(&mut self, tags: Vec<Tag>) {
        for tag in tags {
            self.tags.push(tag);
        }
    }
}

impl TryFrom<PartialTodo> for Todo {
    type Error = TodoError;

    fn try_from(partial: PartialTodo) -> Result<Todo, Self::Error> {
        let mut result = Self::new(partial.heading, partial.body)?;

        if partial.checked { result.check(); }
        result.add_tags(partial.tags);
        Ok(result)
    }
}

#[derive(Error, Debug)]
pub enum TodoError {
    #[error("Heading empty!")]
    EmptyHeading,
    #[error("Body empty!")]
    EmptyBody,
    #[error("Something unexpected happened!")]
    Unknown,
}

#[derive(Debug, Clone, Default)]
pub struct PartialTodo {
    pub heading: Heading,
    pub body: Body,
    pub checked: bool,
    pub tags: Vec<Tag>,
}

impl From<Todo> for PartialTodo {
    fn from(value: Todo) -> Self {
        Self {
            heading: value.heading,
            body: value.body,
            checked: value.checked,
            tags: value.tags,
        }
    }
}
