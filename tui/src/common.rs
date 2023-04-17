use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::{mpsc, Arc},
    thread::JoinHandle,
    time::Duration,
};

use log::trace;

use solana_gossip_proto::{
    protocol::{LegacyContactInfo, Version},
    utils::parse_addr,
    wire::Payload,
};

use crate::app::Context;
use crate::logic::spawn_logic;
use crate::transport::{receiver::spawn_receiver, sender::spawn_sender, CtrlCmd, Stats};

#[derive(Debug)]
pub enum Data {
    LegacyContactInfo(Box<LegacyContactInfo>),
    Version(Version),
}

#[allow(clippy::type_complexity)]
pub fn init_threads(
    ctx: &mut Context,
) -> io::Result<(
    mpsc::Receiver<Data>,
    mpsc::Receiver<Stats>,
    Vec<JoinHandle<()>>,
)> {
    let entrypoint_str = if let Some(entrypoint) = &ctx.model.entrypoint {
        entrypoint.as_str()
    } else {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "entrypoint address not selected",
        ));
    };

    let Some(entrypoint_addr) = parse_addr(entrypoint_str) else {
          return Err(io::Error::new(
            io::ErrorKind::Other,
            "invalid entrypoint address",
          ));
        };

    let gossip_local_ip_addr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
    let gossip_local_listener_addr = SocketAddr::new(gossip_local_ip_addr, ctx.model.listern_port);

    let socket = UdpSocket::bind(gossip_local_listener_addr)?;
    socket.set_read_timeout(Some(Duration::from_millis(1000)))?;

    let socket = Arc::new(socket);
    trace!("[main] gossip_addr:{gossip_local_listener_addr:?}");

    // receiver
    let (ctrl_sender_tx, ctrl_sender_rx) = mpsc::channel::<CtrlCmd>();
    ctx.ctrl_txs.push(ctrl_sender_tx);
    let (sender_tx, sender_rx) = mpsc::channel::<Payload>();

    // sender
    let (ctrl_receiver_tx, ctrl_receiver_rx) = mpsc::channel::<CtrlCmd>();
    ctx.ctrl_txs.push(ctrl_receiver_tx);
    let (receiver_tx, receiver_rx) = mpsc::channel::<Payload>();

    // logic
    let (ctrl_logic_tx, ctrl_logic_rx) = mpsc::channel::<CtrlCmd>();
    ctx.ctrl_txs.push(ctrl_logic_tx);

    // stats
    let (stats_tx, stats_rx) = mpsc::channel::<Stats>();

    let (data_tx, data_rx) = mpsc::channel::<Data>();

    let receiver_t = spawn_receiver(
        socket.clone(),
        receiver_tx,
        ctrl_receiver_rx,
        stats_tx.clone(),
    )?;
    let sender_t = spawn_sender(socket, sender_rx, ctrl_sender_rx, stats_tx.clone())?;
    let logic_t = spawn_logic(
        gossip_local_listener_addr,
        entrypoint_addr,
        sender_tx,
        receiver_rx,
        ctrl_logic_rx,
        stats_tx,
        data_tx,
    )?;

    Ok((data_rx, stats_rx, vec![receiver_t, sender_t, logic_t]))
}
