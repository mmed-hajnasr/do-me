use super::DatabaseOperations;
use color_eyre::{eyre::Ok, Result};
use rusqlite::params;

impl DatabaseOperations {
    pub fn handle_remove_workspace(&self, id: i32) -> Result<()> {
        const REMOVE_WORKSPACE_QUERY: &str = "DELETE FROM Workspace WHERE id = ?";
        self.conn.execute(REMOVE_WORKSPACE_QUERY, params![id])?;
        Ok(())
    }

    pub fn handle_remove_task(&self, id: i32) -> Result<()> {
        const REMOVE_TASK_QUERY: &str = "DELETE FROM Task WHERE id = ?";
        self.conn.execute(REMOVE_TASK_QUERY, params![id])?;
        Ok(())
    }
}
