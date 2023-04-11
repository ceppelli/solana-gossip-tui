use std::{
  io,
  net::UdpSocket,
  sync::{
    mpsc::{Receiver, Sender},
    Arc,
  },
  thread::{Builder, JoinHandle},
};

use crate::transport::{CtrlCmd, Payload, Stats, StatsId, RECV_TIMEOUT};

pub(crate) fn spawn_sender(
  socket: Arc<UdpSocket>,
  rx: Receiver<Payload>,
  ctrl_rx: Receiver<CtrlCmd>,
  stats_tx: Sender<Stats>,
  trace: bool,
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
                .send(Stats { id: StatsId::Sender, counter })
                .unwrap_or(());

              if trace {
                println!("[udp_sender_t] message processed:{}", counter);
              }
            },
          }
        }

        if let Ok(data) = rx.recv_timeout(RECV_TIMEOUT) {
          if let Some(addr) = data.addr {
            if let Some(buf) = data.data(..) {
              // if trace {
              //   println!("[udp_sender_t] index:{} sending to addr:{:?}", index, addr);
              // }

              if let Err(e) = socket.send_to(buf, addr) {
                if trace {
                  println!("[udp_sender_t] index:{} sending Err:{:?}", counter, e);
                }
              }

              counter += 1;
            }
          }
        }
      }

      if trace {
        println!("[udp_sender_t]  index:{counter} terminated");
      }
    })
}
