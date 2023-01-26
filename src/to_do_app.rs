use std::fmt;

use egui::{CentralPanel, Color32, Frame, ScrollArea, Style};

#[derive(Default)]
pub struct Tag(String);

#[derive(Default)]
pub struct Todo {
    heading: String,
    body: String,
    checked: bool,
    tags: Vec<Tag>,
}

pub struct TodoEditor {
    todo: Option<Todo>,
    save_result: Result<(), AddTodoError>,
}

impl Default for TodoEditor {
    fn default() -> Self {
        Self {
            todo: None,
            save_result: Ok(()),
        }
    }
}

#[derive(Default)]
pub struct ToDoApp {
    todos: Vec<Todo>,
    mk_todo_dialog_shown: bool,
    todo_editor: TodoEditor,
}

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

impl ToDoApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn add_todo(&mut self) -> Result<(), AddTodoError> {
        if let Some(todo) = self.todo_editor.todo.as_ref() {
            if !todo.body.is_empty() {
                let mut todo = None;
                std::mem::swap(&mut todo, &mut self.todo_editor.todo);

                if let Some(todo) = todo {
                    self.todos.push(todo);
                }

                self.mk_todo_dialog_shown = false;
                Ok(())
            } else {
                Err(AddTodoError::EmptyBody)
            }
        } else {
            Err(AddTodoError::NoCurrentTodo)
        }
    }
}

impl eframe::App for ToDoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Orange To Do - a minimalistic to do app");

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .always_show_scroll(false)
                .show(ui, |ui| {
                    for todo in self.todos.iter() {
                        ui.label(format!(
                            "{} {}",
                            if todo.checked { "V" } else { "X" },
                            &todo.heading
                        ));
                    }

                    if ui.button("Show make-todo-dialog").clicked() {
                        self.mk_todo_dialog_shown = !self.mk_todo_dialog_shown;

                        // TODO: use logging crate to replace this ugly code
                        eprintln!(
                            "{}",
                            if self.mk_todo_dialog_shown {
                                "mk_todo_dialog shown"
                            } else {
                                "mk_todo_dialog hidden"
                            }
                        );
                    }

                    if self.mk_todo_dialog_shown {
                        Frame::window(&Style::default())
                            .fill(Color32::from_rgb(250, 100, 51))
                            .show(ui, |ui| {
                                if self.todo_editor.todo.is_none() {
                                    self.todo_editor.todo = Some(Todo::default());
                                }

                                ui.text_edit_singleline(
                                    &mut self.todo_editor.todo.as_mut().unwrap().heading,
                                );

                                ui.text_edit_singleline(
                                    &mut self.todo_editor.todo.as_mut().unwrap().body,
                                );

                                for tag in self.todo_editor.todo.as_mut().unwrap().tags.iter() {
                                    ui.label(&tag.0);
                                }

                                if ui.button("Create todo!").clicked() {
                                    self.todo_editor.save_result = self.add_todo();
                                }

                                match &self.todo_editor.save_result {
                                    Err(AddTodoError::EmptyBody) => {
                                        ui.label(
                                            egui::RichText::new(
                                                "Add body. Todo can't have empty body!",
                                            )
                                            .color(Color32::RED),
                                        );
                                    }
                                    _ => {}
                                }
                            });
                    }
                });
        });
    }
}
