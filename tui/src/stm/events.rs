use crossterm::event::KeyCode;

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum Event {
    Reset,
    Key { key_code: KeyCode },
    Debug { message: String },
}
