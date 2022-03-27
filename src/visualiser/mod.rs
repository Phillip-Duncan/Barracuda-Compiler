mod instructions_window;
mod env_vars_window;
mod stack_window;
mod scrollable_list;
mod program_output_window;
mod event_widget;

use crate::emulator::ThreadContext;
use crate::visualiser::instructions_window::InstructionsWindow;
use crate::visualiser::stack_window::StackWindow;
use crate::visualiser::env_vars_window::EnvVarWindow;
use crate::visualiser::program_output_window::ProgramOutputWindow;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io, time::{Duration, Instant}, thread};
use tui::{backend::{Backend, CrosstermBackend}, Terminal, Frame};
use tui::widgets::{Block, Borders, BorderType, Paragraph};
use tui::layout::{Layout, Direction, Constraint, Rect};
use std::io::{Cursor, Seek, BufRead, SeekFrom, Write};
use std::rc::Rc;
use std::cell::{RefCell, RefMut, Ref};
use tui::text::{Span, Spans, Text};
use tui::style::{Style, Color, Modifier};
use crossterm::event::KeyEvent;
use std::ops::Deref;
use crate::visualiser::VisualiserUIWindows::{OUTPUT, INSTRUCTIONS, STACK, ENV_VARS};
use std::borrow::BorrowMut;
use crate::visualiser::event_widget::EventWidget;


#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum VisualiserUIWindows {
    INSTRUCTIONS,
    STACK,
    ENV_VARS,
    OUTPUT
}

impl VisualiserUIWindows {
    fn next(&self) -> VisualiserUIWindows {
        match self {
            VisualiserUIWindows::INSTRUCTIONS => {VisualiserUIWindows::STACK}
            VisualiserUIWindows::STACK => {VisualiserUIWindows::ENV_VARS}
            VisualiserUIWindows::ENV_VARS => {VisualiserUIWindows::OUTPUT}
            VisualiserUIWindows::OUTPUT => {VisualiserUIWindows::INSTRUCTIONS}
        }
    }
}



pub(crate) struct MathStackVisualiser {
    thread_context: ThreadContext,
    instruction_window: InstructionsWindow,
    stack_window: StackWindow,
    env_vars_window: EnvVarWindow,
    program_output_window: ProgramOutputWindow,

    focus_window: VisualiserUIWindows,

    program_output: Rc<RefCell<Cursor<Vec<u8>>>>,
    should_quit: bool
}


impl MathStackVisualiser {
    pub(crate) fn new(context: ThreadContext) -> Self {
        MathStackVisualiser {
            thread_context: context,
            instruction_window: InstructionsWindow::new(),
            stack_window: StackWindow::new(),
            env_vars_window: EnvVarWindow::new(),
            program_output_window: ProgramOutputWindow::new(),
            focus_window: VisualiserUIWindows::INSTRUCTIONS,
            program_output: Rc::new(RefCell::new(Cursor::new(Vec::new()))),
            should_quit: false
        }
    }

    fn init_ui(&mut self) {
        self.instruction_window.update_instructions(&self.thread_context);
        self.instruction_window.update_program_counter(&self.thread_context);
        self.stack_window.update_stack(&self.thread_context);
        self.env_vars_window.update_env_vars(&self.thread_context);

        self.thread_context.set_output_stream(self.program_output.clone());
    }

    pub(crate) fn run(&mut self) -> Result<(), io::Error>{
        // Initialise ui
        self.init_ui();

        // Setup terminal for tui
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Run application
        self.run_app(&mut terminal, Duration::from_millis(200))?;

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>, tick_rate: Duration) -> io::Result<()>{
        let mut last_tick = Instant::now();
        loop {
            // Draw screen
            terminal.draw(|f| {self.draw(f)})?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            // Check events
            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Esc => self.should_quit=true,
                        KeyCode::Tab => {
                            self.focus_window = self.focus_window.next();
                        }
                        KeyCode::Char(c) => {
                            self.on_key(c);
                            self.forward_key_event(key);
                        },
                        _ => {
                            self.forward_key_event(key);
                        }
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                self.on_tick();
                last_tick = Instant::now();
            }

            if self.should_quit {
                return Ok(())
            }
        }
    }

    fn forward_key_event(&mut self, key: KeyEvent) {
        match self.focus_window {
            VisualiserUIWindows::INSTRUCTIONS => {}
            VisualiserUIWindows::STACK => {}
            VisualiserUIWindows::ENV_VARS => {self.env_vars_window.on_key_event(key)}
            VisualiserUIWindows::OUTPUT => {self.program_output_window.on_key_event(key)}
        }
    }

    fn on_key(&mut self, character: char) {
        match character {
            ' ' => {
                self.step_emulator();
            }
            _ => {}
        }
    }

    fn step_emulator(&mut self) {
        match self.thread_context.step() {
            Ok(_) => {}
            Err(error) => {
                let mut out = (&*self.program_output).borrow_mut();
                // Okay to panic if cannot write errors to program output
                write!(out, "ERROR: {}({:?})\n", error.to_string(), error.kind()).unwrap();
            }
        }

        self.instruction_window.update_instructions(&self.thread_context);
        self.instruction_window.update_program_counter(&self.thread_context);
        self.stack_window.update_stack(&self.thread_context);
        self.env_vars_window.update_env_vars(&self.thread_context);
    }

    fn on_tick(&mut self) {

    }

    fn in_focus(&self, window: VisualiserUIWindows) -> bool {
        self.focus_window == window
    }

    fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(70),
                    Constraint::Percentage(30)
                ].as_ref()
            )
            .split(f.size());

        self.draw_execution_window(f, chunks[0]);

        let output_buffer = self.program_output.borrow().deref().clone();
        self.program_output_window.draw(f, chunks[1], output_buffer, self.in_focus(OUTPUT));
    }

    fn draw_execution_window<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(35),
                    Constraint::Percentage(35),
                    Constraint::Percentage(30)
                ].as_ref()
            )
            .split(area);

        self.instruction_window.draw(f, chunks[0], self.in_focus(INSTRUCTIONS));
        self.stack_window.draw(f, chunks[1], self.in_focus(STACK));
        self.env_vars_window.draw(f, chunks[2], self.in_focus(ENV_VARS));
    }
}