use super::Component;
use crate::{
    action::Action,
    config::{Config, StyleName},
    structs::*,
};
use color_eyre::{eyre::Ok, Result};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use std::{
    cmp::{max, min},
    collections::HashMap,
};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Default, Debug)]
pub struct TasksComponent {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    list: TasksTable,
    selected_workspace: Option<i32>,
    input: String,
    character_index: usize,
    mode: Mode,
    sorter: TaskSorter,
    is_focused: bool,
    highlighted_item: (Option<usize>, String),
    to_be_selected: Option<usize>,
    last_selected: HashMap<i32, usize>,
}

#[derive(Default, PartialEq, Eq, Debug)]
enum Mode {
    #[default]
    Normal,
    Insert(usize),
    Edit(usize),
    EditDescription(usize),
}

#[derive(Default, Debug)]
struct TasksTable {
    items: Vec<Task>,
    state: TableState,
}

impl Task {
    fn to_row(
        &self,
        config: &Config,
        highlighting: &(Option<usize>, String),
        override_name: Option<String>,
        override_desc: Option<String>,
    ) -> Row {
        let prioritys = vec![
            Cell::from(
                Text::raw("1")
                    .style(config.styles[&StyleName::Priority1])
                    .alignment(Alignment::Center),
            ),
            Cell::from(
                Text::raw("2")
                    .style(config.styles[&StyleName::Priority2])
                    .alignment(Alignment::Center),
            ),
            Cell::from(
                Text::raw("3")
                    .style(config.styles[&StyleName::Priority3])
                    .alignment(Alignment::Center),
            ),
            Cell::from(
                Text::raw("4")
                    .style(config.styles[&StyleName::Priority4])
                    .alignment(Alignment::Center),
            ),
        ];
        let error_style = config.styles[&StyleName::Error];
        let completed_style = config.styles[&StyleName::Completed];
        let check_cell = if self.completed {
            Cell::from(" ✓").style(completed_style)
        } else {
            Cell::from(" ☐")
        };
        let name = override_name.unwrap_or(self.name.clone());
        let description = override_desc.unwrap_or(self.description.clone());
        let mut name_cell = if self.completed {
            Cell::from(name).style(completed_style)
        } else {
            Cell::from(name)
        };

        if highlighting.0.is_some() && highlighting.1 == self.name {
            name_cell = name_cell.style(error_style);
        }
        Row::new(vec![
            check_cell,
            name_cell,
            prioritys[self.priority as usize - 1].clone(),
            description.into(),
        ])
    }
}

impl TasksComponent {
    pub fn new() -> Self {
        Self::default()
    }

    fn secure_selction(&mut self) {
        if let Some(index) = self.list.state.selected() {
            if self.list.items.is_empty() {
                self.list.state.select(None);
            } else {
                let index = min(index, self.list.items.len() - 1);
                self.list.state.select(Some(index));
            }
        }
    }

    fn mark_task(&mut self) {
        if let Some(selected) = self.list.state.selected() {
            let t = UpdateTask {
                id: self.list.items[selected].id,
                completed: Some(!self.list.items[selected].completed),
                ..Default::default()
            };
            self.command_tx
                .as_ref()
                .unwrap()
                .send(Action::UpdateTask(t))
                .unwrap();
        }
    }

    fn on_select(&mut self) {
        if let Some(current_workspace) = self.selected_workspace {
            match self.list.state.selected() {
                Some(selected) => {
                    self.last_selected.insert(current_workspace, selected);
                }
                None => {
                    self.last_selected.remove(&current_workspace);
                }
            }
        }
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
        self.on_select();
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
        self.on_select();
    }

    fn select_bottom(&mut self) {
        if self.list.items.is_empty() {
            return;
        }
        self.list.state.select(Some(self.list.items.len() - 1));
        self.on_select();
    }

