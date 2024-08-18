use super::DatabaseOperations;
use crate::structs::*;
use chrono::NaiveDateTime;
use color_eyre::Result;
use rusqlite::{params, Error, Row};

pub fn parse_datetime(row: &Row, index: usize) -> rusqlite::Result<NaiveDateTime> {
    let date_str: String = row.get(index)?;
    Ok(NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S").unwrap())
}

impl DatabaseOperations {
    pub fn get_task(&self, id: i32) -> Result<Option<Task>> {
        const GET_TASK_QUERY: &str = "SELECT name, task_order, description, priority, completed, create_date, workspaceid FROM Task WHERE id = ?";
        match self.conn.query_row(GET_TASK_QUERY, params![id], |row| {
            Ok(Task {
                id,
                name: row.get(0)?,
                order: row.get(1)?,
                description: row.get(2)?,
                priority: row.get(3)?,
                completed: row.get(4)?,
                create_date: parse_datetime(row, 5)?,
                workspace_id: row.get(6)?,
            })
        }) {
            Ok(task) => Ok(Some(task)),
            Err(Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_workspace(&self, id: i32) -> Result<Option<Workspace>> {
        const GET_WORKSPACE_QUERY: &str =
            "SELECT name, workspace_order, create_date, update_date FROM Workspace WHERE id = ?";
        match self
            .conn
            .query_row(GET_WORKSPACE_QUERY, params![id], |row| {
                Ok(Workspace {
                    id,
                    name: row.get(0)?,
                    order: row.get(1)?,
                    create_date: parse_datetime(row, 2)?,
                    update_date: parse_datetime(row, 3)?,
                })
            }) {
            Ok(workspace) => Ok(Some(workspace)),
            Err(Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_tasks(&self, workspace_id: i32) -> Result<Vec<Task>> {
        const GET_TASKS_QUERY: &str = "SELECT id, name, task_order, description, priority, completed, create_date FROM Task WHERE workspaceid = ?;";

        let mut stmt = self.conn.prepare(GET_TASKS_QUERY)?;
        let task_iter = stmt.query_map(params![workspace_id], |row| {
            Ok(Task {
                id: row.get(0)?,
                name: row.get(1)?,
                order: row.get(2)?,
                description: row.get(3)?,
                priority: row.get(4)?,
                completed: row.get(5)?,
                create_date: parse_datetime(row, 6)?,
                workspace_id,
            })
        })?;
        Ok(task_iter.map(|task| task.unwrap()).collect())
    }

    pub fn get_workspaces(&self) -> Result<Vec<Workspace>> {
        const GET_WORKSPACES_QUERY: &str =
            "SELECT id, name, workspace_order, create_date, update_date FROM Workspace";
        let mut stmt = self.conn.prepare(GET_WORKSPACES_QUERY)?;
        let workspace_iter = stmt.query_map([], |row| {
            Ok(Workspace {
                id: row.get(0)?,
                name: row.get(1)?,
                order: row.get(2)?,
                create_date: parse_datetime(row, 3)?,
                update_date: parse_datetime(row, 4)?,
            })
        })?;
        Ok(workspace_iter.map(|workspace| workspace.unwrap()).collect())
    }

    pub fn search_task_name(&self, name: &str, workspace_id: i32) -> Result<Option<i32>> {
        const SEARCH_TASK_NAME_QUERY: &str =
            "SELECT id FROM Task WHERE name = ? AND workspaceid = ?";
        match self
            .conn
            .query_row(SEARCH_TASK_NAME_QUERY, params![name, workspace_id], |row| {
                Ok(row.get(0))
            }) {
            Ok(id) => Ok(Some(id?)),
            Err(Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn search_workspace_name(&self, name: &str) -> Result<Option<i32>> {
        const SEARCH_WORKSPACE_NAME_QUERY: &str = "SELECT id FROM Workspace WHERE name = ?";
        match self
            .conn
            .query_row(SEARCH_WORKSPACE_NAME_QUERY, params![name], |row| {
                Ok(row.get(0))
            }) {
            Ok(id) => Ok(Some(id?)),
            Err(Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
