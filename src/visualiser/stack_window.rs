use crate::emulator::ThreadContext;
use tui::widgets::{ListState, Block, Borders, BorderType, ListItem, List};
use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Modifier, Style};

pub struct StackWindow {
    stack_state: ListState,
    stack: Vec<String>
}

impl StackWindow {
    pub(crate) fn new() -> Self {
        StackWindow {
            stack_state: ListState::default(),
            stack: Vec::new()
        }
    }

    pub(crate) fn update_stack(&mut self, context: &ThreadContext) {
        let stack = context.get_stack();

        self.stack.clear();
        for i in 0..stack.len() {
            self.stack.push(format!("{}: {}", i, stack[i]));
        }

        // Select the top of the stack
        if !self.stack.is_empty() {
            self.stack_state.select(Some(self.stack.len()-1));
        }
    }

    pub(crate) fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let block = Block::default()
            .title("Stack")
            .borders(Borders::ALL)
            .border_type(BorderType::Plain);

        let items: Vec<ListItem>= self.stack.iter().map(|i| ListItem::new(i.as_ref())).collect();
        let list = List::new(items)
            .block(block)
            .highlight_symbol("> ");
        f.render_stateful_widget(list, area, &mut self.stack_state);
    }
}