    fn select_top(&mut self) {
        if self.list.items.is_empty() {
            return;
        }
        self.list.state.select(Some(0));
        self.on_select();
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
            Mode::EditDescription(target) => {
                let t = UpdateTask {
                    id: self.list.items[target].id,
                    description: Some(self.input.trim().to_string()),
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

    fn increase_priority(&self) -> Result<()> {
        if let Some(selected) = self.list.state.selected() {
            let t = UpdateTask {
                id: self.list.items[selected].id,
                priority: Some(min(self.list.items[selected].priority + 1, 4)),
                ..Default::default()
            };
            self.command_tx
                .as_ref()
                .unwrap()
                .send(Action::UpdateTask(t))?;
        }
        Ok(())
    }

    fn decrease_priority(&self) -> Result<()> {
        if let Some(selected) = self.list.state.selected() {
            let t = UpdateTask {
                id: self.list.items[selected].id,
                priority: Some(max(self.list.items[selected].priority - 1, 1)),
                ..Default::default()
            };
            self.command_tx
                .as_ref()
                .unwrap()
                .send(Action::UpdateTask(t))?;
        }
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
                self.secure_selction();
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

                // making sure the database is not fucking up the order(again).
                let mut iter = self.list.items.iter();
                if let Some(mut last_order) = iter.next().map(|t| t.order) {
                    for task in iter {
                        assert!(
                            task.order == last_order + 1,
                            "the order is not as expected at workspace {} because we found orders:{},{}",
                            workspace_id,
                            task.order,
                            last_order
                        );
                        last_order = task.order;
                    }
                }

                self.selected_workspace = Some(workspace_id);

                // selection handling
                if self.list.items.is_empty() {
                    self.list.state.select(None);
                } else if let Some(index) = self.to_be_selected {
                    let index = min(index, self.list.items.len() - 1);
                    self.list.state.select(Some(index));
                    self.to_be_selected = None;
                } else if let Some(last_selection) = self.last_selected.get(&workspace_id) {
                    let last_selection = min(*last_selection, self.list.items.len() - 1);
                    self.list.state.select(Some(last_selection));
                } else if self.list.state.selected().is_none() {
                    self.list.state.select(Some(0));
                } else if let Some(selected) = self.list.state.selected_mut() {
                    // making sure no out of bounds
                    *selected = min(*selected, self.list.items.len() - 1);
                }

                self.on_select();
            }
            Action::UnselectWorkspace => {
                self.list.items.clear();
                self.selected_workspace = None;
                self.list.state.select(None);
                self.on_select();
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
            Action::IncreasePriority => {
                self.increase_priority()?;
            }
            Action::DecreasePriority => {
                self.decrease_priority()?;
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
                if self.selected_workspace.is_none() {
                    return Ok(());
                }
                if let Some(selected) = self.list.state.selected() {
                    self.mode = Mode::Insert(selected + 1);
                } else {
                    self.mode = Mode::Insert(self.list.items.len());
                }
                command_tx.send(Action::EnterInsertMode)?;
            }
            Action::AddItemBefore => {
                if self.selected_workspace.is_none() {
                    return Ok(());
                }
                if let Some(selected) = self.list.state.selected() {
                    self.mode = Mode::Insert(selected);
                } else {
                    self.mode = Mode::Insert(0);
                }
                command_tx.send(Action::EnterInsertMode)?;
            }
            Action::ToggleCompletion => {
                self.mark_task();
            }
            Action::DeleteItem => {
                if let Some(selected) = self.list.state.selected() {
                    command_tx.send(Action::RemoveTask(self.list.items[selected].id))?;
                }
            }
            Action::EditDescription => {
                if let Some(selected) = self.list.state.selected() {
                    let t = &self.list.items[selected];
                    self.mode = Mode::EditDescription(selected);
                    self.input.clone_from(&t.description);
                    self.character_index = self.input.len();
                    command_tx.send(Action::EnterInsertMode)?;
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
                        order: Some(self.list.items.len() - 1),
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
        let block_style = if self.is_focused {
            self.config.styles[&StyleName::Highlight]
        } else {
            Style::default()
        };

        let block = Block::default()
            .title("Tasks")
            .border_style(block_style)
            .border_type(BorderType::Thick)
            .borders(Borders::ALL);

        if self.selected_workspace.is_none() {
            let paragraph = Paragraph::new("No workspace selected")
                .block(block)
                .alignment(Alignment::Center);
            frame.render_widget(paragraph, area);
            return Ok(());
        }

        let mut columns_sizes: (u16, u16, u16) = (4, 8, 11);
        let mut items: Vec<Row> = self
            .list
            .items
            .iter()
            .map(|t| {
                columns_sizes.0 = columns_sizes.0.max(t.name.len() as u16);
                columns_sizes.1 = columns_sizes.1.max(t.priority.to_string().len() as u16);
                columns_sizes.2 = columns_sizes.2.max(t.description.len() as u16);
                t.to_row(&self.config, &self.highlighted_item, None, None)
            })
            .collect();

        let cursor_offset = if matches!(self.mode, Mode::EditDescription(..)) {
            columns_sizes.2 = columns_sizes.2.max(self.input.len() as u16);
            9 + columns_sizes.0 + columns_sizes.1
        } else {
            columns_sizes.0 = columns_sizes.0.max(self.input.len() as u16);
            7
        };

        match self.mode {
            Mode::Insert(target) => {
                items.insert(
                    target,
                    Row::new(vec![
                        Cell::from(" ☐"),
                        Cell::from(self.input.clone()),
                        Cell::from(Text::raw("3").alignment(Alignment::Center)),
                        Cell::default(),
                    ]),
                );
                self.list.state.select(Some(target));
            }
            Mode::Edit(target) => {
                items[target] = self.list.items[target].to_row(
                    &self.config,
                    &self.highlighted_item,
                    Some(self.input.clone()),
                    None,
                );
                self.list.state.select(Some(target));
            }
            Mode::EditDescription(target) => {
                items[target] = self.list.items[target].to_row(
                    &self.config,
                    &self.highlighted_item,
                    None,
                    Some(self.input.clone()),
                );
                self.list.state.select(Some(target));
            }
            _ => {}
        }

        let widths = [
            Constraint::Length(3),
            Constraint::Length(columns_sizes.0),
            Constraint::Length(columns_sizes.1),
            Constraint::Length(columns_sizes.2),
        ];

        let table = Table::new(items, widths)
            .block(block)
            .highlight_style(selected_style)
            .highlight_symbol(">>")
            .highlight_spacing(HighlightSpacing::Always)
            .header(
                Row::new(vec!["", "Name", "Priority", "Description"])
                    .style(block_style)
                    .add_modifier(Modifier::REVERSED)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_stateful_widget(table, area, &mut self.list.state);
        if let Mode::Insert(line) | Mode::Edit(line) | Mode::EditDescription(line) = self.mode {
            frame.set_cursor(
                area.x + self.character_index as u16 + cursor_offset,
                area.y + line as u16 + 2,
            );
        }

        Ok(())
    }
}
