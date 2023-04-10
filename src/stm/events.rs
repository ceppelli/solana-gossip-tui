use crossterm::event::KeyCode;

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum Event {
  Reset,
  Key { key_code: KeyCode },
  Debug { message: String },
}

// tests
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_event_enum() {
    let event = Event::Key { key_code: KeyCode::Esc };

    assert!(matches!(event, Event::Key { key_code: _ }));
    assert!(matches!(event, Event::Key { key_code: KeyCode::Esc }));
  }

  #[test]
  fn test_events() -> Result<(), String> {
    let event = Event::Key { key_code: KeyCode::Esc };
    assert!(matches!(event, Event::Key { key_code: KeyCode::Esc }));

    let event = event.clone();
    assert!(matches!(event, Event::Key { key_code: KeyCode::Esc }));

    let event_debug = Event::Debug { message: String::from("INFO") };
    assert!(matches!(event_debug, Event::Debug { message: _ }));

    Ok(())
  }
}
