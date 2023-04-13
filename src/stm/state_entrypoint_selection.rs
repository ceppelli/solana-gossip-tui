use super::{events::Event, State, States};
use crate::app::Context;
use crate::ui::core::{centered_rect, draw_box};
use crate::ui::list_stateful_widget::draw_stateful_list;
use crossterm::event::KeyCode;
use tui::{backend::Backend, Frame};

pub struct EntrypointSelectionState {
  pub on_enter_first: bool,
}

impl Default for EntrypointSelectionState {
  fn default() -> Self {
    Self { on_enter_first: true }
  }
}

impl EntrypointSelectionState {}

impl State for EntrypointSelectionState {
  fn on_enter_once(&mut self, ctx: &mut Context) {
    self.on_enter_first = false;

    for s in &ctx.model.entrypoints {
      ctx.model.entrypoints_stateful.push(s.to_string());
    }
  }

  fn on_event(&mut self, event: Event, ctx: &mut Context) -> Option<States> {
    match event {
      Event::Key { key_code: KeyCode::Down } => {
        ctx.model.entrypoints_stateful.next();
        None
      },
      Event::Key { key_code: KeyCode::Up } => {
        ctx.model.entrypoints_stateful.previous();
        None
      },
      Event::Key { key_code: KeyCode::Left } => {
        ctx.model.entrypoints_stateful.unselect();

        None
      },
      Event::Key { key_code: KeyCode::Enter } => {
        if let Some(index) = ctx.model.entrypoints_stateful.state.selected() {
          let entrypoint = ctx.model.entrypoints_stateful.items.get(index);

          if entrypoint.is_some() {
            ctx.model.entrypoint = entrypoint.cloned();
            return Some(States::Home);
          }
        }

        None
      },
      _ => {
        ctx.debug(format!("[HomeS] on_event {event:?} not match"));
        None
      },
    }
  }

  fn ui<B: Backend>(&self, f: &mut Frame<B>, ctx: &mut Context) {
    let size = f.size();
    draw_box(f, size, " Entrypoint Selection ");

    let bbox = centered_rect(75, 70, size);

    draw_stateful_list(
      f,
      bbox,
      " hosts ",
      &mut ctx.model.entrypoints_stateful,
      false,
    );
  }

  fn help_text(&self) -> &str {
    r##"
      ESC    -> back

      UP     -> previous
      DOWN   -> next
      LEFT   -> deselect
      ENTER  -> select

      D      -> show Debug
      q      -> Quit program
    "##
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crossterm::event::KeyCode;
  use tui::{backend::TestBackend, buffer::Buffer, Terminal};

  #[test]
  fn test_home_state() -> Result<(), String> {
    let mut ctx = Context::new_for_testing();

    let mut state = EntrypointSelectionState::default();

    let event = Event::Key { key_code: KeyCode::Esc };
    let to_state = state.on_event(event, &mut ctx);
    assert_eq!(to_state, None);

    let event = Event::Key { key_code: KeyCode::Down };
    let to_state = state.on_event(event, &mut ctx);
    assert_eq!(to_state, None);

    let event = Event::Key { key_code: KeyCode::Up };
    let to_state = state.on_event(event, &mut ctx);
    assert_eq!(to_state, None);

    let event = Event::Key { key_code: KeyCode::Left };
    let to_state = state.on_event(event, &mut ctx);
    assert_eq!(to_state, None);

    let event = Event::Key { key_code: KeyCode::Enter };
    let to_state = state.on_event(event, &mut ctx);
    assert_eq!(to_state, None);

    Ok(())
  }

  #[test]
  fn test_ui() {
    let backend = TestBackend::new(7, 4);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut ctx = Context::new_for_testing();

    let state = EntrypointSelectionState::default();

    terminal
      .draw(|f| {
        state.ui(f, &mut ctx);
      })
      .unwrap();

    #[rustfmt::skip]
    let expected = Buffer::with_lines(vec![
      "┌ ho┐─╮",
      "└───┘ │",
      "│     │",
      "╰─────╯"
      ]);

    terminal.backend().assert_buffer(&expected);
  }

  #[test]
  fn test_state_help() -> Result<(), String> {
    let state = EntrypointSelectionState::default();
    assert_eq!(state.help_text().len(), 178);

    Ok(())
  }
}
