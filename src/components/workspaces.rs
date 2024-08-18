use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::Action, structs::*};

#[derive(Default)]
pub struct WorkspacesComponent {
    command_tx: Option<UnboundedSender<Action>>,
    list: WorkspacesList,
    input: String,
    character_index: usize,
    mode: Mode,
}

#[derive(Default, PartialEq, Eq)]
enum Mode {
    #[default]
    Normal,
    Insert(i32),
}

#[derive(Default)]
struct WorkspacesList {
    items: Vec<Workspace>,
    state: ListState,
}

impl From<&Workspace> for ListItem<'_> {
    fn from(workspace: &Workspace) -> Self {
        let line = Line::styled(workspace.name.clone(), Style::default());
        ListItem::new(line)
    }
}

impl WorkspacesComponent {
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
                let target = match self.mode {
                    Mode::Insert(target) => target,
                    _ => unreachable!(),
                };
                command_tx.send(Action::AddWorkspace(AddWorkspace {
                    name: self.input.clone(),
                    order: Some(target),
                }))?;
                command_tx.send(Action::LeaveInsertMode)?;
                self.input.clear();
                self.character_index = 0;
                self.mode = Mode::Normal;
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

    fn update(&mut self, action: Action) -> Result<()> {
        match action {
            Action::NewWorkspacesData(workspaces) => {
                self.list.items = workspaces;
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
                if let Mode::Insert(_) = self.mode {
                    self.handle_insert_mode(key)?;
                }
            }
            Action::AddItemAfter => {
                if let Some(selected) = self.list.state.selected() {
                    self.mode = Mode::Insert(selected as i32 + 1);
                } else {
                    self.mode = Mode::Insert(self.list.items.len() as i32);
                }
            }
            Action::AddItemBefore => {
                if let Some(selected) = self.list.state.selected() {
                    self.mode = Mode::Insert(selected as i32);
                } else {
                    self.mode = Mode::Insert(0);
                }
            }
            Action::DeleteItem => {
                if let Some(selected) = self.list.state.selected() {
                    let command_tx = self.command_tx.as_ref().unwrap();
                    command_tx.send(Action::RemoveWorkspace(
                        self.list.items[selected].id,
                    ))?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let block = Block::default()
            .title("Workspaces")
            .border_style(Style::default().fg(Color::Magenta))
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        let mut items: Vec<ListItem> = vec![];
        for (i, item) in self.list.items.iter().enumerate() {
            if self.mode == Mode::Insert(i as i32) {
                items.push(ListItem::new(Line::styled(
                    self.input.clone(),
                    Style::default(),
                )));
            }
            items.push(item.into());
        }

        if self.mode == Mode::Insert(self.list.items.len() as i32) {
            items.push(ListItem::new(Line::styled(
                self.input.clone(),
                Style::default(),
            )));
        }

        let items = List::new(items)
            .block(block)
            .highlight_style(Style::new().add_modifier(Modifier::BOLD));

        frame.render_stateful_widget(items, area, &mut self.list.state);
        if let Mode::Insert(line) = self.mode {
            frame.set_cursor(
                area.x + 1 + self.character_index as u16,
                area.y + line as u16 + 1,
            );
        }
        Ok(())
    }
}
