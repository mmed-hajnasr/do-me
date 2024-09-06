use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Task {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub priority: i32,
    pub completed: bool,
    pub create_date: NaiveDateTime,
    pub order: usize,
    pub workspace_id: i32,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
    pub order: usize,
    pub create_date: NaiveDateTime,
    pub update_date: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum TaskSorter {
    Priority(bool),
    Order(bool),
    CreateDate(bool),
}

impl Default for TaskSorter {
    fn default() -> Self {
        TaskSorter::Order(true)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum WorkspaceSorter {
    Order(bool),
    CreateDate(bool),
    UpdateDate(bool),
}

impl Default for WorkspaceSorter {
    fn default() -> Self {
        WorkspaceSorter::Order(true)
    }
}

impl WorkspaceSorter {
    pub fn sort(&self, workspaces: &mut [Workspace]) {
        let cmp_func = match self {
            WorkspaceSorter::Order(asc) => {
                if *asc {
                    |a: &Workspace, b: &Workspace| a.order.cmp(&b.order)
                } else {
                    |a: &Workspace, b: &Workspace| b.order.cmp(&a.order)
                }
            }
            WorkspaceSorter::CreateDate(asc) => {
                if *asc {
                    |a: &Workspace, b: &Workspace| a.create_date.cmp(&b.create_date)
                } else {
                    |a: &Workspace, b: &Workspace| b.create_date.cmp(&a.create_date)
                }
            }
            WorkspaceSorter::UpdateDate(asc) => {
                if *asc {
                    |a: &Workspace, b: &Workspace| a.update_date.cmp(&b.update_date)
                } else {
                    |a: &Workspace, b: &Workspace| b.update_date.cmp(&a.update_date)
                }
            }
        };
        workspaces.sort_by(cmp_func);
    }
}

impl TaskSorter {
    pub fn sort(&self, tasks: &mut [Task]) {
        let cmp_func = match self {
            TaskSorter::Priority(asc) => {
                if *asc {
                    |a: &Task, b: &Task| a.priority.cmp(&b.priority)
                } else {
                    |a: &Task, b: &Task| b.priority.cmp(&a.priority)
                }
            }
            TaskSorter::Order(asc) => {
                if *asc {
                    |a: &Task, b: &Task| a.order.cmp(&b.order)
                } else {
                    |a: &Task, b: &Task| b.order.cmp(&a.order)
                }
            }
            TaskSorter::CreateDate(asc) => {
                if *asc {
                    |a: &Task, b: &Task| a.create_date.cmp(&b.create_date)
                } else {
                    |a: &Task, b: &Task| b.create_date.cmp(&a.create_date)
                }
            }
        };
        tasks.sort_by(cmp_func);
    }
}

#[derive(Default, Debug, Serialize, Clone, PartialEq, Deserialize, Eq)]
pub struct AddTask {
    pub name: String,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub order: Option<usize>,
    pub workspace_id: i32,
}

#[derive(Default, Debug, Serialize, Clone, PartialEq, Eq, Deserialize)]
pub struct AddWorkspace {
    pub name: String,
    pub order: Option<usize>,
}

#[derive(Default, Debug, Serialize, Clone, PartialEq, Eq, Deserialize)]
pub struct UpdateTask {
    pub id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub completed: Option<bool>,
    pub order: Option<usize>,
}

#[derive(Default, Debug, Serialize, Clone, PartialEq, Eq, Deserialize)]
pub struct UpdateWorkspace {
    pub id: i32,
    pub name: Option<String>,
    pub order: Option<usize>,
}
