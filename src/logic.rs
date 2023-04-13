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
    protocol::{CrdsData, CrdsFilter, CrdsValue, LegacyContactInfo, Ping, Pong, Protocol},
    transport::{CtrlCmd, Payload, Stats, StatsId},
    utils::since_the_epoch_millis,
};

use solana_sdk::{signature::Keypair, signer::Signer};

pub const RECV_TIMEOUT: Duration = Duration::from_millis(30);

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_logic(
    gossip_local_listener_addr: SocketAddr,
    entrypoint_addr: SocketAddr,
    tx: Sender<Payload>,
    rx: Receiver<Payload>,
    ctrl_rx: Receiver<CtrlCmd>,
    stats_tx: Sender<Stats>,
    data_tx: Sender<Data>,
    trace: bool,
) -> io::Result<JoinHandle<()>> {
    Builder::new().name("logic_t".to_string()).spawn(move || {
    let mut counter: u32 = 0;

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
      if let Ok(ctrl_msg) = ctrl_rx.try_recv() {
        match ctrl_msg {
          CtrlCmd::Stop => break 'main_l,
          CtrlCmd::Counter => {
            stats_tx
              .send(Stats { id: StatsId::Logic, counter })
              .unwrap_or(());

            if trace {
              println!("[logic_t] index:{counter} received CtrlCmd::Counter");
            }
          },
        }
      }
      if let Ok(payload) = rx.recv_timeout(RECV_TIMEOUT) {
        if let Some(from_addr) = payload.addr {
          let len = payload.len;
          if trace {
            println!(
              "######## [logic_t] i:{counter} #### addr:{from_addr:?} #### len:{len} ################ 1",
            );
          }
          let r: Result<Protocol, Box<bincode::ErrorKind>> = payload.deserialize_slice(..);
          match r {
            Ok(proto) => match proto {
              Protocol::PingMessage(ping) =>
                    send_pong_response(&ping, from_addr, &keypair_arc, &tx, trace, counter),
              Protocol::PongMessage(pong) => {
                if trace {
                  println!(
                    "# len:{len} PongMessage from_addr:{from_addr:?} pong:{pong:?}",
                  );
                }
              },
              Protocol::PullResponse(from_key, crds_values) => {
                if trace {
                  println!(
                    "# len:{len} PullResponse from_addr:{from_addr:?} from_key:{from_key:?}"
                  );
                }
                for value in &crds_values {
                  if trace {
                    println!("# {value:?}");
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
                  println!("# ??? err protocol:{proto:?}");
                }
              },
            },
            Err(err) => {
              if trace {
                println!("# ??? err:{err:?}");
              }
            },
          }
          if trace {
            println!("#----- [logic_t] ---------------------------------- 2");
          }

          counter += 1;
        }
      }

      send_pull_request(contact_info.clone(), &keypair_arc, entrypoint_addr, &tx, trace, counter);
    }

    if trace {
      println!("[logic_t] index:{counter} terminated");
    }
  })
}

fn send_pull_request(
    contact_info: LegacyContactInfo,
    keypair_arc: &Arc<Keypair>,
    entrypoint_addr: SocketAddr,
    tx: &Sender<Payload>,
    trace: bool,
    counter: u32,
) {
    let crds_data = CrdsData::LegacyContactInfo(Box::new(contact_info));
    let crds_value = CrdsValue::new_signed(crds_data, keypair_arc);
    let crds_filter = CrdsFilter::default();

    let protocol = Protocol::PullRequest(crds_filter, crds_value);
    let mut data = Payload::default();

    let r = data.populate_packet(Some(entrypoint_addr), &protocol);

    match r {
        Ok(_) => {
            tx.send(data).unwrap_or(());
        }
        Err(err) => {
            if trace {
                println!("[logic_t] index:{counter} err:{err:?}");
            }
        }
    }
}

fn send_pong_response(
    ping: &Ping,
    from_addr: SocketAddr,
    keypair_arc: &Arc<Keypair>,
    tx: &Sender<Payload>,
    trace: bool,
    counter: u32,
) {
    if trace {
        println!("# PingMessage ping:{ping:?}");
    }

    let pong_r = Pong::new(ping, keypair_arc);

    if let Ok(pong) = pong_r {
        let proto_pong = Protocol::PongMessage(pong);

        let mut data = Payload::default();
        let r = data.populate_packet(Some(from_addr), &proto_pong);

        match r {
            Ok(_) => {
                tx.send(data).unwrap_or(());
            }
            Err(err) => {
                if trace {
                    println!("# index:{counter} err:{err:?}");
                }
            }
        }
    }
}
