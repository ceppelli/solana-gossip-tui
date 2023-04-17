use crate::app::Context;
use crate::stm::events::Event;
use tui::{backend::Backend, Frame};

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum States {
    Unknown,
    PreviousOne,
    NextOne,
    Debug,

    EntrypointSelection,
    Home,
    Help,
}

trait State {
    // this method is called only the first time when the state machine transitioning to this state.
    #[allow(unused_variables)]
    fn on_enter_once(&mut self, ctx: &mut Context) {}

    // this method returns an Option<States>. If it is not NONE the optional States value
    // is the new state which the State Machine will transition
    #[allow(unused_variables)]
    fn on_event(&mut self, event: Event, ctx: &mut Context) -> Option<States> {
        None
    }

    fn ui<B: Backend>(&self, f: &mut Frame<B>, _ctx: &mut Context);

    fn help_text(&self) -> &str {
        r##"
    Help
    "##
    }
}

pub(crate) mod events;
mod state_debug;
mod state_entrypoint_selection;
mod state_help;
mod state_home;
mod state_unknown;
pub(crate) mod stm_main;
