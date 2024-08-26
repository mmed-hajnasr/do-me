use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout},
    prelude::Rect,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::{
    action::Action,
    components::{fps::FpsCounter, workspaces::WorkspacesComponent, Component},
    config::Config,
    database_ops::DatabaseOperations,
    errors::DoMeError,
    tui::{Event, Tui},
};

pub struct App {
    config: Config,
    database: DatabaseOperations,
    tick_rate: f64,
    frame_rate: f64,
    components: HashMap<ComponentId, Box<dyn Component>>,
    focused: ComponentId,
    should_quit: bool,
    should_suspend: bool,
    mode: Mode,
    last_key_events: (Vec<KeyEvent>, Option<usize>),
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    selected_workspace: Option<i32>,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentId {
    #[default]
    Workspaces,
    Tasks,
    FpsCounter,
    DatabaseGet,
    DatabaseSetTasks,
    DatabaseSetWorkspaces,
    All,
    Focused,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Navigation,
    Global,
    Insert,
}

impl App {
    pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let mut components: HashMap<ComponentId, Box<dyn Component>> = HashMap::new();
        let config = Config::new()?;
        components.insert(ComponentId::FpsCounter, Box::new(FpsCounter::new()));
        components.insert(
            ComponentId::Workspaces,
            Box::new(WorkspacesComponent::new()),
        );
        Ok(Self {
            database: DatabaseOperations::new(config.config.data_dir.join("do_me.sqlite")),
            tick_rate,
            frame_rate,
            components,
            focused: ComponentId::default(),
            should_quit: false,
            should_suspend: false,
            config,
            mode: Mode::default(),
            last_key_events: (Vec::new(), None),
            action_tx,
            action_rx,
            selected_workspace: None,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?
            // .mouse(true) // uncomment this line to enable mouse support
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        tui.enter()?;

        for component in self.components.values_mut() {
            component.register_action_handler(self.action_tx.clone())?;
        }
        for component in self.components.values_mut() {
            component.register_config_handler(self.config.clone())?;
        }
        for component in self.components.values_mut() {
            component.init(tui.size()?)?;
        }

        self.components
            .get_mut(&ComponentId::Workspaces)
            .unwrap()
            .focus(true)?;

        let action_tx = self.action_tx.clone();
        loop {
            self.handle_events(&mut tui).await?;
            self.handle_actions(&mut tui)?;
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                action_tx.send(Action::ClearScreen)?;
                // tui.mouse(true);
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };
        let action_tx = self.action_tx.clone();
        match event {
            Event::Quit => action_tx.send(Action::Quit)?,
            Event::Tick => action_tx.send(Action::Tick)?,
            Event::Render => action_tx.send(Action::Render)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            Event::Key(key) => self.handle_key_event(key)?,
            Event::FocusGained => {
                self.components
                    .get_mut(&self.focused)
                    .unwrap()
                    .focus(true)?;
            }
            Event::FocusLost => {
                self.components
                    .get_mut(&self.focused)
                    .unwrap()
                    .focus(false)?;
                panic!("Focus lost event not implemented.");
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        // if editing mode send all keypresses to the focused component.
        if self.mode == Mode::Insert {
            self.components
                .get_mut(&self.focused)
                .unwrap()
                .update(Action::SendKeyEvent(key))?;
            return Ok(());
        }

        let global_keymap = self
            .config
            .keybindings
            .get(&Mode::Global)
            .expect("did not find global keybindings")
            .clone();

        // use global keymap if action found return.
        if self.use_keymap(key, &global_keymap)? {
            return Ok(());
        }

        // search of the keymap of the focused component. if not found no action will be sent.
        let Some(keymap) = self.config.keybindings.get(&self.mode) else {
            return Ok(());
        };

        if self.use_keymap(key, &keymap.clone())? {
            return Ok(());
        }

        if self.last_key_events.1.is_none() {
            self.last_key_events.1 = Some(4);
        }
        self.last_key_events.0.push(key);

        Ok(())
    }

    fn use_keymap(
        &mut self,
        key: KeyEvent,
        keymap: &HashMap<Vec<KeyEvent>, Action>,
    ) -> Result<bool> {
        let action_tx = self.action_tx.clone();

        match keymap.get(&vec![key]) {
            Some(action) => {
                action_tx.send(action.clone())?;
                return Ok(true);
            }

            _ => {
                if self.last_key_events.1.is_none() {
                    return Ok(false);
                }
                let mut events_list = self.last_key_events.0.clone();
                events_list.push(key);
                for i in 0..events_list.len() {
                    if let Some(action) = keymap.get(&events_list[i..]) {
                        action_tx.send(action.clone())?;
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != Action::Tick && action != Action::Render {
                info!("Got action: {action:?}");
            }

            let target = action.get_target();
            match target {
                ComponentId::All => {
                    match action {
                        Action::Tick => match self.last_key_events.1 {
                            Some(0) => {
                                self.last_key_events.0.clear();
                                self.last_key_events.1 = None;
                            }
                            Some(i) => {
                                self.last_key_events.1 = Some(i - 1);
                            }
                            None => {}
                        },
                        Action::Quit => self.should_quit = true,
                        Action::Suspend => self.should_suspend = true,
                        Action::Resume => self.should_suspend = false,
                        Action::ClearScreen => tui.terminal.clear()?,
                        Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                        Action::Render => self.render(tui)?,
                        Action::EnterInsertMode => self.mode = Mode::Insert,
                        Action::LeaveInsertMode => self.mode = Mode::Navigation,
                        _ => {}
                    }
                    for component in self.components.values_mut() {
                        component.update(action.clone())?;
                    }
                }
                ComponentId::Focused => {
                    if let Some(component) = self.components.get_mut(&self.focused) {
                        component.update(action.clone())?;
                    } else {
                        error!("Component not found: {:?}", &self.focused);
                    }
                }
                ComponentId::DatabaseSetTasks => {
                    self.database.handle_update_actions(action.clone())?;
                    if let Some(workspace_id) = self.selected_workspace {
                        self.action_tx
                            .send(Action::RequestTasksData(workspace_id))?;
                    } 
                }
                ComponentId::DatabaseSetWorkspaces => {
                    if let Err(e) = self.database.handle_update_actions(action.clone()) {
                        if let Some(DoMeError::WorkspaceAlreadyExists(name)) =
                            e.downcast_ref::<DoMeError>()
                        {
                            self.action_tx.send(Action::HighlightWorkspace(name.to_string()))?;
                        } else {
                            return Err(e);
                        }
                    }
                    self.action_tx.send(Action::RequestWorkspacesData)?;
                }
                ComponentId::DatabaseGet => match action {
                    Action::RequestTasksData(workspace_id) => {
                        let tasks = self.database.get_tasks(workspace_id)?;
                        self.action_tx.send(Action::NewTasksData(tasks))?;
                    }
                    Action::RequestWorkspacesData => {
                        let workspaces = self.database.get_workspaces()?;
                        self.action_tx.send(Action::NewWorkspacesData(workspaces))?;
                    }
                    _ => {}
                },
                _ => {
                    if let Some(component) = self.components.get_mut(&target) {
                        component.update(action.clone())?;
                    } else {
                        error!("Component not found: {:?}", target);
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|frame| {
            let area = frame.size();
            let [workspace_area, task_area] =
                Layout::horizontal([Constraint::Length(20), Constraint::Fill(1)]).areas(area);
            for (id, component) in &mut self.components {
                let area = match id {
                    ComponentId::Workspaces => workspace_area,
                    ComponentId::Tasks => task_area,
                    _ => continue,
                };

                if let Err(err) = component.draw(frame, area) {
                    let _ = self
                        .action_tx
                        .send(Action::Error(format!("Failed to draw: {:?}", err)));
                }
            }

            let fps = self.components.get_mut(&ComponentId::FpsCounter).unwrap();
            let _ = fps.draw(frame, area);
        })?;
        Ok(())
    }
}
