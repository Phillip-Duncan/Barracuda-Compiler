use tui::widgets::{ListItem, List, ListState, Block};
use tui::layout::Rect;
use tui::Frame;
use tui::style::Style;
use tui::backend::Backend;


pub(crate) struct ScrollableList {
    expected_area: Option<Rect>,

    style: Option<Style>,
    highlight_style: Option<Style>,
    highlight_symbol: Option<String>,

    list_state: ListState,
    scroll_offset: usize
}

// Constructor for a new scrollable list
impl ScrollableList {

    pub fn new() -> Self {
        ScrollableList {
            expected_area: None,
            style: None,
            highlight_style: None,
            highlight_symbol: None,
            list_state: ListState::default(),
            scroll_offset: 0
        }
    }

    pub fn style(&mut self, style: Style) {
        self.style = Some(style);
    }

    pub fn highlight_style(&mut self, style: Style) {
        self.highlight_style = Some(style);
    }

    pub fn highlight_symbol(&mut self, symbol: String) {
        self.highlight_symbol = Some(symbol);
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.list_state.select(index);
    }

    pub fn selected(&mut self) -> Option<usize> {
        self.list_state.selected()
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset = self.scroll_offset - 1
        }
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scroll_down(&mut self) {
        // If scroll offset is greater than the item list it is truncated during the draw call.
        self.scroll_offset = self.scroll_offset + 1;
    }

    fn get_scroll_range(list_height: usize, scroll_offset: usize, frame_height: usize) -> (usize, usize) {
        let scroll_limit = ((list_height as i64)-(frame_height as i64)).max(0) as usize;
        let start = scroll_offset.max(0).min(scroll_limit);
        let end = (start+frame_height).max(start).min(list_height);
        (start, end)
    }

    fn truncate_list_by_scroll_offset(list_items: Vec<ListItem>, scroll_offset: usize, frame_height: usize) -> Vec<ListItem> {
        let list_height = list_items.len();
        let scroll_range = ScrollableList::get_scroll_range(list_height, scroll_offset, frame_height);

        list_items[scroll_range.0..scroll_range.1].iter().cloned().collect()
    }

    pub(crate) fn draw<B: Backend>(&mut self, f: &mut Frame<B>, list_items: Vec<ListItem>, area: Rect, block: Option<Block>) {
        // Set area height
        if let Some(block) = block.clone() {
            self.expected_area = Some(block.inner(area).clone());
        } else {
            self.expected_area = Some(area.clone());
        }

        // Update drawing cache members

        let mut list_state = ScrollableList::convert_list_state(&self.list_state, &list_items, self.scroll_offset, self.expected_area.unwrap());
        let scroll_limit = ((list_items.len() as i64) - (self.expected_area.unwrap().height as i64)).max(0) as usize;
        self.scroll_offset = self.scroll_offset.max(0).min(scroll_limit);


        let list_items = ScrollableList::truncate_list_by_scroll_offset(list_items, self.scroll_offset, self.expected_area.unwrap().height as usize);
        let mut list = List::new(list_items);

        if let Some(block) = block {
            list = list.block(block);
        }

        if let Some(style) = self.style {
            list = list.style(style);
        }

        if let Some(style) = self.highlight_style {
            list = list.highlight_style(style);
        }

        if let Some(symbol) = &self.highlight_symbol {
            list = list.highlight_symbol(symbol.as_str().clone());
        }

        f.render_stateful_widget(list, area, &mut list_state);
    }

    fn convert_list_state(prev_state: &ListState, list_items: &Vec<ListItem>, scroll_offset: usize, available_area : Rect) -> ListState {
        match prev_state.selected() {
            Some(selected) => {
                let mut new_state = ListState::default();
                let scroll_range = ScrollableList::get_scroll_range(list_items.len(), scroll_offset, available_area.height as usize);
                if selected >= scroll_range.0 && selected <= scroll_range.1 {
                    new_state.select(Some(selected - scroll_range.0));
                }
                new_state
            },
            None => {
                ListState::default()
            }
        }
    }
}