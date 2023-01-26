use std::rc::Rc;

use egui::{CentralPanel, Color32, ComboBox, Frame, RichText, Rounding, ScrollArea, Style, Ui};

#[derive(Default)]
pub struct Tag(String);

#[derive(Default)]
pub struct Todo {
    heading: String,
    body: String,
    tags: Vec<Tag>,
}

#[derive(Default)]
pub struct ToDoApp {
    todos: Vec<Todo>,
    mk_todo_dialog_shown: bool,
    edited_todo: Option<Todo>,
}

impl ToDoApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
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
                        ui.label(&todo.heading);
                    }

                    if ui.button("Show make-todo-dialog").clicked() {
                        self.mk_todo_dialog_shown = !self.mk_todo_dialog_shown;

                        println!(
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
                                if self.edited_todo.is_none() {
                                    self.edited_todo = Some(Todo::default());
                                }

                                ui.text_edit_singleline(&mut self.edited_todo.as_mut().unwrap().heading);

                                ui.text_edit_singleline(&mut self.edited_todo.as_mut().unwrap().body);

                                for tag in self.edited_todo.as_mut().unwrap().tags.iter() {
                                    ui.label(&tag.0);
                                }

                                if ui.button("Create todo!").clicked() {
                                    let mut todo = None;
                                    std::mem::swap(&mut todo, &mut self.edited_todo);
                                    self.todos.push(todo.unwrap());
                                    self.mk_todo_dialog_shown = false;
                                }
                            });
                    }
                });
        });
    }
}
