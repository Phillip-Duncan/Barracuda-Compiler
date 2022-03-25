use tui::widgets::{ListItem, ListState, Block, Borders, BorderType, List};
use crate::emulator::ThreadContext;
use crate::emulator::instructions::MathStackInstructions::{VALUE, OP};
use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Style, Modifier};

pub struct InstructionsWindow {
    instructions_state: ListState,
    instructions: Vec<String>
}

impl InstructionsWindow {
    pub(crate) fn new() -> Self {
        InstructionsWindow {
            instructions_state: ListState::default(),
            instructions: Vec::new()
        }
    }

    pub(crate) fn update_instructions(&mut self, context: &ThreadContext) {
        let values = context.get_values();
        let instructions = context.get_instructions();
        let operations = context.get_operations();

        self.instructions.clear();
        for i in 0..instructions.len() {
            let index = instructions.len()-1-i;
            let instruction_str = match instructions[index] {
                VALUE => {
                    format!("VALUE={}", values[index])
                },
                OP => {
                    format!("{:?}", operations[index])
                },
                _ => {
                    format!("{:?}", instructions[index])
                }
            };
            self.instructions.push(format!("{}: {}", i, instruction_str));
        }
    }

    pub(crate) fn update_program_counter(&mut self, context: &ThreadContext) {
        let program_counter = context.get_pc();
        self.instructions_state.select(Some(program_counter))
    }

    pub(crate) fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let block = Block::default()
            .title("Instructions")
            .borders(Borders::ALL)
            .border_type(BorderType::Plain);

        let items: Vec<ListItem>= self.instructions.iter().map(|i| ListItem::new(i.as_ref())).collect();
        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");
        f.render_stateful_widget(list, area, &mut self.instructions_state);
    }
}