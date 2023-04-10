use super::{events::Event, State, States};
use crate::ui::core::draw_box;
use crate::ui::list_stateful_widget::draw_stateful_list;
use crate::ui::table_stateful_widget::draw_stateful_table;
use crate::{app::AppContext, ui::core::layout_columns_70_30};
use crossterm::event::KeyCode;
use tui::{backend::Backend, Frame};

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
  fn on_enter_once(&mut self, _ctx: &mut AppContext) {
    self.on_enter_first = false;
  }

  fn on_event(&mut self, event: Event, ctx: &mut AppContext) -> Option<States> {
    #[allow(clippy::let_and_return)]
    let to_state = match event {
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
        ctx.debug(format!("[HomeS] on_event {:?} not match", event));
        None
      },
    };

    to_state
  }

  fn ui<B: Backend>(&self, f: &mut Frame<B>, ctx: &mut AppContext) {
    let size = f.size();
    draw_box(f, size, " Solana Home ");

    let bboxs = layout_columns_70_30(size);

    let title = format!("Nodes Info [{:?}]", ctx.model.entrypoint);
    draw_stateful_table(
      f,
      bboxs[0],
      &title,
      vec![
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
      &mut ctx.model.home_stateful_table,
    );

    draw_stateful_list(
      f,
      bboxs[1],
      " stats ",
      &mut ctx.model.home_stats_stateful,
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
