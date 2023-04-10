use super::{events::Event, State, States};
use crate::app::AppContext;
use crossterm::event::KeyCode;
use tui::{backend::Backend, Frame};

pub struct HelpState;

impl State for HelpState {
  #[allow(clippy::let_and_return)]
  fn on_event(&mut self, event: Event, ctx: &mut AppContext) -> Option<States> {
    let to_state = match event {
      Event::Key { key_code: KeyCode::Esc } => Some(States::PreviousOne),
      _ => {
        ctx.debug(format!("[HelpS] on_event {:?} not match", event));
        None
      },
    };

    to_state
  }

  fn ui<B: Backend>(&self, _f: &mut Frame<B>, _ctx: &mut AppContext) {}
}

#[cfg(test)]
mod tests {
  use super::*;
  use crossterm::event::KeyCode;

  #[test]
  fn test_home_state() -> Result<(), String> {
    let mut ctx = AppContext::new_for_testing();

    let mut help = HelpState;

    let event = Event::Key { key_code: KeyCode::Char('*') };
    let to_state = help.on_event(event, &mut ctx);
    assert_eq!(to_state, None);

    Ok(())
  }

  #[test]
  fn test_state_help() -> Result<(), String> {
    let state = HelpState;
    assert_eq!(state.help_text().len(), 14);

    Ok(())
  }
}
