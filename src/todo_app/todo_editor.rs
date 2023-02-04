use super::todo::{TodoError, PartialTodo};

#[derive(Debug)]
pub struct TodoEditor {
    pub todo: Option<PartialTodo>,
    pub save_result: Result<(), TodoError>,
}

impl Default for TodoEditor {
    fn default() -> Self {
        Self {
            todo: None,
            save_result: Ok(()),
        }
    }
}
