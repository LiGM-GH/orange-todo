pub mod todo;
pub mod todo_editor;

use std::{
    mem,
    time::{Duration, Instant},
};

use self::{
    todo::{Id, PartialTodo, Todo, TodoError},
    todo_editor::TodoEditor,
};

use crate::db::{self, Config};
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
    removed_todos: Vec<Id>,
    todo_to_remove: Option<Id>,
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
            image: RetainedImage::from_image_bytes("orange.png", include_bytes!("orange.png"))
                .expect("Couldn't find image 'orange.jpg' which is by default in src/ dir."),
            removed_todos: Vec::new(),
            todo_to_remove: None,
        }
    }
}

impl TodoApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut default = Self::default();
        default.read_from_db().expect("Couldn't read from db");
        default
    }

    fn make_editor(&mut self, ui: &mut Ui) {
        Frame::window(&Style::default())
            .fill(EDITOR_COLOR)
            .show(ui, |ui| {
                if self.edit.todo_maker.todo.is_none() {
                    self.edit.todo_maker.todo = Some(match self.todos.iter().last() {
                        Some(val) => PartialTodo::new(val.id().clone() + 1),
                        _ => PartialTodo::new(1 as i32),
                    });
                }

                {
                    ui.text_edit_singleline(
                        &mut self.edit.todo_maker.todo.as_mut().unwrap().heading,
                    );

                    ui.text_edit_singleline(&mut self.edit.todo_maker.todo.as_mut().unwrap().body);
                }

                for tag in self.edit.todo_maker.todo.as_mut().unwrap().tags.iter() {
                    ui.label(tag);
                }

                if ui.button("Create todo!").clicked() {
                    self.edit.todo_maker.save_result = match Todo::try_from(
                        self.edit
                            .todo_maker
                            .todo
                            .clone()
                            .expect("Should have created todo first..."),
                    ) {
                        Ok(val) => {
                            self.todos.push(val);
                            self.edit.todo_maker.todo = None;
                            self.edit.show_todo_maker = false;
                            Ok(())
                        }
                        Err(err) => Err(err),
                    };
                };

                match &self.edit.todo_maker.save_result {
                    Err(TodoError::EmptyBody) => {
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

    fn show_all_todos(&mut self, ui: &mut Ui) {
        if self.todo_to_remove.is_some() {
            match self
                .todos
                .iter()
                .position(|e| *e.id() == self.todo_to_remove.unwrap())
            {
                Some(val) => {
                    self.todos.remove(val);
                    self.removed_todos.push(self.todo_to_remove.unwrap());
                }
                _ => {}
            }

            self.todo_to_remove = None;
        }

        for (i, todo) in self.todos.iter_mut().enumerate() {
            let (todo_check_clicked, todo_icon_clicked, todo_remove_clicked) =
                make_todo_edit(ui, &todo);

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

            if todo_remove_clicked {
                self.todo_to_remove = Some(todo.id().clone());
            }

            if todo_icon_clicked {
                log::trace!("Edit-todo dialog shown");

                if self.edit.button_switch_timer.is_none() {
                    self.edit.button_switch_timer = Some(Instant::now());
                }

                if self.edit.todo_bound_with_editor.is_none() {
                    self.edit.todo_bound_with_editor = Some(i);
                    self.edit.todo_editor.todo = Some(Clone::clone(todo).into());
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
                                        let edition =
                                            TryInto::<Todo>::try_into(edited_todo.clone());

                                        match edition {
                                            Ok(mut val) => {
                                                mem::swap(&mut val, todo);
                                                self.edit.todo_editor.todo = None;
                                                Ok(())
                                            }
                                            Err(err) => Err(err),
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
                                        Err(TodoError::EmptyBody) => {
                                            log::info!("Tried to remove todo's body!");
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            match self.edit.todo_editor.save_result {
                                Err(TodoError::EmptyBody) => {
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

    fn read_from_db(&mut self) -> Result<(), anyhow::Error> {
        let mut client = set_client()?;

        for row in client.query("SELECT * FROM todo", &[])? {
            match Todo::try_from(PartialTodo {
                id: row.try_get(0)?,
                heading: row.try_get(1)?,
                body: row.try_get(2)?,
                checked: row.try_get(3)?,
                tags: match row.try_get::<usize, Vec<String>>(6) {
                    Ok(val) => val,
                    Err(_) => Vec::new(),
                },
            }) {
                Ok(val) => self.todos.push(val),
                Err(_) => {}
            };
        }

        Ok(())
    }

    fn save_into_db(&self) -> Result<(), anyhow::Error> {
        let mut client = set_client()?;

        for id in self.removed_todos.iter() {
            client.execute("DELETE FROM todo WHERE id=$1", &[id])?;
        }

        for todo in self.todos.iter() {
            let part: PartialTodo = todo.clone().into();

            log::trace!("{:?}", part);

            log::trace!(
                "UPDATE todo SET heading = {}, body = {}, checked = {} WHERE id = {}",
                part.heading,
                part.body,
                part.checked,
                part.id,
            );

            let mut transaction = client.transaction()?;

            if let Ok(_) =
                transaction.query_one("SELECT * FROM todo WHERE id=$1 LIMIT 1", &[&part.id])
            {
                transaction.execute(
                    "UPDATE todo SET heading = $1, body = $2, checked = $3 WHERE id = $4",
                    &[&part.heading, &part.body, &part.checked, &part.id],
                )?;
            } else {
                transaction.execute(
                    "INSERT INTO todo (id, heading, body, checked) VALUES ($1, $2, $3, $4)",
                    &[&part.id, &part.heading, &part.body, &part.checked],
                )?;
            }

            transaction.commit()?;
        }

        Ok(())
    }
}

impl eframe::App for TodoApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Orange To Do - a minimalistic to do app");

            ScrollArea::both()
                .auto_shrink([false, false])
                .always_show_scroll(false)
                .show(ui, |ui| {
                    self.image.show_scaled(ui, 0.3);

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

fn display_tags(todo: &PartialTodo, ui: &mut Ui) {
    for tag in todo.tags.iter() {
        ui.label(tag);
    }
}

fn set_client() -> Result<Client, db::set::Error> {
    use db::set::Error;

    let password = std::fs::read_to_string("./secrets.toml").map_err(|_| Error::ReadConfig)?;
    let config: Config = toml::de::from_str(&password).map_err(|_| Error::ReadConfig)?;

    let connect_config = &format!(
        "host=localhost port=5432 user={} password={}",
        config.db.user, config.db.password
    );

    Ok(Client::connect(&connect_config, NoTls).map_err(|_| Error::SetupDb)?)
}

fn make_todo_edit(ui: &mut Ui, todo: &&mut Todo) -> (bool, bool, bool) {
    let mut todo_check_clicked: bool = false;
    let mut todo_icon_clicked: bool = false;
    let mut todo_remove_clicked: bool = false;

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
                let text = todo.heading();

                let rich = if todo.checked {
                    RichText::from(text).strikethrough()
                } else {
                    RichText::from(text)
                };

                rich
            })
            .clicked();
        todo_remove_clicked = ui.button(RichText::from("🗑️").color(Color32::RED)).clicked();
    });

    (todo_check_clicked, todo_icon_clicked, todo_remove_clicked)
}
