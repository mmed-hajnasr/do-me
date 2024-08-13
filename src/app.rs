use std::collections::HashMap;

use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::{
    action::Action,
    components::{fps::FpsCounter, home::Home, Component},
    config::Config,
    tui::{Event, Tui},
};

pub struct App {
    config: Config,
    tick_rate: f64,
    frame_rate: f64,
    components: HashMap<ComponentId, Box<dyn Component>>,
    focused: ComponentId,
    should_quit: bool,
    should_suspend: bool,
    mode: Mode,
    last_tick_key_events: Vec<KeyEvent>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentId {
    #[default]
    Home,
    FpsCounter,
    All,
    Focused,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Navigation,
    Global,
}

impl App {
    pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let mut components: HashMap<ComponentId, Box<dyn Component>> = HashMap::new();
        components.insert(ComponentId::Home, Box::new(Home::new()));
        components.insert(ComponentId::FpsCounter, Box::new(FpsCounter::new()));
        Ok(Self {
            tick_rate,
            frame_rate,
            components,
            focused: ComponentId::default(),
            should_quit: false,
            should_suspend: false,
            config: Config::new()?,
            mode: Mode::default(),
            last_tick_key_events: Vec::new(),
            action_tx,
            action_rx,
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
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
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

        self.last_tick_key_events.push(key);

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
                info!("Got action: {action:?}");
                action_tx.send(action.clone())?;
                return Ok(true);
            }

            _ => {
                self.last_tick_key_events.push(key);
                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                    info!("Got action: {action:?}");
                    action_tx.send(action.clone())?;
                    return Ok(true);
                }
                self.last_tick_key_events.pop();
            }
        }
        Ok(false)
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            assert_ne!(&self.focused, &ComponentId::All);
            assert_ne!(&self.focused, &ComponentId::Focused);
            if action != Action::Tick && action != Action::Render {
                debug!("{action:?}");
            }

            let target = action.get_target();
            match target {
                ComponentId::All => {
                    match action {
                        Action::Tick => {
                            self.last_tick_key_events.clear();
                        }
                        Action::Quit => self.should_quit = true,
                        Action::Suspend => self.should_suspend = true,
                        Action::Resume => self.should_suspend = false,
                        Action::ClearScreen => tui.terminal.clear()?,
                        Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                        Action::Render => self.render(tui)?,
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
            for component in self.components.values_mut() {
                if let Err(err) = component.draw(frame, frame.size()) {
                    let _ = self
                        .action_tx
                        .send(Action::Error(format!("Failed to draw: {:?}", err)));
                }
            }
        })?;
        Ok(())
    }
}
