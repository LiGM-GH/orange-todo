use thiserror::Error;

pub type Heading = String;
pub type Body = String;
pub type Tag = String;
pub type Id = i32;

#[derive(Default, Debug, Clone)]
pub struct Todo {
    id: Id,
    heading: Heading,
    body: Body,
    pub checked: bool,
    pub tags: Vec<Tag>,
}

impl Todo {
    pub fn new(id: Id, heading: String, body: String) -> Result<Self, TodoError> {
        let mut instance = Self {
            id: 0,
            heading: String::new(),
            body: String::new(),
            checked: false,
            tags: Vec::new(),
        };

        instance.set_id(id)?;
        instance.set_body(body)?;
        instance.set_heading(heading)?;

        Ok(instance)
    }

    pub fn set_id(&mut self, id: Id) -> Result<(), TodoError> {
        if id == 0 {
            Err(TodoError::ZeroId)
        } else {
            self.id = id;
            Ok(())
        }
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

    pub fn id(&self) -> &i32 {
        &self.id
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
        let mut result = Self::new(partial.id, partial.heading, partial.body)?;

        if partial.checked {
            result.check();
        }

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
    #[error("Zero ID!")]
    ZeroId,
}

#[derive(Debug, Clone)]
pub struct PartialTodo {
    pub id: Id,
    pub heading: Heading,
    pub body: Body,
    pub checked: bool,
    pub tags: Vec<Tag>,
}

impl PartialTodo {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            heading: String::new(),
            body: String::new(),
            checked: false,
            tags: Vec::new(),
        }
    }
}

impl From<Todo> for PartialTodo {
    fn from(value: Todo) -> Self {
        Self {
            id: value.id,
            heading: value.heading,
            body: value.body,
            checked: value.checked,
            tags: value.tags,
        }
    }
}
