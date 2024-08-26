use super::Component;
use crate::{
    action::Action,
    config::{Config, StyleName},
    structs::*,
};
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Default)]
pub struct TasksComponent {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    list: TasksList,
    input: String,
    character_index: usize,
    mode: Mode,
    sorter: TaskSorter,
    is_focused: bool,
    highlighted_item: (Option<usize>, String),
}

#[derive(Default, PartialEq, Eq)]
enum Mode {
    #[default]
    Normal,
    Insert(i32),
    Edit(i32),
}

#[derive(Default)]
struct TasksList {
    items: Vec<Task>,
    state: ListState,
}

impl TasksComponent {
    pub fn new() -> Self {
        Self::default()
    }

    fn select_next(&mut self) {
        match self.list.state.selected_mut() {
            Some(selected) => {
                *selected += 1;
                *selected %= self.list.items.len();
            }
            None => {
                self.list.state.select(Some(0));
            }
        }
    }

    fn select_previous(&mut self) {
        match self.list.state.selected_mut() {
            Some(selected) => {
                if *selected == 0 {
                    *selected = self.list.items.len() - 1;
                } else {
                    *selected -= 1;
                }
            }
            None => {
                self.list.state.select(Some(0));
            }
        }
    }

    fn select_bottom(&mut self) {
        self.list.state.select(Some(self.list.items.len() - 1));
    }

    fn select_top(&mut self) {
        self.list.state.select(Some(0));
    }

    fn submit(&mut self) -> Result<()> {
        let command_tx = self.command_tx.as_ref().unwrap();
        match self.mode {
            Mode::Insert(target) => {
                let t = AddTask {
                    name: self.input.trim().to_string(),
                    order: Some(target),
                    ..Default::default()
                };
                command_tx.send(Action::AddTask(t))?;
                command_tx.send(Action::LeaveInsertMode)?;
                self.input.clear();
                self.character_index = 0;
                self.mode = Mode::Normal;
            }
            Mode::Edit(target) => {
                let t = UpdateTask {
                    id: self.list.items[target as usize].id,
                    name: Some(self.input.trim().to_string()),
                    ..Default::default()
                };
                command_tx.send(Action::UpdateTask(t))?;
                command_tx.send(Action::LeaveInsertMode)?;
                self.input.clear();
                self.character_index = 0;
                self.mode = Mode::Normal;
            }
            _ => unreachable!(),
        };
        Ok(())
    }

    fn handle_insert_mode(&mut self, key: KeyEvent) -> Result<()> {
        let command_tx = self.command_tx.as_ref().unwrap();
        match key.code {
            KeyCode::Char(c) => {
                self.input.insert(self.character_index, c);
                self.character_index += 1;
            }
            KeyCode::Backspace => {
                if self.character_index > 0 {
                    self.character_index -= 1;
                    self.input.remove(self.character_index);
                }
            }
            KeyCode::Enter => {
                self.submit()?;
            }
            KeyCode::Esc => {
                command_tx.send(Action::LeaveInsertMode)?;
                self.input.clear();
                self.character_index = 0;
                self.mode = Mode::Normal;
            }
            KeyCode::Left => {
                if self.character_index > 0 {
                    self.character_index -= 1;
                }
            }
            KeyCode::Right => {
                if self.character_index < self.input.len() {
                    self.character_index += 1;
                }
            }
            _ => {}
        };
        Ok(())
    }
}

impl Component for TasksComponent {
    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn focus(&mut self, focus: bool) -> Result<()> {
        self.is_focused = focus;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<()> {
        let command_tx = self.command_tx.as_ref().unwrap().clone();
        match action {
            Action::NewTasksData(tasks) => {
                self.list.items = tasks;
                self.sorter.sort(&mut self.list.items);
                if !self.list.items.is_empty() && self.list.state.selected().is_none() {
                    self.list.state.select(Some(0));
                }
            }
            Action::GoUp => {
                self.select_previous();
            }
            Action::GoDown => {
                self.select_next();
            }
            Action::GoToTop => {
                self.select_top();
            }
            Action::GoToBottom => {
                self.select_bottom();
            }
            Action::SendKeyEvent(key) => match self.mode {
                Mode::Insert(_) | Mode::Edit(_) => {
                    self.handle_insert_mode(key)?;
                }
                _ => unreachable!(),
            },
            Action::AddItemAfter => {
                if let Some(selected) = self.list.state.selected_mut() {
                    self.mode = Mode::Insert(*selected as i32 + 1);
                    *selected += 1;
                } else {
                    self.mode = Mode::Insert(self.list.items.len() as i32);
                }
                command_tx.send(Action::EnterInsertMode)?;
            }
            Action::AddItemBefore => {
                if let Some(selected) = self.list.state.selected() {
                    self.mode = Mode::Insert(selected as i32);
                } else {
                    self.mode = Mode::Insert(0);
                }
                command_tx.send(Action::EnterInsertMode)?;
            }
            Action::DeleteItem => {
                if let Some(selected) = self.list.state.selected() {
                    command_tx.send(Action::RemoveWorkspace(self.list.items[selected].id))?;
                }
            }
            Action::EditItem => {
                if let Some(selected) = self.list.state.selected() {
                    self.mode = Mode::Edit(selected as i32);
                    self.input.clone_from(&self.list.items[selected].name);
                    self.character_index = self.input.len();
                    command_tx.send(Action::EnterInsertMode)?;
                }
            }
            Action::HighlightWorkspace(name) => {
                self.highlighted_item = (Some(9), name);
            }
            Action::Tick => match self.highlighted_item.0 {
                Some(0) => {
                    self.highlighted_item.0 = None;
                    self.highlighted_item.1.clear();
                }
                Some(n) => {
                    self.highlighted_item.0 = Some(n - 1);
                }
                None => {}
            },
            Action::MoveItemTop => {
                if let Some(selected) = self.list.state.selected() {
                    let w = UpdateWorkspace {
                        id: self.list.items[selected].id,
                        order: Some(0),
                        ..Default::default()
                    };
                    command_tx.send(Action::UpdateWorkspace(w))?;
                    self.select_top();
                }
            }
            Action::MoveItemUp => {
                if let Some(selected) = self.list.state.selected() {
                    if selected > 0 {
                        let w = UpdateWorkspace {
                            id: self.list.items[selected].id,
                            order: Some((selected - 1) as i32),
                            ..Default::default()
                        };
                        command_tx.send(Action::UpdateWorkspace(w))?;
                        self.select_previous();
                    }
                }
            }
            Action::MoveItemDown => {
                if let Some(selected) = self.list.state.selected() {
                    if selected < self.list.items.len() - 1 {
                        let w = UpdateWorkspace {
                            id: self.list.items[selected].id,
                            order: Some((selected + 1) as i32),
                            ..Default::default()
                        };
                        command_tx.send(Action::UpdateWorkspace(w))?;
                        self.select_next();
                    }
                }
            }
            Action::MoveItemBottom => {
                if let Some(selected) = self.list.state.selected() {
                    let w = UpdateWorkspace {
                        id: self.list.items[selected].id,
                        order: Some(self.list.items.len() as i32),
                        ..Default::default()
                    };
                    command_tx.send(Action::UpdateWorkspace(w))?;
                    self.select_bottom();
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        todo!()
    }
}
