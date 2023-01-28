use std::{
    error::Error,
    fmt, mem,
    time::{Duration, Instant},
};

use egui::{Align, CentralPanel, Color32, Frame, Layout, RichText, ScrollArea, Style};

const EDITOR_COLOR: Color32 = Color32::from_rgb(250, 100, 51);
const EDITOR_WARNING_COLOR: Color32 = Color32::WHITE;
const BUTTON_SWITCH_DURATION: Duration = Duration::from_millis(100);
const CHECKED_TODO_MARK_COLOR: Color32 = Color32::GREEN;
const UNCHECKED_TODO_MARK_COLOR: Color32 = Color32::RED;

#[derive(Default, Clone)]
pub struct Tag(String);

#[derive(Default, Clone)]
pub struct Todo {
    pub heading: String,
    pub body: String,
    pub checked: bool,
    pub tags: Vec<Tag>,
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
    show_todo_maker: bool,
    todo_maker: TodoEditor,
    todo_editor: TodoEditor,
    todo_bound_with_editor: Option<usize>,
    button_switch_timer: Option<std::time::Instant>,
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

impl Error for AddTodoError {}

impl ToDoApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    pub fn insert_todo(&mut self, todo: Todo) {
        self.todos.push(todo);
    }

    fn add_todo(&mut self) -> Result<(), AddTodoError> {
        if let Some(todo) = self.todo_maker.todo.as_ref() {
            if !todo.body.is_empty() {
                let mut todo = None;

                // &mut self... should be dropped as fast as possible
                {
                    std::mem::swap(&mut todo, &mut self.todo_maker.todo);
                }

                if let Some(todo) = todo {
                    self.todos.push(todo);
                }

                self.show_todo_maker = false;
                Ok(())
            } else {
                Err(AddTodoError::EmptyBody)
            }
        } else {
            Err(AddTodoError::NoCurrentTodo)
        }
    }

    fn make_editor(&mut self, ui: &mut egui::Ui) {
        Frame::window(&Style::default())
            .fill(EDITOR_COLOR)
            .show(ui, |ui| {
                if self.todo_maker.todo.is_none() {
                    self.todo_maker.todo = Some(Todo::default());
                }

                {
                    ui.text_edit_singleline(&mut self.todo_maker.todo.as_mut().unwrap().heading);

                    ui.text_edit_singleline(&mut self.todo_maker.todo.as_mut().unwrap().body);
                }

                for tag in self.todo_maker.todo.as_mut().unwrap().tags.iter() {
                    ui.label(&tag.0);
                }

                if ui.button("Create todo!").clicked() {
                    self.todo_maker.save_result = self.add_todo();
                }

                match &self.todo_maker.save_result {
                    Err(AddTodoError::EmptyBody) => {
                        ui.label(
                            RichText::new("Add body. Todo can't have empty body!")
                                .color(EDITOR_WARNING_COLOR),
                        );
                        log::info!("Todo body not added while trying to save.");
                    }
                    _ => {}
                }
            });
    }

    fn show_all_todos(&mut self, ui: &mut egui::Ui) {
        for (i, todo) in self.todos.iter_mut().enumerate() {
            let mut todo_icon_clicked: bool = false;
            let mut todo_check_clicked: bool = false;

            ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                todo_check_clicked = ui
                    .button({
                        let text = if todo.checked { "V" } else { "X" };
                        let rich = if todo.checked {
                            RichText::from(text).color(CHECKED_TODO_MARK_COLOR)
                        } else {
                            RichText::from(text).color(UNCHECKED_TODO_MARK_COLOR)
                        };

                        rich
                    })
                    .clicked();
                todo_icon_clicked = ui
                    .button({
                        let text = &todo.heading;

                        let rich = if todo.checked {
                            RichText::from(text).strikethrough()
                        } else {
                            RichText::from(text)
                        };

                        rich
                    })
                    .clicked()
            });

            if todo_check_clicked {
                todo.checked = !todo.checked;

                match self.todo_bound_with_editor {
                    Some(num) if num == i => {
                        self.todo_editor.todo.as_mut().unwrap().checked =
                            !self.todo_editor.todo.as_ref().unwrap().checked;
                    }
                    _ => {}
                }
            }

            if todo_icon_clicked {
                log::trace!("Edit-todo dialog shown");

                if self.button_switch_timer.is_none() {
                    self.button_switch_timer = Some(Instant::now());
                }

                if self.todo_bound_with_editor.is_none() {
                    self.todo_bound_with_editor = Some(i);
                    self.todo_editor.todo = Some(Clone::clone(todo));
                } else {
                    log::trace!("{:#?}", self.button_switch_timer);
                    if let Some(timer) = self.button_switch_timer {
                        log::trace!(
                            "Comparison ended with result {}",
                            timer.elapsed() >= BUTTON_SWITCH_DURATION
                        );
                        if timer.elapsed() >= BUTTON_SWITCH_DURATION {
                            log::trace!("Elapsed {} millis", timer.elapsed().as_millis());
                            self.todo_bound_with_editor = None;
                            self.button_switch_timer = None;
                        };
                    }
                }
            }

            match self.todo_bound_with_editor {
                Some(num) if num == i => {
                    Frame::window(&Style::default())
                        .fill(EDITOR_COLOR)
                        .show(ui, |ui| {
                            if let Some(edited_todo) = self.todo_editor.todo.as_mut() {
                                ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                                    ui.text_edit_singleline(&mut edited_todo.heading);

                                    ui.text_edit_singleline(&mut edited_todo.body);
                                });

                                ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                                    for tag in edited_todo.tags.iter() {
                                        ui.label(&tag.0);
                                    }
                                });

                                if ui.button("Save todo!").clicked() {
                                    self.todo_editor.save_result = {
                                        if edited_todo.body.is_empty() {
                                            Err(AddTodoError::EmptyBody)
                                        } else {
                                            mem::swap(edited_todo, todo);
                                            Ok(())
                                        }
                                    };

                                    log::trace!("Editor save result: {:?}", self.todo_editor.save_result);
                                    match self.todo_editor.save_result {
                                        Ok(_) => {
                                            self.todo_editor.todo = None;
                                            self.todo_bound_with_editor = None;
                                        }
                                        Err(AddTodoError::EmptyBody) => {
                                            log::info!("Tried to remove todo's body!");
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            match self.todo_editor.save_result {
                                Err(AddTodoError::EmptyBody) => {
                                    ui.label(
                                        RichText::from("Add body. Todo can't have empty body!")
                                            .color(EDITOR_WARNING_COLOR),
                                    );
                                }
                                _ => {}
                            }
                        });
                }
                _ => (),
            }
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
                    self.show_all_todos(ui);

                    if ui.button("Show make-todo-dialog").clicked() {
                        self.show_todo_maker = !self.show_todo_maker;

                        if self.show_todo_maker {
                            log::trace!("mk_todo_dialog shown")
                        } else {
                            log::trace!("mk_todo_dialog hidden")
                        }
                    }

                    if self.show_todo_maker {
                        self.make_editor(ui);
                    }
                });
        });
    }
}
