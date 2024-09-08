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
    NewTasksData((Vec<Task>, i32)),
    NewWorkspacesData(Vec<Workspace>),
    SelectWorkspace(i32),
    UnselectWorkspace,
    HighlightWorkspace(String),
    HighlightTask(String),
    FocusOnTasks,
    FocusOnWorkspaces,
    ToggleCompletion,
    EditDescription,
    IncreasePriority,
    DecreasePriority,
    SortTasks(TaskSorter),
    SortWorkspaces(WorkspaceSorter),
    ToggleSortDirection,
    Select,
    ExitSortMenu(ComponentId),
    SetupSortMenu(ComponentId),
    Cancel,
    OpenSortMenu,
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

            Action::GoUp
            | Action::GoDown
            | Action::GoToTop
            | Action::GoToBottom
            | Action::AddItemAfter
            | Action::AddItemBefore
            | Action::DeleteItem
            | Action::EditDescription
            | Action::ToggleCompletion
            | Action::IncreasePriority
            | Action::DecreasePriority
            | Action::EditItem
            | Action::MoveItemUp
            | Action::MoveItemDown
            | Action::MoveItemTop
            | Action::MoveItemBottom
            | Action::SendKeyEvent(..) => ComponentId::Focused,

            Action::SortTasks(_) => ComponentId::Tasks,
            Action::SortWorkspaces(_) => ComponentId::Workspaces,
            Action::ToggleSortDirection | Action::SetupSortMenu(_) => ComponentId::SortMenu,
            Action::Select | Action::Cancel => ComponentId::Focused,

            Action::Tick
            | Action::Render
            | Action::Resize(..)
            | Action::Suspend
            | Action::Resume
            | Action::Quit
            | Action::ClearScreen
            | Action::Error(_)
            | Action::Help
            | Action::LeaveInsertMode
            | Action::EnterInsertMode
            | Action::SelectWorkspace(_)
            | Action::UnselectWorkspace
            | Action::FocusOnTasks
            | Action::FocusOnWorkspaces
            | Action::ExitSortMenu(_)
            | Action::OpenSortMenu => ComponentId::All,
        }
    }
}
