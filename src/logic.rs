use std::{
  io,
  net::SocketAddr,
  sync::{
    mpsc::{Receiver, Sender},
    Arc,
  },
  thread::{Builder, JoinHandle},
  time::Duration,
};

use crate::{
  common::Data,
  protocol::{CrdsData, CrdsFilter, CrdsValue, LegacyContactInfo, Pong, Protocol},
  transport::{CtrlCmd, Payload},
  utils::since_the_epoch_millis,
};

use solana_sdk::{signature::Keypair, signer::Signer};

pub const RECV_TIMEOUT: Duration = Duration::from_millis(30);

pub(crate) fn spawn_logic(
  gossip_local_listener_addr: SocketAddr,
  entrypoint_addr: SocketAddr,
  tx: Sender<Payload>,
  rx: Receiver<Payload>,
  ctrl_rx: Receiver<CtrlCmd>,
  data_tx: Sender<Data>,
  trace: bool,
) -> io::Result<JoinHandle<()>> {
  #[allow(clippy::let_and_return)]
  Builder::new().name("logic_t".to_string()).spawn(move || {
    let mut index: u32 = 0;

    let keypair = Keypair::new();
    let keypair_arc = Arc::new(keypair);
    let shred_version: u16 = 0;

    let contact_info = LegacyContactInfo {
      id: keypair_arc.pubkey(),
      gossip: gossip_local_listener_addr,
      wallclock: since_the_epoch_millis(),
      shred_version,
      ..LegacyContactInfo::default()
    };

    'main_l: loop {
      index += 1;

      match ctrl_rx.try_recv() {
        Ok(ctrl_msg) => match ctrl_msg {
          CtrlCmd::Stop => break 'main_l,
          CtrlCmd::Counter => {
            if trace {
              println!("[logic_t] index:{index} received CtrlCmd::Counter");
            }
          },
        },
        Err(_e) => {},
      }

      if let Ok(payload) = rx.recv_timeout(RECV_TIMEOUT) {
        if let Some(from_addr) = payload.addr {
          let len = payload.len;
          if trace {
            println!(
              "######## [logic_t] i:{index} #### addr:{:?} #### len:{len} ################ 1",
              from_addr
            );
          }
          let r: Result<Protocol, Box<bincode::ErrorKind>> = payload.deserialize_slice(..);
          match r {
            Ok(proto) => match proto {
              Protocol::PingMessage(ping) => {
                if trace {
                  println!("# PingMessage ping:{:?}", ping);
                }
                let pong_r = Pong::new(&ping, &keypair_arc);
                if let Ok(pong) = pong_r {
                  let proto_pong = Protocol::PongMessage(pong);

                  let mut data = Payload::default();
                  let r = data.populate_packet(Some(from_addr), &proto_pong);

                  match r {
                    Ok(_) => {
                      tx.send(data).unwrap_or(());
                    },
                    Err(err) => {
                      if trace {
                        println!("[logic_t] index:{index} err:{:?}", err);
                      }
                    },
                  }
                }
              },
              Protocol::PongMessage(pong) => {
                if trace {
                  println!(
                    "# len:{len} PongMessage from_addr:{:?} pong:{:?}",
                    from_addr, pong
                  );
                }
              },
              Protocol::PullResponse(from_key, crds_values) => {
                if trace {
                  println!(
                    "# len:{len} PullResponse from_addr:{:?} from_key:{:?}",
                    from_addr, from_key
                  );
                }
                for value in &crds_values {
                  if trace {
                    println!("# {:?}", crds_data_print(value));
                  }

                  match &value.data {
                    CrdsData::LegacyContactInfo(info) => {
                      data_tx
                        .send(Data::LegacyContactInfo(info.clone()))
                        .unwrap_or(());
                    },
                    CrdsData::Version(version) => {
                      data_tx.send(Data::Version(version.clone())).unwrap_or(());
                    },
                    _ => {},
                  }
                }
              },
              _ => {
                if trace {
                  println!("# ??? err protocol:{:?}", proto);
                }
              },
            },
            Err(err) => {
              if trace {
                println!("# ??? err:{:?}", err);
              }
            },
          }
          if trace {
            println!("#----- [logic_t] ---------------------------------- 2");
          }
        }
      }

      let crds_data = CrdsData::LegacyContactInfo(contact_info.clone());
      let crds_value = CrdsValue::new_signed(crds_data.clone(), &keypair_arc);
      let crds_filter = CrdsFilter::default();

      let protocol = Protocol::PullRequest(crds_filter, crds_value);

      if trace && index % 50 == 0 {
        println!("#===== [logic_t] ==================================");
        println!("# index:{index} LegacyContactInfo has been sended");
        println!("#----- [logic_t] ----------------------------------");
      }

      let mut data = Payload::default();

      let r = data.populate_packet(Some(entrypoint_addr), &protocol);

      match r {
        Ok(_) => {
          tx.send(data).unwrap_or(());
        },
        Err(err) => {
          if trace {
            println!("[logic_t] index:{index} err:{:?}", err);
          }
        },
      }
    }

    if trace {
      println!("[logic_t] index:{index} terminated");
    }
  })
}

fn crds_data_print(value: &CrdsValue) -> String {
  match value.data.clone() {
    CrdsData::LegacyContactInfo(info) => {
      format!("LegacyContactInfo info:{:?}", info)
    },
    CrdsData::Vote(_, _) => {
      "Vote".to_string()
    },
    CrdsData::LowestSlot(_, _) => {
      "LowestSlot".to_string()
    },
    CrdsData::SnapshotHashes(snapshot) => {
      format!("SnapshotHashes snapshot:{:?}", snapshot)
    },
    CrdsData::AccountsHashes(snapshot) => {
      format!("AccountsHashes snapshot:{:?}", snapshot)
    },
    CrdsData::EpochSlots(_, _) => {
      "EpochSlots".to_string()
    },
    CrdsData::LegacyVersion(version) => {
      format!("LegacyVersion version:{:?}", version)
    },
    CrdsData::Version(version) => {
      format!("Version version:{:?}", version)
    },
    CrdsData::NodeInstance(node_instance) => {
      format!("NodeInstance nodeInstance:{:?}", node_instance)
    },
    CrdsData::DuplicateShred() => {
      "DuplicateShred".to_string()
    },
    CrdsData::IncrementalSnapshotHashes(_) => {
      "IncrementalSnapshotHashes".to_string()
    },
    CrdsData::ContactInfo() => {
      "ContactInfo".to_string()
    },
  }
}
