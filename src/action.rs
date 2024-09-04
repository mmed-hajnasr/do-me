use crate::app::ComponentId;
use crate::structs::*;
use crossterm::event::KeyEvent;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, PartialEq, Eq, Clone, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,
    GoUp,
    GoDown,
    GoToTop,
    GoToBottom,
    LeaveInsertMode,
    EnterInsertMode,
    AddItemBefore,
    AddItemAfter,
    DeleteItem,
    EditItem,
    MoveItemUp,
    MoveItemDown,
    MoveItemTop,
    MoveItemBottom,
    SendKeyEvent(KeyEvent),
    AddTask(AddTask),
    AddWorkspace(AddWorkspace),
    UpdateTask(UpdateTask),
    UpdateWorkspace(UpdateWorkspace),
    RemoveTask(i32),
    RemoveWorkspace(i32),
    RequestTasksData(i32),
    RequestWorkspacesData,
    NewTasksData((Vec<Task>,i32)),
    NewWorkspacesData(Vec<Workspace>),
    SelectWorkspace(i32),
    UnselectWorkspace,
    HighlightWorkspace(String),
    HighlightTask(String),
    MoveFocusRight,
    MoveFocusLeft,
}

impl Action {
    pub fn get_target(&self) -> ComponentId {
        match self {
            Action::AddTask(_) | Action::UpdateTask(_) | Action::RemoveTask(_) => {
                ComponentId::DatabaseSetTasks
            }
            Action::AddWorkspace(_) | Action::UpdateWorkspace(_) | Action::RemoveWorkspace(_) => {
                ComponentId::DatabaseSetWorkspaces
            }
            Action::RequestTasksData(_) | Action::RequestWorkspacesData => ComponentId::DatabaseGet,
            Action::NewTasksData(_) | Action::HighlightTask(_) => ComponentId::Tasks,
            Action::NewWorkspacesData(_) | Action::HighlightWorkspace(_) => ComponentId::Workspaces,
            Action::GoUp | Action::GoDown | Action::GoToTop | Action::GoToBottom => {
                ComponentId::Focused
            }
            Action::AddItemAfter
            | Action::AddItemBefore
            | Action::DeleteItem
            | Action::EditItem => ComponentId::Focused,
            Action::MoveItemUp
            | Action::MoveItemDown
            | Action::MoveItemTop
            | Action::MoveItemBottom => ComponentId::Focused,
            Action::SendKeyEvent(..) => ComponentId::Focused,
            _ => ComponentId::All,
        }
    }
}
