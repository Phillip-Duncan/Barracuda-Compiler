use crate::emulator::{ThreadContext, ENV_VAR_COUNT};
use tui::widgets::{ListState, Block, Borders, BorderType, ListItem, List};
use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Modifier, Style};

pub struct EnvVarWindow {
    env_vars: Vec<String>
}

impl EnvVarWindow {
    pub(crate) fn new() -> Self {
        EnvVarWindow {
            env_vars: Vec::new()
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

    pub(crate) fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let block = Block::default()
            .title("Environment Vars")
            .borders(Borders::ALL)
            .border_type(BorderType::Plain);

        let items: Vec<ListItem>= self.env_vars.iter().map(|i| ListItem::new(i.as_ref())).collect();
        let list = List::new(items)
            .block(block);
        f.render_widget(list, area);
    }
}