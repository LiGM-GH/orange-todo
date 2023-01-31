pub mod add_todo_error;
pub mod todo;
pub mod todo_editor;

use std::{
    mem,
    time::{Duration, Instant},
};

use self::{
    add_todo_error::AddTodoError,
    todo::{Tag, Todo},
    todo_editor::TodoEditor,
};
use crate::db::Config;
use egui::{Align, CentralPanel, Color32, Frame, Layout, RichText, ScrollArea, Style, Ui};
use egui_extras::RetainedImage;
use postgres::{Client, NoTls};

const EDITOR_COLOR: Color32 = Color32::from_rgb(250, 100, 51);
const EDITOR_WARNING_COLOR: Color32 = Color32::WHITE;
const BUTTON_SWITCH_DURATION: Duration = Duration::from_millis(100);
const CHECKED_TODO_MARK_COLOR: Color32 = Color32::GREEN;
const UNCHECKED_TODO_MARK_COLOR: Color32 = Color32::RED;
const LABEL_WIDTH: f32 = 20.0;

pub struct TodoApp {
    todos: Vec<Todo>,
    on_close: OnClose,
    edit: TodoEditing,
    image: egui_extras::RetainedImage,
}

#[derive(Default, Debug)]
struct TodoEditing {
    show_todo_maker: bool,
    todo_maker: TodoEditor,
    todo_editor: TodoEditor,
    todo_bound_with_editor: Option<usize>,
    button_switch_timer: Option<std::time::Instant>,
}

#[derive(Default, Debug)]
struct OnClose {
    allowed_to_close: bool,
    on_close_dialog: bool,
}

impl Default for TodoApp {
    fn default() -> Self {
        Self {
            todos: Vec::new(),
            on_close: OnClose::default(),
            edit: TodoEditing::default(),
            image: RetainedImage::from_image_bytes("orange.jpg", include_bytes!("orange.jpg"))
                .expect("Couldn't find image 'orange.jpg' which is by default in src/ dir."),
        }
    }
}

