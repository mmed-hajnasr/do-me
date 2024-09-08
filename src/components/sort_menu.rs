use super::Component;
use crate::action::Action;
use crate::app::ComponentId;
use crate::config::{Config, StyleName};
use crate::structs::{TaskSortType, TaskSorter, WorkspaceSortType, WorkspaceSorter};
use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

/// Sorting options
const WORKSPACE_OPTIONS: [&str; 3] = ["Name", "Date created", "Last Updated"];
const TASK_OPTIONS: [&str; 5] = [
    "Name",
    "Completion",
    "Date created",
    "Priority",
    "Description",
];

const WORKSPACE_SORTERS: [WorkspaceSortType; 3] = [
    WorkspaceSortType::Name,
    WorkspaceSortType::CreateDate,
    WorkspaceSortType::UpdateDate,
];

const TASK_SORTERS: [TaskSortType; 5] = [
    TaskSortType::Name,
    TaskSortType::Completion,
    TaskSortType::CreateDate,
    TaskSortType::Priority,
    TaskSortType::Description,
];

#[derive(Debug)]
pub struct SortMenu {
    is_focused: bool,
    objective: ComponentId,
    list: OptionList<'static>,
    desc: bool,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

#[derive(Debug, Default, PartialEq)]
struct OptionList<'a> {
    items: Vec<&'a str>,
    state: ListState,
}

impl SortMenu {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            is_focused: false,
            objective: ComponentId::Workspaces,
            list: OptionList::default(),
            desc: false,
            config: Config::default(),
        }
    }

    fn select_next(&mut self) {
        if self.list.items.is_empty() {
            return;
        }
        match self.list.state.selected_mut() {
            Some(selected) => {
                *selected += 1;
                *selected %= self.list.items.len();
            }
            None => {
                self.list.state.select(Some(0));
            }
        }
    }

    fn select_previous(&mut self) {
        if self.list.items.is_empty() {
            return;
        }
        match self.list.state.selected_mut() {
            Some(selected) => {
                if *selected == 0 {
                    *selected = self.list.items.len() - 1;
                } else {
                    *selected -= 1;
                }
            }
            None => {
                self.list.state.select(Some(0));
            }
        }
    }

    fn select_bottom(&mut self) {
        if self.list.items.is_empty() {
            return;
        }
        self.list.state.select(Some(self.list.items.len() - 1));
    }

    fn select_top(&mut self) {
        if self.list.items.is_empty() {
            return;
        }
        self.list.state.select(Some(0));
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

impl Component for SortMenu {
    fn focus(&mut self, focus: bool) -> Result<()> {
        self.is_focused = focus;
        if !focus {
            self.list = OptionList::default();
            self.desc = false;
        }
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<()> {
        match action {
            Action::GoUp => {
                self.select_previous();
            }
            Action::GoDown => {
                self.select_next();
            }
            Action::GoToTop => {
                self.select_top();
            }
            Action::GoToBottom => {
                self.select_bottom();
            }
            Action::Select => {
                let command_tx = self.command_tx.as_ref().unwrap();
                let selected = self.list.state.selected().unwrap();
                match self.objective {
                    ComponentId::Workspaces => {
                        let sorter =
                            WorkspaceSorter::new(WORKSPACE_SORTERS[selected].clone(), self.desc);
                        command_tx.send(Action::SortWorkspaces(sorter))?;
                    }
                    ComponentId::Tasks => {
                        let sorter = TaskSorter::new(TASK_SORTERS[selected].clone(), self.desc);
                        command_tx.send(Action::SortTasks(sorter))?;
                    }
                    _ => unreachable!(),
                }
                command_tx.send(Action::ExitSortMenu(self.objective))?;
            }
            Action::Cancel => {
                let command_tx = self.command_tx.as_ref().unwrap();
                command_tx.send(Action::ExitSortMenu(self.objective))?;
            }
            Action::ToggleSortDirection => {
                self.desc = !self.desc;
            }
            Action::SetupSortMenu(component_id) => {
                self.objective = component_id;
                match component_id {
                    ComponentId::Tasks => {
                        self.list.items = TASK_OPTIONS.to_vec();
                    }
                    ComponentId::Workspaces => {
                        self.list.items = WORKSPACE_OPTIONS.to_vec();
                    }
                    _ => unreachable!(),
                }
                self.list.state.select(Some(0));
            }
            _ => {}
        };
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if self.is_focused {
            let block_style = self.config.styles[&StyleName::Highlight];
            let selection_style = self.config.styles[&StyleName::Selected];
            let title = if self.desc {
                "Sort by descending order"
            } else {
                "Sort by ascending order:"
            };

            let block = Block::bordered()
                .border_type(BorderType::Rounded)
                .title(title)
                .style(block_style);

            let items: Vec<ListItem> = self
                .list
                .items
                .iter()
                .map(|&item| ListItem::new(item).style(Style::default()))
                .collect();

            let items = List::new(items)
                .block(block)
                .highlight_style(selection_style)
                .highlight_symbol(">>")
                .highlight_spacing(HighlightSpacing::Always);

            let area = centered_rect(60, 20, area);
            frame.render_widget(Clear, area); //this clears out the background
            frame.render_stateful_widget(items, area, &mut self.list.state);
        }
        Ok(())
    }
}
