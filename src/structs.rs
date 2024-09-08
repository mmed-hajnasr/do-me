use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Task {
    pub id: i32,
    pub name: String,
    pub description: String,
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

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct TaskSorter {
    desc: bool,
    sort_type: TaskSortType,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum TaskSortType {
    #[default]
    Order,
    Priority,
    Completion,
    CreateDate,
    Name,
    Description,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct WorkspaceSorter {
    desc: bool,
    sort_type: WorkspaceSortType,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum WorkspaceSortType {
    #[default]
    Order,
    CreateDate,
    UpdateDate,
    Name,
}

impl WorkspaceSorter {
    pub fn new(sort_type: WorkspaceSortType, desc: bool) -> Self {
        Self { sort_type, desc }
    }
    pub fn sort(&self, workspaces: &mut [Workspace]) {
        let cmp_func = |a: &Workspace, b: &Workspace| {
            let mut order = match self.sort_type {
                WorkspaceSortType::Order => a.order.cmp(&b.order),
                WorkspaceSortType::CreateDate => a.create_date.cmp(&b.create_date),
                WorkspaceSortType::UpdateDate => a.update_date.cmp(&b.update_date),
                WorkspaceSortType::Name => a.name.cmp(&b.name),
            };
            if self.desc {
                order = order.reverse();
            }
            order.then(a.order.cmp(&b.order))
        };
        workspaces.sort_by(cmp_func);
    }
}

impl TaskSorter {
    pub fn new(sort_type: TaskSortType, desc: bool) -> Self {
        Self { sort_type, desc }
    }

    pub fn sort(&self, tasks: &mut [Task]) {
        let cmp_func = |a: &Task, b: &Task| {
            let mut order = match self.sort_type {
                TaskSortType::Priority => a.priority.cmp(&b.priority),
                TaskSortType::Order => a.order.cmp(&b.order),
                TaskSortType::CreateDate => a.create_date.cmp(&b.create_date),
                TaskSortType::Name => a.name.cmp(&b.name),
                TaskSortType::Description => a.description.cmp(&b.description),
                TaskSortType::Completion => a.completed.cmp(&b.completed),
            };
            if self.desc {
                order = order.reverse();
            }
            order.then(a.order.cmp(&b.order))
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
