use super::DatabaseOperations;
use crate::errors::DoMeError;
use crate::structs::*;
use color_eyre::{eyre::Ok, Result};
use rusqlite::{params, Error};

impl DatabaseOperations {
    pub fn handle_update_workspace(&self, info: UpdateWorkspace) -> Result<()> {
        const UPDATE_WORKSPACE_QUERY: &str =
            "UPDATE Workspace SET name = COALESCE(?, name),workspace_order = COALESCE(?, workspace_order) WHERE id = ?";
        match self.conn.execute(
            UPDATE_WORKSPACE_QUERY,
            params![info.name, info.order, info.id],
        ) {
            Err(Error::SqliteFailure(e, _)) => {
                if e.code == rusqlite::ErrorCode::ConstraintViolation {
                    Err(DoMeError::WorkspaceAlreadyExists(
                        info.name
                            .expect("the name is not none since it exist in the database"),
                    )
                    .into())
                } else {
                    Err(e.into())
                }
            }
            Err(e) => Err(e.into()),
            _ => Ok(()),
        }
    }

    pub fn handle_update_task(&self, info: UpdateTask) -> Result<()> {
        const UPDATE_TASK_QUERY: &str = "UPDATE Task SET name = COALESCE(?, name), task_order = COALESCE(?, task_order), description = COALESCE(?, description), priority = COALESCE(?, priority), completed = COALESCE(?, completed) WHERE id = ?";
        match self.conn.execute(
            UPDATE_TASK_QUERY,
            params![
                info.name,
                info.order,
                info.description,
                info.priority,
                info.completed,
                info.id
            ],
        ) {
            Err(Error::SqliteFailure(e, _)) => {
                if e.code == rusqlite::ErrorCode::ConstraintViolation {
                    Err(DoMeError::TaskAlreadyExists(
                        info.name
                            .expect("the name is not none since it exist in the database"),
                    )
                    .into())
                } else {
                    Err(e.into())
                }
            }
            Err(e) => Err(e.into()),
            _ => Ok(()),
        }
    }
}
