use crate::emulator::{ThreadContext, ENV_VAR_COUNT};
use crate::visualiser::scrollable_list::ScrollableList;
use crate::visualiser::event_widget::EventWidget;
use tui::widgets::{ListState, Block, Borders, BorderType, ListItem, List};
use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Modifier, Style};
use crossterm::event::{KeyEvent, KeyCode};

pub struct EnvVarWindow {
    env_vars: Vec<String>,
    scroll_list: ScrollableList
}

impl EnvVarWindow {
    pub(crate) fn new() -> Self {
        EnvVarWindow {
            env_vars: Vec::new(),
            scroll_list: ScrollableList::new()
        }
    }

    pub(crate) fn update_env_vars(&mut self, context: &ThreadContext) {
        self.env_vars.clear();
        for i in 0..ENV_VAR_COUNT {
            let value = context.get_env_var(i).unwrap_or(0.0);
            let name = context.get_env_var_name(i).unwrap_or(String::from("N/A"));

            self.env_vars.push(format!("{}={}", name, value));
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