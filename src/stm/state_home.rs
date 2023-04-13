use super::{events::Event, State, States};
use crate::ui::core::draw_box;
use crate::ui::list_stateful_widget::draw_stateful_list;
use crate::ui::table_stateful_widget::draw_stateful_table;
use crate::{app::Context, ui::core::layout_columns_70_30};
use crossterm::event::KeyCode;
use tui::{backend::Backend, layout::Constraint, Frame};

pub struct HomeState {
  pub on_enter_first: bool,
}

impl Default for HomeState {
  fn default() -> Self {
    Self { on_enter_first: true }
  }
}

impl HomeState {}

impl State for HomeState {
  fn on_enter_once(&mut self, _ctx: &mut Context) {
    self.on_enter_first = false;
  }

  fn on_event(&mut self, event: Event, ctx: &mut Context) -> Option<States> {
    match event {
      Event::Key { key_code: KeyCode::Down } => {
        ctx.model.home_stateful_table.next_row();
        None
      },
      Event::Key { key_code: KeyCode::Up } => {
        ctx.model.home_stateful_table.previous_row();
        None
      },
      Event::Key { key_code: KeyCode::Left } => {
        ctx.model.home_stateful_table.unselect();

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
    draw_box(f, size, " Solana Home ");

    let bboxs = layout_columns_70_30(size);

    let title = format!("Nodes Info [{:?}]", ctx.model.entrypoint);

    let withs = [
      Constraint::Percentage(14),
      Constraint::Percentage(6),
      Constraint::Percentage(16),
      Constraint::Percentage(6),
      Constraint::Percentage(6),
      Constraint::Percentage(6),
      Constraint::Percentage(6),
      Constraint::Percentage(6),
      Constraint::Percentage(6),
      Constraint::Percentage(6),
      Constraint::Percentage(6),
      Constraint::Percentage(6),
      Constraint::Percentage(6),
    ];
    draw_stateful_table(
      f,
      bboxs[0],
      &title,
      &[
        "IP",
        "Age(ms)",
        "Node Identifier",
        "Version",
        "Gossip",
        "TPUvote",
        "TPU",
        "TPUfwd",
        "TVU",
        "TVUfwd",
        "Repair",
        "ServeR",
        "ShredVer",
      ],
      &withs,
      &mut ctx.model.home_stateful_table,
    );

    draw_stateful_list(
      f,
      bboxs[1],
      " stats ",
      &mut ctx.model.home_stats_stateful_list,
      false,
    );
  }

  fn help_text(&self) -> &str {
    r##"
      ESC    -> back
      c      -> connect to entrypoint
      d      -> disconnect from entrypoint

      UP     -> previous
      DOWN   -> next
      LEFT   -> deselect

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

    let mut state = HomeState::default();

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

    Ok(())
  }

  #[test]
  fn test_ui() {
    let backend = TestBackend::new(7, 4);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut ctx = Context::new_for_testing();

    let state = HomeState::default();

    terminal
      .draw(|f| {
        state.ui(f, &mut ctx);
      })
      .unwrap();

    #[rustfmt::skip]
    let expected = Buffer::with_lines(vec![
      " Sola─╮",
      "│ ┌┐┌ │",
      "│ └┘└ │",
      "╰─────╯"
      ]);

    terminal.backend().assert_buffer(&expected);
  }

  #[test]
  fn test_state_help() -> Result<(), String> {
    let state = HomeState::default();
    assert_eq!(state.help_text().len(), 236);

    Ok(())
  }
}
