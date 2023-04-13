use super::{events::Event, State, States};
use crate::app::Context;
use crossterm::event::KeyCode;
use tui::{backend::Backend, Frame};

pub struct HelpState;

impl State for HelpState {
  fn on_event(&mut self, event: Event, ctx: &mut Context) -> Option<States> {
    if let Event::Key { key_code: KeyCode::Esc } = event {
      Some(States::PreviousOne)
    } else {
      ctx.debug(format!("[HelpS] on_event {event:?} not match"));
      None
    }
  }

  fn ui<B: Backend>(&self, _f: &mut Frame<B>, _ctx: &mut Context) {}
}

#[cfg(test)]
mod tests {
  use super::*;
  use crossterm::event::KeyCode;

  #[test]
  fn test_home_state() -> Result<(), String> {
    let mut ctx = Context::new_for_testing();

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
