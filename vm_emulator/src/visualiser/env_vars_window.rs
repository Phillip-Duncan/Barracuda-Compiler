use crate::emulator::ThreadContext;
use crate::visualiser::scrollable_list::ScrollableList;
use crate::visualiser::event_widget::EventWidget;
use tui::widgets::{Block, Borders, BorderType, ListItem};
use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use crossterm::event::{KeyEvent, KeyCode};
use crate::EnvironmentVariable;

pub struct EnvVarWindow {
    env_vars: Vec<String>,
    scroll_list: ScrollableList
}

impl EnvVarWindow {
    pub(crate) fn new() -> Self {
        EnvVarWindow {
            env_vars: Default::default(),
            scroll_list: ScrollableList::new()
        }
    }

    pub(crate) fn update_env_vars(&mut self, context: &ThreadContext) {
        self.env_vars.clear();

        let mut env_vars: Vec<EnvironmentVariable> = context.get_env_vars().values().cloned().collect();
        env_vars.sort_by(|a,b| a.address.cmp(&b.address));

        for var in env_vars {
            let address = var.address;
            let value = var.value;
            let name = var.name.clone().unwrap_or(String::from("N/A"));

            self.env_vars.push(format!("{}:{}={}", address, name, value));
        }
    }

    pub(crate) fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, highlighted: bool) {
        let mut block = Block::default()
            .title("Environment Vars")
            .borders(Borders::ALL);

        if highlighted {
            block = block.border_type(BorderType::Thick)
        }

        let items: Vec<ListItem>= self.env_vars.iter().map(|i| ListItem::new(i.as_ref())).collect();

        self.scroll_list.draw(f, items, area, Some(block));
    }
}

impl EventWidget for EnvVarWindow {
    fn on_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                self.scroll_list.scroll_up();
            },
            KeyCode::Down => {
                self.scroll_list.scroll_down();
            },
            _ => {}
        }
    }
}