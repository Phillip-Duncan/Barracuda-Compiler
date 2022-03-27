use std::io::{Cursor, BufRead};
use tui::text::Text;
use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::widgets::{Block, Borders, BorderType, Paragraph};
use tui::style::{Style, Color, Modifier};
use std::borrow::BorrowMut;
use crate::visualiser::event_widget::EventWidget;
use crossterm::event::{KeyEvent, KeyCode};

pub struct ProgramOutputWindow {
    scroll_offset: u16,
    last_output_len: usize
}

impl ProgramOutputWindow {
    pub fn new() -> Self {
        ProgramOutputWindow {
            scroll_offset: 0,
            last_output_len: 0
        }
    }

    fn get_program_output_as_span(mut program_out: Cursor<Vec<u8>>) -> Text<'static> {
        let mut text: Text = Text::from("");
        program_out.set_position(0);
        for line in program_out.lines() {
            let mut line_text: String = line.unwrap();
            line_text.push('\n');
            if line_text.starts_with("ERROR") {
                let style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
                text.extend(Text::styled(line_text, style));
            } else {
                text.extend(Text::raw(line_text));
            }
        }

        text
    }

    pub(crate) fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, output_buffer: Cursor<Vec<u8>>, highlighted: bool) {
        let mut block = Block::default()
            .title("Program Output")
            .borders(Borders::ALL);

        if highlighted {
            block = block.border_type(BorderType::Thick);
        }

        let inner_area = block.inner(area);
        let text = ProgramOutputWindow::get_program_output_as_span(output_buffer);

        // Set scroll offset to bottom of paragraph if size has changed since last draw call.
        if text.height() != self.last_output_len {
            self.last_output_len = text.height();
            self.scroll_offset = (self.last_output_len as u16 - inner_area.height.min(self.last_output_len as u16));
        } else {
            // Truncate scroll offset to scroll window. Only known during draw call
            self.scroll_offset = self.scroll_offset.min((self.last_output_len as i32 - inner_area.height as i32).max(0) as u16)
        }

        let paragraph = Paragraph::new(text).block(block).scroll((self.scroll_offset, 0));

        f.render_widget(paragraph, area);
    }
}

impl EventWidget for ProgramOutputWindow {
    fn on_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
            },
            KeyCode::Down => {
                self.scroll_offset += 1;
            },
            _ => {}
        }
    }
}