use super::DatabaseOperations;
use crate::errors::DoMeError;
use crate::structs::*;
use color_eyre::{eyre::Ok, Result};
use rusqlite::{params, Error};

impl DatabaseOperations {
    pub fn handle_add_workspace(&self, info: AddWorkspace) -> Result<()> {
        const ADD_WORKSPACE_QUERY: &str =
            "INSERT INTO Workspace (name, workspace_order) VALUES (?, ?)";
        match self
            .conn
            .execute(ADD_WORKSPACE_QUERY, params![info.name, info.order])
        {
            Err(Error::SqliteFailure(e, _)) => {
                if e.code == rusqlite::ErrorCode::ConstraintViolation {
                    Err(DoMeError::WorkspaceAlreadyExists(info.name).into())
                } else {
                    Err(e.into())
                }
            }
            Err(e) => Err(e.into()),
            _ => Ok(()),
        }
    }

    pub fn handle_add_task(&self, info: AddTask) -> Result<()> {
        const ADD_TASK_QUERY: &str = "INSERT INTO Task (name, description, priority, task_order, workspaceid) VALUES (?, COALESCE(?, ''), COALESCE(?, 3), ?, ?)";
        match self.conn.execute(
            ADD_TASK_QUERY,
            params![
                info.name,
                info.description,
                info.priority,
                info.order,
                info.workspace_id
            ],
        ) {
            Err(Error::SqliteFailure(e, _)) => {
                if e.code == rusqlite::ErrorCode::ConstraintViolation {
                    Err(DoMeError::TaskAlreadyExists(info.name).into())
                } else {
                    Err(e.into())
                }
            }
            Err(e) => Err(e.into()),
            _ => Ok(()),
        }
    }
}
