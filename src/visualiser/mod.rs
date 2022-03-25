mod instructions_window;
mod env_vars_window;
mod stack_window;

use crate::emulator::ThreadContext;
use crate::visualiser::instructions_window::InstructionsWindow;
use crate::visualiser::stack_window::StackWindow;
use crate::visualiser::env_vars_window::EnvVarWindow;
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
use std::cell::{RefCell, RefMut};
use tui::text::{Span, Spans};
use tui::style::{Style, Color, Modifier};


pub(crate) struct MathStackVisualiser {
    thread_context: ThreadContext,
    instruction_window: InstructionsWindow,
    stack_window: StackWindow,
    env_vars_window: EnvVarWindow,

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
                        KeyCode::Left => self.on_key_left(),
                        KeyCode::Right => self.on_key_right(),
                        KeyCode::Up => self.on_key_up(),
                        KeyCode::Down => self.on_key_down(),
                        KeyCode::Esc => self.should_quit=true,
                        KeyCode::Char(c) => self.on_key(c),
                        _ => {}
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

    fn on_key_up(&mut self) {

    }

    fn on_key_down(&mut self) {

    }

    fn on_key_left(&mut self) {

    }

    fn on_key_right(&mut self) {

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
                let mut out = self.program_output.borrow_mut() as RefMut<dyn Write>;
                write!(out, "ERROR: {}({:?})\n", error.to_string(), error.kind());
            }
        }
        self.instruction_window.update_program_counter(&self.thread_context);
        self.stack_window.update_stack(&self.thread_context);
        self.env_vars_window.update_env_vars(&self.thread_context);
    }

    fn on_tick(&mut self) {

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
        self.draw_output_window(f, chunks[1]);
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

        self.instruction_window.draw(f, chunks[0]);
        self.stack_window.draw(f, chunks[1]);
        self.env_vars_window.draw(f, chunks[2]);
    }

    fn get_program_output_as_span(&mut self) -> Vec<Span> {
        let mut text: Vec<Span> = Vec::new();
        let mut program_out = self.program_output.borrow_mut();
        program_out.seek(SeekFrom::Start(0));
        for line in program_out.get_ref().lines() {
            let line_text: String = line.unwrap();
            if line_text.starts_with("ERROR") {
                let style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
                text.push(Span::styled(line_text, style));
            } else {
                text.push(Span::raw(line_text));
            }
        }

        text
    }

    fn draw_output_window<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let block = Block::default()
            .title("Program Output")
            .borders(Borders::ALL);

        let text = Spans::from(self.get_program_output_as_span());
        let paragraph = Paragraph::new(text).block(block);

        f.render_widget(paragraph, area);
    }
}