impl TodoApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut default = Self::default();
        default.read_from_db().expect("Couldn't read from db");
        default
    }

    pub fn insert_todo(&mut self, todo: Todo) {
        self.todos.push(todo);
    }

    fn add_todo(&mut self) -> Result<(), AddTodoError> {
        if let Some(todo) = self.edit.todo_maker.todo.as_ref() {
            if !todo.body.is_empty() {
                let mut todo = None;

                // &mut self... should be dropped as fast as possible
                {
                    std::mem::swap(&mut todo, &mut self.edit.todo_maker.todo);
                }

                if let Some(todo) = todo {
                    self.todos.push(todo);
                }

                self.edit.show_todo_maker = false;
                Ok(())
            } else {
                Err(AddTodoError::EmptyBody)
            }
        } else {
            Err(AddTodoError::NoCurrentTodo)
        }
    }

    fn make_editor(&mut self, ui: &mut Ui) {
        Frame::window(&Style::default())
            .fill(EDITOR_COLOR)
            .show(ui, |ui| {
                if self.edit.todo_maker.todo.is_none() {
                    self.edit.todo_maker.todo = Some(Todo::default());
                }

                {
                    ui.text_edit_singleline(
                        &mut self.edit.todo_maker.todo.as_mut().unwrap().heading,
                    );

                    ui.text_edit_singleline(&mut self.edit.todo_maker.todo.as_mut().unwrap().body);
                }

                for tag in self.edit.todo_maker.todo.as_mut().unwrap().tags.iter() {
                    ui.label(&tag.0);
                }

                if ui.button("Create todo!").clicked() {
                    self.edit.todo_maker.save_result = self.add_todo();
                }

                match &self.edit.todo_maker.save_result {
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

    fn show_all_todos(&mut self, mut ui: &mut Ui) {
        for (i, todo) in self.todos.iter_mut().enumerate() {
            let (todo_check_clicked, todo_icon_clicked) = make_todo_edit(&mut ui, &todo);

            if todo_check_clicked {
                todo.checked = !todo.checked;

                match self.edit.todo_bound_with_editor {
                    Some(num) if num == i => {
                        self.edit.todo_editor.todo.as_mut().unwrap().checked =
                            !self.edit.todo_editor.todo.as_ref().unwrap().checked;
                    }
                    _ => {}
                }
            }

            if todo_icon_clicked {
                log::trace!("Edit-todo dialog shown");

                if self.edit.button_switch_timer.is_none() {
                    self.edit.button_switch_timer = Some(Instant::now());
                }

                if self.edit.todo_bound_with_editor.is_none() {
                    self.edit.todo_bound_with_editor = Some(i);
                    self.edit.todo_editor.todo = Some(Clone::clone(todo));
                } else {
                    log::trace!("{:#?}", self.edit.button_switch_timer);
                    if let Some(timer) = self.edit.button_switch_timer {
                        log::trace!(
                            "Comparison ended with result {}",
                            timer.elapsed() >= BUTTON_SWITCH_DURATION
                        );
                        if timer.elapsed() >= BUTTON_SWITCH_DURATION {
                            log::trace!("Elapsed {} millis", timer.elapsed().as_millis());
                            self.edit.todo_bound_with_editor = None;
                            self.edit.button_switch_timer = None;
                        };
                    }
                }
            }

            match self.edit.todo_bound_with_editor {
                Some(num) if num == i => {
                    Frame::window(&Style::default())
                        .fill(EDITOR_COLOR)
                        .show(ui, |ui| {
                            if let Some(edited_todo) = self.edit.todo_editor.todo.as_mut() {
                                ui.text_edit_singleline(&mut edited_todo.heading);

                                ui.text_edit_multiline(&mut edited_todo.body);

                                ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                                    display_tags(edited_todo, ui);

                                    let mut new_label = String::new();

                                    ui.add(
                                        egui::TextEdit::singleline(&mut new_label)
                                            .desired_width(LABEL_WIDTH),
                                    )
                                });

                                if ui.button("Save todo!").clicked() {
                                    self.edit.todo_editor.save_result = {
                                        if edited_todo.body.is_empty() {
                                            Err(AddTodoError::EmptyBody)
                                        } else {
                                            mem::swap(edited_todo, todo);
                                            Ok(())
                                        }
                                    };

                                    log::trace!(
                                        "Editor save result: {:?}",
                                        self.edit.todo_editor.save_result
                                    );
                                    match self.edit.todo_editor.save_result {
                                        Ok(_) => {
                                            self.edit.todo_editor.todo = None;
                                            self.edit.todo_bound_with_editor = None;
                                        }
                                        Err(AddTodoError::EmptyBody) => {
                                            log::info!("Tried to remove todo's body!");
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            match self.edit.todo_editor.save_result {
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

    fn read_from_db(&mut self) -> Result<(), String> {
        let mut client = set_client()?;
        for row in client
            .query("SELECT * FROM todo", &[])
            .map_err(|err| format!("read_from_db: {}", err.to_string()))?
        {
            self.todos.push(Todo {
                heading: row
                    .try_get(1)
                    .map_err(|err| format!("read_from_db: row: {}", err.to_string()))?,
                body: row
                    .try_get(2)
                    .map_err(|err| format!("read_from_db: row: {}", err.to_string()))?,
                checked: row
                    .try_get(3)
                    .map_err(|err| format!("read_from_db: row: {}", err.to_string()))?,
                tags: match row
                    .try_get::<usize, Vec<String>>(6)
                    .map_err(|err| format!("read_from_db: row: {}", err.to_string()))
                {
                    Ok(val) => val.into_iter().map(|tagname| Tag(tagname)).collect(),
                    Err(_) => Vec::new(),
                },
            });
        }
        Ok(())
    }

    fn save_into_db(&self) -> Result<(), String> {
        let mut client = set_client()?;
        for todo in self.todos.iter() {
            client
                .execute(
                    "INSERT INTO todo (heading, body, checked) VALUES ($1, $2, $3)",
                    &[&todo.heading, &todo.body, &todo.checked],
                )
                .map_err(|err| format!("execute: {}", err.to_string()))?;
        }

        Ok(())
    }
}

impl eframe::App for TodoApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Orange To Do - a minimalistic to do app");
            self.image.show_size(ui, egui::Vec2 { x: 10.0, y: 10.0 });

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .always_show_scroll(false)
                .show(ui, |ui| {
                    self.show_all_todos(ui);

                    if ui.button("Show make-todo-dialog").clicked() {
                        self.edit.show_todo_maker = !self.edit.show_todo_maker;

                        if self.edit.show_todo_maker {
                            log::trace!("mk_todo_dialog shown")
                        } else {
                            log::trace!("mk_todo_dialog hidden")
                        }
                    }

                    if self.edit.show_todo_maker {
                        self.make_editor(ui);
                    }
                });
        });

        if self.on_close.on_close_dialog {
            // Show confirmation dialog:
            egui::Window::new("Error occured while saving. Quit?")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.on_close.on_close_dialog = false;
                        }

                        if ui.button("Yes!").clicked() {
                            self.on_close.allowed_to_close = true;
                            frame.close();
                        }
                    });
                });
        }
    }

    fn on_close_event(&mut self) -> bool {
        match self.save_into_db() {
            Err(err) => {
                log::trace!("Error occured while saving: {}", err.to_string());
                self.on_close.on_close_dialog = true;
            }
            _ => {
                log::trace!("Save executed successfully!");
                self.on_close.allowed_to_close = true;
            }
        };

        self.on_close.allowed_to_close
    }
}

fn display_tags(edited_todo: &Todo, ui: &mut Ui) {
    for tag in edited_todo.tags.iter() {
        ui.label(&tag.0);
    }
}

fn set_client() -> Result<Client, String> {
    let password = std::fs::read_to_string("./secrets.toml").map_err(|err| err.to_string())?;
    let config: Config = toml::de::from_str(&password).map_err(|err| err.to_string())?;

    let connect_config = &format!(
        "host=localhost port=5432 user={} password={}",
        config.db.user, config.db.password
    );

    Ok(Client::connect(&connect_config, NoTls)
        .map_err(|err| format!("ClientConnect: {}", err.to_string()))?)
}

fn make_todo_edit(ui: &mut &mut Ui, todo: &&mut Todo) -> (bool, bool) {
    let mut todo_check_clicked: bool = false;
    let mut todo_icon_clicked: bool = false;

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
            .clicked();
    });

    (todo_check_clicked, todo_icon_clicked)
}
