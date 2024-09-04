use super::Component;
use crate::{
    action::Action,
    config::{Config, StyleName},
    structs::*,
};
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use std::cmp::min;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Default, Debug)]
pub struct TasksComponent {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    list: TasksList,
    selected_workspace: Option<i32>,
    input: String,
    character_index: usize,
    mode: Mode,
    sorter: TaskSorter,
    is_focused: bool,
    highlighted_item: (Option<usize>, String),
    to_be_selected: Option<usize>,
}

#[derive(Default, PartialEq, Eq, Debug)]
enum Mode {
    #[default]
    Normal,
    Insert(usize),
    Edit(usize),
}

#[derive(Default, Debug)]
struct TasksList {
    items: Vec<Task>,
    state: ListState,
}

impl TasksComponent {
    pub fn new() -> Self {
        Self::default()
    }

    fn select_next(&mut self) {
        if self.list.items.is_empty() {
            return;
        }
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
        if self.list.items.is_empty() {
            return;
        }
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
        if self.list.items.is_empty() {
            return;
        }
        self.list.state.select(Some(self.list.items.len() - 1));
    }

    fn select_top(&mut self) {
        if self.list.items.is_empty() {
            return;
        }
        self.list.state.select(Some(0));
    }

    fn submit(&mut self) -> Result<()> {
        let command_tx = self.command_tx.as_ref().unwrap();
        match self.mode {
            Mode::Insert(target) => {
                let t = AddTask {
                    name: self.input.trim().to_string(),
                    order: Some(target),
                    workspace_id: self.selected_workspace.unwrap(),
                    ..Default::default()
                };
                command_tx.send(Action::AddTask(t))?;
                command_tx.send(Action::LeaveInsertMode)?;
                self.to_be_selected = Some(target);
                self.input.clear();
                self.character_index = 0;
                self.mode = Mode::Normal;
            }
            Mode::Edit(target) => {
                let t = UpdateTask {
                    id: self.list.items[target].id,
                    name: Some(self.input.trim().to_string()),
                    ..Default::default()
                };
                command_tx.send(Action::UpdateTask(t))?;
                command_tx.send(Action::LeaveInsertMode)?;
                self.to_be_selected = Some(target);
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
            Action::NewTasksData((tasks, workspace_id)) => {
                self.list.items = tasks;
                self.sorter.sort(&mut self.list.items);
                self.selected_workspace = Some(workspace_id);

                // selection handling
                if self.list.items.is_empty() {
                    self.list.state.select(None);
                } else if let Some(index) = self.to_be_selected {
                    let index = min(index, self.list.items.len() - 1);
                    self.list.state.select(Some(index));
                    self.to_be_selected = None;
                } else if self.list.state.selected().is_none() {
                    self.list.state.select(Some(0));
                } else if let Some(selected) = self.list.state.selected_mut() {
                    // making sure no out of bounds
                    *selected = min(*selected, self.list.items.len() - 1);
                }
            }
            Action::UnselectWorkspace => {
                self.list.items.clear();
                self.selected_workspace = None;
                self.list.state.select(None);
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
            Action::SendKeyEvent(key) => {
                assert_ne!(
                    self.mode,
                    Mode::Normal,
                    "the Component is in normal while app is in edit mode"
                );
                self.handle_insert_mode(key)?;
            }
            Action::AddItemAfter => {
                if let Some(selected) = self.list.state.selected() {
                    self.mode = Mode::Insert(selected + 1);
                } else {
                    self.mode = Mode::Insert(self.list.items.len());
                }
                command_tx.send(Action::EnterInsertMode)?;
            }
            Action::AddItemBefore => {
                if let Some(selected) = self.list.state.selected() {
                    self.mode = Mode::Insert(selected);
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
                    self.mode = Mode::Edit(selected);
                    self.input.clone_from(&self.list.items[selected].name);
                    self.character_index = self.input.len();
                    command_tx.send(Action::EnterInsertMode)?;
                }
            }
            Action::HighlightTask(name) => {
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
                    let t = UpdateTask {
                        id: self.list.items[selected].id,
                        order: Some(0),
                        ..Default::default()
                    };
                    command_tx.send(Action::UpdateTask(t))?;
                    self.select_top();
                }
            }
            Action::MoveItemUp => {
                if let Some(selected) = self.list.state.selected() {
                    if selected > 0 {
                        let t = UpdateTask {
                            id: self.list.items[selected].id,
                            order: Some(selected - 1),
                            ..Default::default()
                        };
                        command_tx.send(Action::UpdateTask(t))?;
                        self.select_previous();
                    }
                }
            }
            Action::MoveItemDown => {
                if let Some(selected) = self.list.state.selected() {
                    if selected < self.list.items.len() - 1 {
                        let t = UpdateTask {
                            id: self.list.items[selected].id,
                            order: Some(selected + 1),
                            ..Default::default()
                        };
                        command_tx.send(Action::UpdateTask(t))?;
                        self.select_next();
                    }
                }
            }
            Action::MoveItemBottom => {
                if let Some(selected) = self.list.state.selected() {
                    let t = UpdateTask {
                        id: self.list.items[selected].id,
                        order: Some(self.list.items.len()),
                        ..Default::default()
                    };
                    command_tx.send(Action::UpdateTask(t))?;
                    self.select_bottom();
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let selected_style = self.config.styles[&StyleName::Selected];
        let error_style = self.config.styles[&StyleName::Error];
        let block_style = if self.is_focused {
            self.config.styles[&StyleName::Highlight]
        } else {
            Style::default()
        };

        let block = Block::default()
            .title("Tasks")
            .border_style(block_style)
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        if self.selected_workspace.is_none() {
            let paragraph = Paragraph::new("No workspace selected")
                .block(block)
                .alignment(Alignment::Center);
            frame.render_widget(paragraph, area);
            return Ok(());
        }

        let mut items: Vec<ListItem> = self
            .list
            .items
            .iter()
            .map(|t| {
                if self.highlighted_item.0.is_some() && self.highlighted_item.1 == t.name {
                    ListItem::new(Line::from(t.name.clone()).style(error_style))
                } else {
                    ListItem::new(Line::from(t.name.clone()))
                }
            })
            .collect();

        match self.mode {
            Mode::Insert(target) => {
                items.insert(target, ListItem::new(self.input.clone()));
                self.list.state.select(Some(target));
            }
            Mode::Edit(target) => {
                items[target] = ListItem::new(self.input.clone());
                self.list.state.select(Some(target));
            }
            _ => {}
        }

        let items = List::new(items)
            .block(block)
            .highlight_style(selected_style);

        frame.render_stateful_widget(items, area, &mut self.list.state);
        if let Mode::Insert(line) | Mode::Edit(line) = self.mode {
            frame.set_cursor(
                area.x + 1 + self.character_index as u16,
                area.y + line as u16 + 1,
            );
        }
        Ok(())
    }
}
