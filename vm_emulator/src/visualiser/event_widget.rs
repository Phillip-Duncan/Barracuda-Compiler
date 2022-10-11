use crossterm::event::KeyEvent;

pub(crate) trait EventWidget {
    fn on_key_event(&mut self, key: KeyEvent);
}