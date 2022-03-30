use crate::visualiser::scrollable_list::ScrollableList;
use crate::emulator::ThreadContext;
use tui::backend::Backend;
use tui::Frame;
use tui::layout::{Rect, Layout, Direction, Constraint};
use tui::widgets::{ListItem, List, Block, Borders, ListState};
use std::fmt;
use tui::style::{Style, Modifier, Color};
use tui::text::{Spans, Span, Text};
use std::borrow::Borrow;

pub struct HeapViewer {
    current_memory_region: Option<u16>,
    byte_scroll_list: ScrollableList,
    region_viewer_rows: Vec<[String; 3]>,
    region_summaries: Vec<MemoryRegionSummary>,
    expected_area: Option<Rect>
}

struct MemoryRegionSummary {
    index: u16,
    size: usize
}

impl fmt::Debug for MemoryRegionSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Region {} ({:#X})", self.index, self.size)
    }
}

impl HeapViewer {
    pub(crate) fn new() -> Self {
        HeapViewer {
            current_memory_region: None,
            byte_scroll_list: ScrollableList::new(),
            region_viewer_rows: Vec::new(),
            region_summaries: Vec::new(),
            expected_area: None
        }
    }

    fn generate_region_summaries(context: &ThreadContext) -> Vec<MemoryRegionSummary> {
        let heap = context.get_heap();
        let mut summaries = Vec::new();

        for key in heap.get_memory_region_keys() {
            let region = heap.get_memory_region(*key).unwrap();
            summaries.push(MemoryRegionSummary {
                index: *key,
                size: region.borrow().len()
            });
        }

        summaries.sort_by(|a, b| a.index.cmp(&b.index));
        summaries
    }

    /// Generates the byte viewer text for displaying bytes in the memory region
    /// @context: Threadcontext with heap to generate from
    /// @region_index: Region index to generate for. Must be a valid address or will panic
    fn generate_region_byte_viewer_list(context: &ThreadContext, region_index: u16, line_width: u16) -> Vec<[String; 3]> {
        let mut viewer_text: Vec<[String; 3]> = Vec::new();

        let heap = context.get_heap();
        let region = heap.get_memory_region(region_index).unwrap().borrow();

        let bytes_per_line = ((line_width - 15) / 4) as usize;
        let mut byte_count: usize  = 0;
        for line_bytes in region.chunks(bytes_per_line) {
            let mut byte_segment = String::new();
            let mut char_segment = String::new();
            for byte in line_bytes {
                byte_segment = format!("{} {:02X}", byte_segment, *byte);

                let byte_char = if byte.is_ascii_graphic() {
                    *byte as char
                } else {
                    '.'
                };
                char_segment = format!("{}{}", char_segment, byte_char);

            }
            let address_segment = format!("{:012X}:", byte_count);
            let byte_segment = format!("{:1$} |", byte_segment, bytes_per_line*3);

            viewer_text.push([address_segment, byte_segment, char_segment]);
            byte_count += bytes_per_line;
        }

        viewer_text
    }

    pub(crate) fn update_heap_viewer(&mut self, context: &ThreadContext) {
        let heap = context.get_heap();
        self.region_summaries = Self::generate_region_summaries(context);

        // Update current_memory_region if invalid
        self.current_memory_region = match self.current_memory_region {
            Some(index) => {
                if heap.get_memory_region(index).is_some() {
                    Some(index)
                } else {
                    self.byte_scroll_list.scroll_to_top();
                    None
                }
            }
            None => None
        };

        // If no memory region is select, select first one
        if self.current_memory_region.is_none() && self.region_summaries.len() > 0 {
            self.current_memory_region = Some(self.region_summaries.first().unwrap().index)
        }

        self.region_viewer_rows.clear();

        if self.current_memory_region.is_some() {
            let region_index = self.current_memory_region.clone().unwrap();
            let area = self.expected_area.unwrap_or(Rect {
                x: 0,
                y: 0,
                width: 40,
                height: 0
            });
            self.region_viewer_rows = Self::generate_region_byte_viewer_list(context, region_index, area.width)
        }

    }

    pub(crate) fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(20),
                    Constraint::Percentage(80)
                ].as_ref()
            )
            .split(area);

        self.draw_region_viewer(f, chunks[0]);
        self.draw_byte_viewer(f, chunks[1]);
    }

    fn draw_byte_viewer<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        self.expected_area = Some(area.clone());
        let mut block = Block::default()
            .title("Region Hex Viewer")
            .borders(Borders::ALL);

        let mut viewer_spans:Vec<Spans> = Vec::new();
        for hex_row in self.region_viewer_rows.clone() {
            viewer_spans.push(Spans::from(vec![
                Span::styled(hex_row[0].clone(), Style::default().fg(Color::Magenta)),
                Span::styled(hex_row[1].clone(), Style::default().fg(Color::White)),
                Span::styled(hex_row[2].clone(), Style::default().fg(Color::Gray))
            ]));
        }

        let items: Vec<ListItem>= viewer_spans.iter()
            .map(|i| ListItem::new(Text::from(i.clone()))).collect();

        self.byte_scroll_list.draw(f, items, area, Some(block))
    }

    fn draw_region_viewer<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let mut block = Block::default()
            .title("Memory Allocations")
            .borders(Borders::ALL);

        let items: Vec<ListItem>= self.region_summaries.iter()
            .map(|i| ListItem::new(format!("{:?}", i))).collect();
        let list = List::new(items)
            .block(block)
            .highlight_symbol("> ")
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        let list_index = match self.region_summaries
            .binary_search_by(|region| region.index.cmp(&self.current_memory_region.unwrap_or(0))) {
            Ok(index) => {Some(index)}
            Err(_) => {None}
        };
        let mut list_state = ListState::default();
        list_state.select(list_index);

        f.render_stateful_widget(list, area, &mut list_state);
    }
}