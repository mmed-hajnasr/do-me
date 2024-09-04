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
pub struct WorkspacesComponent {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    list: WorkspacesList,
    input: String, // save the currently edited text
    character_index: usize,
    mode: Mode,
    sorter: WorkspaceSorter,
    is_focused: bool,
    highlighted_item: (Option<usize>, String),
    to_be_selected: Option<usize>, // to save the index of the new element to be selected.
}

#[derive(Default, PartialEq, Eq, Debug)]
enum Mode {
    #[default]
    Normal,
    Insert(usize),
    Edit(usize),
}

#[derive(Default, Debug)]
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
        if self.list.items.is_empty() {
            return Ok(());
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
        self.send_workspace_id()?;
        Ok(())
    }

    fn select_previous(&mut self) -> Result<()> {
        if self.list.items.is_empty() {
            return Ok(());
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
                self.list.state.select(Some(self.list.items.len() - 1));
            }
        }
        self.send_workspace_id()?;
        Ok(())
    }

    fn select_bottom(&mut self) -> Result<()> {
        if self.list.items.is_empty() {
            return Ok(());
        }
        self.list.state.select(Some(self.list.items.len() - 1));
        self.send_workspace_id()?;
        Ok(())
    }

    fn select_top(&mut self) -> Result<()> {
        if self.list.items.is_empty() {
            return Ok(());
        }
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
                self.to_be_selected = Some(target);
                self.input.clear();
                self.character_index = 0;
                self.mode = Mode::Normal;
            }
            Mode::Edit(target) => {
                let w = UpdateWorkspace {
                    id: self.list.items[target].id,
                    name: Some(self.input.trim().to_string()),
                    ..Default::default()
                };
                command_tx.send(Action::UpdateWorkspace(w))?;
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

                // selection handling
                if self.list.items.is_empty() {
                    self.list.state.select(None);
                    command_tx.send(Action::UnselectWorkspace)?;
                } else if let Some(index) = self.to_be_selected {
                    let index = min(index, self.list.items.len() - 1);
                    self.list.state.select(Some(index));
                    self.to_be_selected = None;
                } else if self.list.state.selected().is_none() {
                    self.list.state.select(Some(0));
                } else if let Some(selected) = self.list.state.selected_mut() {
                    *selected = min(*selected, self.list.items.len() - 1);
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
                            order: Some(selected - 1),
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
                            order: Some(selected + 1),
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
                        order: Some(self.list.items.len()),
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
        let selected_style = self.config.styles[&StyleName::Selected];
        let error_style = self.config.styles[&StyleName::Error];
        let block_style = if self.is_focused {
            self.config.styles[&StyleName::Highlight]
        } else {
            Style::default()
        };

        let block = Block::default()
            .title("Workspaces")
            .border_style(block_style)
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        let mut items: Vec<ListItem> = self
            .list
            .items
            .iter()
            .map(|w| {
                if self.highlighted_item.0.is_some() && self.highlighted_item.1 == w.name {
                    ListItem::new(Line::from(w.name.clone()).style(error_style))
                } else {
                    ListItem::new(Line::from(w.name.clone()))
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
        if let Mode::Edit(line) | Mode::Insert(line) = self.mode {
            frame.set_cursor(
                area.x + 1 + self.character_index as u16,
                area.y + line as u16 + 1,
            );
        }
        Ok(())
    }
}
