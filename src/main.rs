mod app;
mod common;
mod logic;
mod protocol;
mod stm;
mod terminal;
mod transport;
mod ui;
mod utils;

use std::net::SocketAddr;
use std::{io, sync::mpsc::Receiver, time::Duration};

use crossterm::event::{self, Event, KeyCode};

use tui::{backend::Backend, Terminal};

use crate::app::AppContext;
use crate::common::{init_threads, Data};
use crate::logic::RECV_TIMEOUT;
use crate::protocol::LegacyVersion2;
use crate::stm::{events, stm_main::MainStm, States};
use crate::transport::CtrlCmd;

const APP_ID: &str = "solana_gossip_tui";
const APP_VERSION: &str = "0.0.1+";

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // initialize terminal state
  let mut xterm = terminal::XTerminal::new()?;

  // initialize app context and state machine
  let mut ctx = AppContext::new(String::from(APP_ID), String::from(APP_VERSION));
  let mut stm = MainStm::new("stm", true);

  let res = run_app(&mut xterm.terminal, &mut ctx, &mut stm, true);

  // check for errors
  if let Err(err) = res {
    println!("[main] {:?} {:?}", ctx.info(), err)
  }

  // restore terminal state
  xterm.restore()?;

  Ok(())
}

fn run_app<B: Backend>(
  terminal: &mut Terminal<B>,
  ctx: &mut AppContext,
  stm: &mut MainStm,
  looping: bool,
) -> io::Result<()> {
  // reset the state machine
  stm.switch_state(States::EntrypointSelection, ctx);

  let mut data_rx: Option<Receiver<Data>> = None;

  loop {
    terminal.draw(|f| stm.draw(f, ctx))?;

    if !looping {
      return Ok(());
    }

    if event::poll(Duration::from_millis(200))? {
      if let Event::Key(key) = event::read()? {
        stm.on_event(events::Event::Key { key_code: key.code }, ctx);

        if let KeyCode::Char('q') = key.code {
          for ctrl_tx in &ctx.ctrl_txs {
            ctrl_tx.send(CtrlCmd::Stop).unwrap_or(());
          }
          return Ok(());
        } else if KeyCode::Char('c') == key.code
          && stm.current_st == States::Home
          && data_rx.is_none()
        {
          let res = init_threads(ctx);
          if let Ok((rx, _t_hdls)) = res {
            data_rx = Some(rx);
          }
        } else if KeyCode::Char('d') == key.code
          && stm.current_st == States::Home
          && data_rx.is_some()
        {
          for ctrl_tx in &ctx.ctrl_txs {
            ctrl_tx.send(CtrlCmd::Stop).unwrap_or(());
          }

          data_rx = None;
        }
      }
    }

    if let Some(ref data_rx) = data_rx {
      if let Ok(data) = data_rx.recv_timeout(RECV_TIMEOUT) {
        match data {
          Data::LegacyContactInfo(info) => {
            fn format_ip(addr: SocketAddr) -> String {
              format!("{}", addr.ip())
            }

            fn format_port(addr: SocketAddr) -> String {
              format!("{}", addr.port())
            }

            let rows = ctx
              .model
              .home_stateful_table
              .items
              .iter()
              .filter(|row| -> bool {
                let id = &row[2];

                id == &info.id.to_string()
              })
              .collect::<Vec<_>>();

            if rows.is_empty() {
              let row = vec![
                format!("{}", format_ip(info.gossip)),         // "IP",
                format!("{}", info.wallclock),                 // "Age(ms)",
                format!("{:?}", info.id),                      // "Node Identifier",
                format!(" - ",),                               // "Version",
                format!("{}", format_port(info.gossip)),       // "Gossip",
                format!("{}", format_port(info.tpu_vote)),     // "TPUvote",
                format!("{}", format_port(info.tpu)),          // "TPU",
                format!("{}", format_port(info.tpu_forwards)), // "TPUfwd",
                format!("{}", format_port(info.tvu)),          // "TVU",
                format!("{}", format_port(info.tvu_forwards)), // "TVUfwd",
                format!("{}", format_port(info.repair)),       // "Repair",
                format!("{}", format_port(info.serve_repair)), // "ServeR",
                format!("{}", info.shred_version),             // "ShredVer",
              ];
              ctx.model.home_stateful_table.push_row(row);
            }
          },
          Data::Version(version) => {
            fn format_version(version: LegacyVersion2) -> String {
              format!("{}.{}.{}", version.major, version.minor, version.patch)
            }

            let at_index = ctx
              .model
              .home_stateful_table
              .items
              .iter()
              .position(|row| -> bool {
                let id = &row[2];

                id == &version.from.to_string()
              });

            if let Some(index) = at_index {
              let row = &mut ctx.model.home_stateful_table.items[index];

              let _ = std::mem::replace(&mut row[3], format_version(version.version));
            }
          },
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_format_version() {
    let mut items = vec![
      vec!["row_1x1", "row_1x2", "row_1x3"],
      vec!["row_2x1", "row_2x2", "row_2x3"],
      vec!["row_3x1", "row_3x2", "row_3x3"],
      vec!["row_4x1", "row_4x2", "row_4x3"],
    ]
    .into_iter()
    .map(|row| {
      row
        .into_iter()
        .map(|cell| cell.to_string())
        .collect::<Vec<String>>()
    })
    .collect::<Vec<Vec<String>>>();

    let cell = "row_3x3".to_string();

    let at_index = items.iter().position(|row| -> bool {
      let id = &row[2];

      id == &cell
    });

    assert_eq!(at_index, Some(2));

    match at_index {
      Some(index) => {
        let row = &mut items[index];

        let eq = vec!["row_3x1", "row_3x2", "row_3x3"]
          .into_iter()
          .map(|cell| cell.to_string())
          .collect::<Vec<String>>();

        assert_eq!(row, &eq);

        let _ = std::mem::replace(&mut row[0], "XXX".to_string());
        let _ = std::mem::replace(&mut row[1], "YYY".to_string());

        let eq = vec!["XXX", "YYY", "row_3x3"]
          .into_iter()
          .map(|cell| cell.to_string())
          .collect::<Vec<String>>();

        assert_eq!(items[index], eq);
      },
      None => {},
    }
  }
}
