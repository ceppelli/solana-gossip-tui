use crate::transport::{CtrlCmd, Payload};
use std::{
  io,
  net::UdpSocket,
  sync::{mpsc::Receiver, Arc},
  thread::{Builder, JoinHandle},
};

use super::RECV_TIMEOUT;

pub(crate) fn spawn_sender(
  socket: Arc<UdpSocket>,
  rx: Receiver<Payload>,
  ctrl_rx: Receiver<CtrlCmd>,
  trace: bool,
) -> io::Result<JoinHandle<()>> {
  Builder::new()
    .name("udp_sender_t".to_string())
    .spawn(move || {
      let mut index: u32 = 0;

      'main_l: loop {
        index += 1;

        if let Ok(ctrl_msg) = ctrl_rx.try_recv() {
          match ctrl_msg {
            CtrlCmd::Stop => break 'main_l,
            CtrlCmd::Counter => {
              if trace {
                println!("[udp_sender_t] message processed:{}", index);
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
                  println!("[udp_sender_t] index:{} sending Err:{:?}", index, e);
                }
              }
            }
          }
        }
      }

      if trace {
        println!("[udp_sender_t]  index:{index} terminated");
      }
    })
}
