use crate::transport::{CtrlCmd, Payload, PACKET_DATA_SIZE};
use std::{
  io,
  net::UdpSocket,
  sync::mpsc::{Receiver, Sender},
  sync::Arc,
  thread::{Builder, JoinHandle},
};

pub(crate) fn spawn_receiver(
  socket: Arc<UdpSocket>,
  tx: Sender<Payload>,
  ctrl_rx: Receiver<CtrlCmd>,
  trace: bool,
) -> io::Result<JoinHandle<()>> {
  Builder::new()
    .name("udp_receiver_t".to_string())
    .spawn(move || {
      let mut index: u32 = 0;

      'main_l: loop {
        index += 1;

        if let Ok(ctrl_msg) = ctrl_rx.try_recv() {
          match ctrl_msg {
            CtrlCmd::Stop => break 'main_l,
            CtrlCmd::Counter => {
              if trace {
                println!("[udp_receiver_t] message processed:{}", index);
              }
            },
          }
        }

        let mut buf = [0; PACKET_DATA_SIZE];

        match socket.recv_from(&mut buf) {
          Ok((len, addr)) => {
            // if trace {
            //   println!(
            //     "[udp_receiver_t] index:{index} received addr:{:?} len:{len} bytes {:?}",
            //     addr,
            //     &buf[..len]
            //   );
            // }

            // if trace && len == 254 {
            //   println!(
            //     "[udp_receiver_t] index:{index} received addr:{:?} len:{len}",
            //     addr
            //   );
            // }

            let include: Vec<usize> = vec![
              132, // PingMessage / PongMessage
            ];

            let exlude: Vec<usize> = vec![
              //254,  // LegacyContactInfo
              472, // Vote
              430, 442, 446, 454, 466, 478, 491, 503, 515, 724, 185, // LowestSlot
              240, // SnapshotHashes
              200, 800,  // AccountsHashes
              1049, // EpochSlots
              1022, 1026, 1028, 1032, 1038, 1039, 163, // LegacyVersion
              //167,  // Version
              168, // NodeInstance
              360, // IncrementalSnapshotHashes
              320, 280,
            ];

            if include.contains(&len) || !exlude.contains(&len) {
              tx.send(Payload { len, buf, addr: Some(addr) })
                .unwrap_or(());
            }
          },
          Err(e) => {
            if trace {
              println!("[udp_receiver_t] index:{index} recv function failed: {e:?}");
            }
          },
        }
      }

      if trace {
        println!("[udp_receiver_t] index:{index} terminated");
      }
    })
}