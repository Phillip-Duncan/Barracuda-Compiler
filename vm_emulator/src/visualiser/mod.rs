mod instructions_window;
mod env_vars_window;
mod stack_window;
mod scrollable_list;
mod program_output_window;
mod event_widget;
mod heap_viewer;

use crate::emulator::ThreadContext;
use crate::visualiser::instructions_window::InstructionsWindow;
use crate::visualiser::stack_window::StackWindow;
use crate::visualiser::env_vars_window::EnvVarWindow;
use crate::visualiser::program_output_window::ProgramOutputWindow;
use crate::visualiser::heap_viewer::HeapViewer;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, time::{Duration, Instant}};
use tui::{backend::{Backend, CrosstermBackend}, Terminal, Frame};
use tui::layout::{Layout, Direction, Constraint, Rect};
use std::io::{Cursor, Write};
use std::rc::Rc;
use std::cell::{RefCell};
use crossterm::event::{KeyEvent, KeyModifiers};
use std::ops::Deref;
use crate::visualiser::VisualiserUIWindows::{OUTPUT, INSTRUCTIONS, STACK, ENV_VARS, HEAP};
use crate::visualiser::event_widget::EventWidget;
use tui::widgets::Tabs;
use tui::text::Spans;
use tui::style::{Style, Color};


#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum VisualiserUIWindows {
    INSTRUCTIONS,
    STACK,
    ENV_VARS,
    OUTPUT,
    HEAP
}

impl VisualiserUIWindows {
    fn next(&self) -> VisualiserUIWindows {
        match self {
            VisualiserUIWindows::INSTRUCTIONS => {VisualiserUIWindows::STACK}
            VisualiserUIWindows::STACK => {VisualiserUIWindows::ENV_VARS}
            VisualiserUIWindows::ENV_VARS => {VisualiserUIWindows::OUTPUT}
            VisualiserUIWindows::OUTPUT => {VisualiserUIWindows::INSTRUCTIONS}
            VisualiserUIWindows::HEAP => {VisualiserUIWindows::HEAP}
        }
    }

    fn next_tab(&self) -> VisualiserUIWindows {
        match self {
            INSTRUCTIONS => {VisualiserUIWindows::HEAP}
            STACK => {VisualiserUIWindows::HEAP}
            ENV_VARS => {VisualiserUIWindows::HEAP}
            OUTPUT => {VisualiserUIWindows::HEAP}
            HEAP => {VisualiserUIWindows::INSTRUCTIONS}
        }
    }

    fn help_message(&self) -> &str {
        match self {
            INSTRUCTIONS => "Scroll:Up/Down",
            STACK => "Scroll:Up/Down",
            ENV_VARS => "Scroll:Up/Down",
            OUTPUT => "Scroll:Up/Down",
            HEAP => "Switch Memory Region:Tab, Scroll:Up/Down"
        }
    }

    fn tab_index(&self) -> usize {
        match self {
            INSTRUCTIONS => 0,
            STACK => 0,
            ENV_VARS => 0,
            OUTPUT => 0,
            HEAP => 1
        }
    }
}



pub(crate) struct MathStackVisualiser {
    thread_context: ThreadContext,
    instruction_window: InstructionsWindow,
    stack_window: StackWindow,
    env_vars_window: EnvVarWindow,
    program_output_window: ProgramOutputWindow,
    heap_window: HeapViewer,

    focus_window: VisualiserUIWindows,

    program_output: Rc<RefCell<Cursor<Vec<u8>>>>,
    should_quit: bool,
    emulator_paused: bool
}


impl MathStackVisualiser {
    pub(crate) fn new(context: ThreadContext) -> Self {
        MathStackVisualiser {
            thread_context: context,
            instruction_window: InstructionsWindow::new(),
            stack_window: StackWindow::new(),
            env_vars_window: EnvVarWindow::new(),
            program_output_window: ProgramOutputWindow::new(),
            heap_window: HeapViewer::new(),
            focus_window: VisualiserUIWindows::INSTRUCTIONS,
            program_output: Rc::new(RefCell::new(Cursor::new(Vec::new()))),
            should_quit: false,
            emulator_paused: true
        }
    }

    fn init_ui(&mut self) {
        self.instruction_window.update_instructions(&self.thread_context);
        self.instruction_window.update_program_counter(&self.thread_context);
        self.stack_window.update_stack(&self.thread_context);
        self.env_vars_window.update_env_vars(&self.thread_context);

        self.thread_context.set_output_stream(self.program_output.clone());
    }

    fn partial_update_ui(&mut self) {
        self.instruction_window.update_instructions(&self.thread_context);
        self.instruction_window.update_program_counter(&self.thread_context);
        self.stack_window.update_stack(&self.thread_context);
        self.env_vars_window.update_env_vars(&self.thread_context);
        self.heap_window.update_heap_viewer(&self.thread_context);
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

            if !self.emulator_paused {
                self.run_emulator_until_timeout(timeout);
            }

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
                            self.forward_key_event(key);
                        },
                        KeyCode::BackTab => {
                            self.focus_window = self.focus_window.next_tab()
                        },
                        KeyCode::Enter => {
                            self.emulator_paused = !self.emulator_paused;
                        },
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
            VisualiserUIWindows::OUTPUT => {self.program_output_window.on_key_event(key)},
            VisualiserUIWindows::HEAP => {}
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

    fn step_emulator_without_update(&mut self) {
        match self.thread_context.step() {
            Ok(_) => {}
            Err(error) => {
                let mut out = (&*self.program_output).borrow_mut();
                // Okay to panic if cannot write errors to program output
                write!(out, "ERROR: {}({:?})\n", error.to_string(), error.kind()).unwrap();
            }
        }
    }

    fn step_emulator(&mut self) {
        self.step_emulator_without_update();
        self.partial_update_ui();
    }

    fn run_emulator_until_timeout(&mut self, timeout: Duration) {
        let start_time = Instant::now();
        while !self.thread_context.is_execution_finished() &&
            !timeout.checked_sub(start_time.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0)).is_zero() {
            self.step_emulator_without_update();
        }

        self.partial_update_ui();
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
                    Constraint::Length(1),
                    Constraint::Min(0),
                    Constraint::Length(1)
                ].as_ref()
            )
            .split(f.size());

        // Draw Tab Selector
        let tab_titles = ["Main", "Memory Heap"].iter().cloned().map(Spans::from).collect();
        let tab_index = self.focus_window.tab_index();
        let tabs = Tabs::new(tab_titles)
            .highlight_style(Style::default().fg(Color::Green))
            .select(tab_index);
        f.render_widget(tabs, chunks[0]);

        match tab_index {
            0 => self.draw_main_tab(f, chunks[1]),
            1 => self.heap_window.draw(f, chunks[1]),
            _ => {}
        }

    }

    fn draw_main_tab<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(70),
                    Constraint::Percentage(30)
                ].as_ref()
            )
            .split(area);

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