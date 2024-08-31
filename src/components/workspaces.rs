use std::cmp::min;

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
pub struct WorkspacesComponent {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    list: WorkspacesList,
    input: String,
    character_index: usize,
    mode: Mode,
    sorter: WorkspaceSorter,
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
struct WorkspacesList {
    items: Vec<Workspace>,
    state: ListState,
}

impl WorkspacesComponent {
    pub fn new() -> Self {
        Self::default()
    }

    fn send_workspace_id(&self) -> Result<()> {
        if let Some(selected) = self.list.state.selected() {
            let workspace_id = self.list.items[selected].id;
            let command_tx = self.command_tx.as_ref().unwrap();
            command_tx.send(Action::SelectWorkspace(workspace_id))?;
        }
        Ok(())
    }

    fn select_next(&mut self) -> Result<()> {
        match self.list.state.selected_mut() {
            Some(selected) => {
                *selected += 1;
                *selected %= self.list.items.len();
            }
            None => {
                self.list.state.select(Some(0));
            }
        }
        self.send_workspace_id()?;
        Ok(())
    }

    fn select_previous(&mut self) -> Result<()> {
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
        self.send_workspace_id()?;
        Ok(())
    }

    fn select_bottom(&mut self) -> Result<()> {
        self.list.state.select(Some(self.list.items.len() - 1));
        self.send_workspace_id()?;
        Ok(())
    }

    fn select_top(&mut self) -> Result<()> {
        self.list.state.select(Some(0));
        self.send_workspace_id()?;
        Ok(())
    }

    fn submit(&mut self) -> Result<()> {
        let command_tx = self.command_tx.as_ref().unwrap();
        match self.mode {
            Mode::Insert(target) => {
                let w = AddWorkspace {
                    name: self.input.trim().to_string(),
                    order: Some(target),
                };
                command_tx.send(Action::AddWorkspace(w))?;
                command_tx.send(Action::LeaveInsertMode)?;
                self.input.clear();
                self.character_index = 0;
                self.mode = Mode::Normal;
            }
            Mode::Edit(target) => {
                let w = UpdateWorkspace {
                    id: self.list.items[target as usize].id,
                    name: Some(self.input.trim().to_string()),
                    ..Default::default()
                };
                command_tx.send(Action::UpdateWorkspace(w))?;
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

impl Component for WorkspacesComponent {
    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn init(&mut self, _area: Rect) -> Result<()> {
        self.command_tx
            .as_ref()
            .unwrap()
            .send(Action::RequestWorkspacesData)?;
        Ok(())
    }

    fn focus(&mut self, focus: bool) -> Result<()> {
        self.is_focused = focus;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<()> {
        let command_tx = self.command_tx.as_ref().unwrap().clone();
        match action {
            Action::NewWorkspacesData(workspaces) => {
                self.list.items = workspaces;
                self.sorter.sort(&mut self.list.items);
                if !self.list.items.is_empty() && self.list.state.selected().is_none() {
                    self.list.state.select(Some(0));
                }
                self.send_workspace_id()?;
            }
            Action::GoUp => {
                self.select_previous()?;
            }
            Action::GoDown => {
                self.select_next()?;
            }
            Action::GoToTop => {
                self.select_top()?;
            }
            Action::GoToBottom => {
                self.select_bottom()?;
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
                    self.select_top()?;
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
                        self.select_previous()?;
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
                        self.select_next()?;
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
                    self.select_bottom()?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let style = if self.is_focused {
            self.config.styles[&StyleName::Highlight]
        } else {
            Style::default()
        };

        // if adding an item at the end fails the selected item will be higher than the number of
        // items so i did this:
        if let Some(selected) = self.list.state.selected_mut() {
            *selected = min(*selected, self.list.items.len() - 1);
        }

        let block = Block::default()
            .title("Workspaces")
            .border_style(style)
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        let mut items: Vec<ListItem> = vec![];
        for (i, item) in self.list.items.iter().enumerate() {
            if self.mode == Mode::Edit(i as i32) {
                items.push(ListItem::new(Line::styled(
                    self.input.clone(),
                    Style::default(),
                )));
                continue;
            }
            if self.mode == Mode::Insert(i as i32) {
                items.push(ListItem::new(Line::styled(
                    self.input.clone(),
                    Style::default(),
                )));
            }
            let style = if self.highlighted_item.0.is_some() && self.highlighted_item.1 == item.name
            {
                self.config.styles[&StyleName::Error]
            } else {
                Style::default()
            };
            items.push(ListItem::new(Line::from(item.name.clone()).style(style)));
        }

        if self.mode == Mode::Insert(self.list.items.len() as i32) {
            items.push(ListItem::new(Line::styled(
                self.input.clone(),
                Style::default(),
            )));
        }

        let items = List::new(items)
            .block(block)
            .highlight_style(self.config.styles[&StyleName::Selected]);

        frame.render_stateful_widget(items, area, &mut self.list.state);
        if let Mode::Insert(line) = self.mode {
            frame.set_cursor(
                area.x + 1 + self.character_index as u16,
                area.y + line as u16 + 1,
            );
        }

        if let Mode::Edit(line) = self.mode {
            frame.set_cursor(
                area.x + 1 + self.character_index as u16,
                area.y + line as u16 + 1,
            );
        }
        Ok(())
    }
}
