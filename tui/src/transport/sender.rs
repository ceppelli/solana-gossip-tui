use std::{
    io,
    net::UdpSocket,
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
    thread::{Builder, JoinHandle},
};

use log::trace;

use solana_gossip_proto::wire::Payload;

use crate::transport::{CtrlCmd, Stats, StatsId, RECV_TIMEOUT};

pub(crate) fn spawn_sender(
    socket: Arc<UdpSocket>,
    rx: Receiver<Payload>,
    ctrl_rx: Receiver<CtrlCmd>,
    stats_tx: Sender<Stats>,
) -> io::Result<JoinHandle<()>> {
    Builder::new()
        .name("udp_sender_t".to_string())
        .spawn(move || {
            let mut counter: u32 = 0;

            'main_l: loop {
                if let Ok(ctrl_msg) = ctrl_rx.try_recv() {
                    match ctrl_msg {
                        CtrlCmd::Stop => break 'main_l,
                        CtrlCmd::Counter => {
                            stats_tx
                                .send(Stats {
                                    id: StatsId::Sender,
                                    counter,
                                })
                                .unwrap_or(());

                            trace!("message processed:{counter}");
                        }
                    }
                }

                if let Ok(data) = rx.recv_timeout(RECV_TIMEOUT) {
                    if let Some(addr) = data.addr {
                        if let Some(buf) = data.data(..) {
                            if let Err(err) = socket.send_to(buf, addr) {
                                trace!("counter:{counter} sending err:{err:?}");
                            }

                            counter += 1;
                        }
                    }
                }
            }

            trace!("counter:{counter} terminated");
        })
}
