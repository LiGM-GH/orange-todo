use std::{
    error::Error,
    fmt, 
};

#[derive(Debug)]
pub enum AddTodoError {
    EmptyBody,
    NoCurrentTodo,
}

impl std::fmt::Display for AddTodoError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}",
            match self {
                Self::EmptyBody => "Body empty",
                Self::NoCurrentTodo => "Current todo is missing!",
            }
        )
    }
}

impl Error for AddTodoError {}

