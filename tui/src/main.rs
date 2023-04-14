mod app;
mod common;
mod logic;
mod stm;
mod terminal;
mod transport;
mod ui;

use std::{
    io,
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode};
use solana_gossip_proto::protocol::{LegacyContactInfo, LegacyVersion2};
use tui::{backend::Backend, Terminal};

use crate::{
    app::Context,
    common::{init_threads, Data},
    logic::RECV_TIMEOUT,
    stm::{events, stm_main::MainStm, States},
    transport::{CtrlCmd, Stats},
};

const APP_ID: &str = "solana_gossip_tui";
const APP_VERSION: &str = "0.0.1+";
const STATS_INTERVAL: Duration = Duration::from_millis(1000);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize terminal state
    let mut xterm = terminal::XTerminal::new()?;

    // initialize app context and state machine
    let mut ctx = Context::new(String::from(APP_ID), String::from(APP_VERSION));
    let mut stm = MainStm::new("stm", true);

    let res = run_app(&mut xterm.terminal, &mut ctx, &mut stm);

    // check for errors
    if let Err(err) = res {
        println!("[main] {:?} {err:?}", ctx.info());
    }

    // restore terminal state
    xterm.restore()?;

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    ctx: &mut Context,
    stm: &mut MainStm,
) -> io::Result<()> {
    // reset the state machine
    stm.switch_state(States::EntrypointSelection, ctx);

    let mut data_rx: Option<Receiver<Data>> = None;
    let mut stats_rx: Option<Receiver<Stats>> = None;
    let mut before = Instant::now();

    loop {
        terminal.draw(|f| stm.draw(f, ctx))?;

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
                    if let Ok((data_receiver, stats_receiver, _)) = res {
                        data_rx = Some(data_receiver);
                        stats_rx = Some(stats_receiver);
                    }
                } else if KeyCode::Char('d') == key.code
                    && stm.current_st == States::Home
                    && data_rx.is_some()
                {
                    for ctrl_tx in &ctx.ctrl_txs {
                        ctrl_tx.send(CtrlCmd::Stop).unwrap_or(());
                    }

                    data_rx = None;
                    stats_rx = None;
                }
            }
        }

        if let Some(ref data_rx) = data_rx {
            if let Ok(data) = data_rx.recv_timeout(RECV_TIMEOUT) {
                match data {
                    Data::LegacyContactInfo(info) => {
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
                            let row = create_row(&info);
                            ctx.model.home_stateful_table.push_row(row);
                        }
                    }
                    Data::Version(version) => {
                        fn format_version(version: &LegacyVersion2) -> String {
                            format!("{}.{}.{}", version.major, version.minor, version.patch)
                        }

                        let at_index =
                            ctx.model
                                .home_stateful_table
                                .items
                                .iter()
                                .position(|row| -> bool {
                                    let id = &row[2];

                                    id == &version.from.to_string()
                                });

                        if let Some(index) = at_index {
                            let row = &mut ctx.model.home_stateful_table.items[index];

                            let _ =
                                std::mem::replace(&mut row[3], format_version(&version.version));
                        }
                    }
                }
            }
        }

        if let Some(ref stats_rx) = stats_rx {
            if let Ok(stats) = stats_rx.recv_timeout(RECV_TIMEOUT) {
                fn format_stats(ctx: &mut Context, index: usize, stats: &Stats) {
                    let _ = std::mem::replace(
                        &mut ctx.model.home_stats_stateful_list.items[index],
                        format!("[{:?}] processed msgs #: {}", stats.id, stats.counter),
                    );
                }
                match stats.id {
                    transport::StatsId::Receiver => format_stats(ctx, 0, &stats),
                    transport::StatsId::Sender => format_stats(ctx, 1, &stats),
                    transport::StatsId::Logic => format_stats(ctx, 2, &stats),
                }
            }
        }

        let now = Instant::now();
        if (now - before) > STATS_INTERVAL {
            before = now;

            for ctrl_tx in &ctx.ctrl_txs {
                ctrl_tx.send(CtrlCmd::Counter).unwrap_or(());
            }
        }
    }
}

fn create_row(info: &LegacyContactInfo) -> Vec<String> {
    vec![
        format!("{}", info.gossip.ip()),         // "IP",
        format!("{}", info.wallclock),           // "Age(ms)",
        format!("{:?}", info.id),                // "Node Identifier",
        format!(" - ",),                         // "Version",
        format!("{}", info.gossip.port()),       // "Gossip",
        format!("{}", info.tpu_vote.port()),     // "TPUvote",
        format!("{}", info.tpu.port()),          // "TPU",
        format!("{}", info.tpu_forwards.port()), // "TPUfwd",
        format!("{}", info.tvu.port()),          // "TVU",
        format!("{}", info.tvu_forwards.port()), // "TVUfwd",
        format!("{}", info.repair.port()),       // "Repair",
        format!("{}", info.serve_repair.port()), // "ServeR",
        format!("{}", info.shred_version),       // "ShredVer",
    ]
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
            row.into_iter()
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
            }
            None => {}
        }
    }
}
