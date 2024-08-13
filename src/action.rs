use serde::{Deserialize, Serialize};
use strum::Display;

use crate::app::ComponentId;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
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
}

impl Action {
    pub fn get_target(&self) -> ComponentId {
        ComponentId::All
    }
}
