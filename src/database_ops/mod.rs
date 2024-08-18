use std::path::PathBuf;

use crate::action::Action;
use color_eyre::eyre::Ok;
use color_eyre::Result;
use rusqlite::Connection;
mod add;
mod output;
mod remove;
#[cfg(test)]
mod tests;
mod update;

pub struct DatabaseOperations {
    conn: Connection,
}

impl DatabaseOperations {
    pub fn new(database_path: PathBuf) -> DatabaseOperations {
        let conn = Connection::open(database_path).unwrap();
        conn.execute_batch(include_str!("../../sql/schema.sql"))
            .expect("Error setting up the database");
        DatabaseOperations { conn }
    }

    pub fn handle_update_actions(&self, action: Action) -> Result<()> {
        match action {
            Action::AddWorkspace(info) => self.handle_add_workspace(info),
            Action::AddTask(info) => self.handle_add_task(info),
            Action::UpdateWorkspace(info) => self.handle_update_workspace(info),
            Action::UpdateTask(info) => self.handle_update_task(info),
            Action::RemoveWorkspace(id) => self.handle_remove_workspace(id),
            Action::RemoveTask(id) => self.handle_remove_task(id),
            _ => Ok(()),
        }
    }
}
