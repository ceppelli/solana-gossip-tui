use super::{events::Event, State, States};
use crate::ui::core::{centered_rect, draw_box};
use crate::{app::Context, ui::list_stateful_widget::draw_stateful_list};
use crossterm::event::KeyCode;
use tui::{backend::Backend, Frame};

pub struct DebugState;

impl State for DebugState {
    fn on_event(&mut self, event: Event, ctx: &mut Context) -> Option<States> {
        match event {
            Event::Key {
                key_code: KeyCode::Esc,
            } => Some(States::PreviousOne),
            Event::Key {
                key_code: KeyCode::Down,
            } => {
                ctx.model.debug_messages_stateful.next();
                None
            }
            Event::Key {
                key_code: KeyCode::Up,
            } => {
                ctx.model.debug_messages_stateful.previous();
                None
            }
            Event::Key {
                key_code: KeyCode::Left,
            } => {
                ctx.model.debug_messages_stateful.unselect();
                None
            }
            _ => None,
        }
    }

    fn ui<B: Backend>(&self, f: &mut Frame<B>, ctx: &mut Context) {
        let size = f.size();
        draw_box(f, size, " Debug State ");

        let bbox = centered_rect(95, 90, size);
        draw_stateful_list(
            f,
            bbox,
            "debug messages",
            &mut ctx.model.debug_messages_stateful,
            true,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;
    use tui::{backend::TestBackend, buffer::Buffer, Terminal};

    #[test]
    fn test_debug_state() -> Result<(), String> {
        let mut ctx = Context::new_for_testing();

        let mut debug = DebugState;

        let event = Event::Key {
            key_code: KeyCode::Esc,
        };
        let to_state = debug.on_event(event, &mut ctx);
        assert_eq!(to_state, Some(States::PreviousOne));

        let event = Event::Key {
            key_code: KeyCode::Down,
        };
        let to_state = debug.on_event(event, &mut ctx);
        assert_eq!(to_state, None);

        let event = Event::Key {
            key_code: KeyCode::Up,
        };
        let to_state = debug.on_event(event, &mut ctx);
        assert_eq!(to_state, None);

        let event = Event::Key {
            key_code: KeyCode::Left,
        };
        let to_state = debug.on_event(event, &mut ctx);
        assert_eq!(to_state, None);

        Ok(())
    }

    #[test]
    fn test_ui() {
        let backend = TestBackend::new(7, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut ctx = Context::new_for_testing();

        let state = DebugState;

        terminal
            .draw(|f| {
                state.ui(f, &mut ctx);
            })
            .unwrap();

        #[rustfmt::skip]
    let expected = Buffer::with_lines(vec![
      "┌debu┐╮",
      "│    ││",
      "└────┘│",
      "╰─────╯"
      ]);

        terminal.backend().assert_buffer(&expected);
    }

    #[test]
    fn test_state_help() -> Result<(), String> {
        let state = DebugState;
        assert_eq!(state.help_text().len(), 14);

        Ok(())
    }
}
