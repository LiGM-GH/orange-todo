use crate::Todo;

use super::add_todo_error::AddTodoError;

pub struct TodoEditor {
    pub todo: Option<Todo>,
    pub save_result: Result<(), AddTodoError>,
}

impl Default for TodoEditor {
    fn default() -> Self {
        Self {
            todo: None,
            save_result: Ok(()),
        }
    }
}

