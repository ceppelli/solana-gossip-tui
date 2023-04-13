use super::{events::Event, State, States};
use crate::app::Context;
use crate::ui::core::{centered_rect, clear_box, draw_box};
use crossterm::event::KeyCode;
use tui::{backend::Backend, Frame};

pub struct UnknownState;

impl State for UnknownState {
    fn on_event(&mut self, event: Event, ctx: &mut Context) -> Option<States> {
        if let Event::Key {
            key_code: KeyCode::Esc,
        } = event
        {
            Some(States::Home)
        } else {
            ctx.debug(format!("[UnknownS] on_event {event:?} not match"));
            None
        }
    }

    fn ui<B: Backend>(&self, f: &mut Frame<B>, _ctx: &mut Context) {
        let size = f.size();
        draw_box(f, size, " Unknow State ");

        let area = centered_rect(80, 40, size);
        clear_box(f, area); //this clears out the background
        draw_box(f, area, " Popup");
    }

    fn help_text(&self) -> &str {
        r##"
        ESC   -> back
        "##
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;
    use tui::{backend::TestBackend, buffer::Buffer, Terminal};

    #[test]
    fn test_unknown_state() -> Result<(), String> {
        let mut ctx = Context::new(String::from("APP_ID"), String::from("APP_VERSION"));

        let event = Event::Key {
            key_code: KeyCode::Esc,
        };

        let mut unkwnown = UnknownState;
        let to_state = unkwnown.on_event(event, &mut ctx);

        assert_eq!(to_state, Some(States::Home));

        Ok(())
    }

    #[test]
    fn test_ui() {
        let backend = TestBackend::new(7, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut ctx = Context::new_for_testing();

        let state = UnknownState;

        terminal
            .draw(|f| {
                state.ui(f, &mut ctx);
            })
            .unwrap();

        #[rustfmt::skip]
    let expected = Buffer::with_lines(vec![
      " Unkn─╮",
      " Po─╮ │",
      "│     │",
      "╰─────╯"
      ]);

        terminal.backend().assert_buffer(&expected);
    }

    #[test]
    fn test_state_help() -> Result<(), String> {
        let state = UnknownState;
        assert_eq!(state.help_text().len(), 31);

        Ok(())
    }
}
