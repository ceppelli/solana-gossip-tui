use std::{
  io,
  net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket},
  sync::{mpsc, Arc},
  thread::JoinHandle, time::Duration,
};

use crate::logic::spawn_logic;
use crate::transport::{receiver::spawn_receiver, sender::spawn_sender, CtrlCmd, Payload};
use crate::{
  app::AppContext,
  protocol::{LegacyContactInfo, Version},
};

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Data {
  LegacyContactInfo(LegacyContactInfo),
  Version(Version),
}

pub fn init_threads(
  ctx: &mut AppContext,
) -> io::Result<(mpsc::Receiver<Data>, Vec<JoinHandle<()>>)> {
  let entrypoint_addr = parse_addr(&ctx.model.entrypoints[1]);

  if entrypoint_addr.is_none() {
    return Err(io::Error::new(
      io::ErrorKind::Other,
      "invalid entrypoint address",
    ));
  }

  let entrypoint_addr = entrypoint_addr.unwrap();

  let gossip_local_ip_addr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
  let gossip_local_listener_addr = SocketAddr::new(gossip_local_ip_addr, ctx.model.listern_port);

  let socket = UdpSocket::bind(gossip_local_listener_addr)?;
  socket.set_read_timeout(Some(Duration::from_millis(1000)))?;

  let socket = Arc::new(socket);
  if ctx.trace {
    println!("[main] gossip_addr:{:?}", gossip_local_listener_addr);
  }

  let (ctrl_sender_tx, ctrl_sender_rx) = mpsc::channel::<CtrlCmd>();
  ctx.ctrl_txs.push(ctrl_sender_tx);
  let (sender_tx, sender_rx) = mpsc::channel::<Payload>();

  let (ctrl_receiver_tx, ctrl_receiver_rx) = mpsc::channel::<CtrlCmd>();
  ctx.ctrl_txs.push(ctrl_receiver_tx);
  let (receiver_tx, receiver_rx) = mpsc::channel::<Payload>();

  let (ctrl_logic_tx, ctrl_logic_rx) = mpsc::channel::<CtrlCmd>();
  ctx.ctrl_txs.push(ctrl_logic_tx);

  let (data_tx, data_rx) = mpsc::channel::<Data>();

  let trace = ctx.trace;
  let receiver_t = spawn_receiver(socket.clone(), receiver_tx, ctrl_receiver_rx, trace)?;
  let sender_t = spawn_sender(socket, sender_rx, ctrl_sender_rx, trace)?;
  let logic_t = spawn_logic(
    gossip_local_listener_addr,
    entrypoint_addr,
    sender_tx,
    receiver_rx,
    ctrl_logic_rx,
    data_tx,
    trace,
  )?;

  Ok((data_rx, vec![receiver_t, sender_t, logic_t]))
}

pub fn parse_addr(addr: &str) -> Option<SocketAddr> {
  let addrs = addr
    .to_socket_addrs()
    .unwrap_or(Vec::new().into_iter())
    .collect::<Vec<SocketAddr>>();
  addrs.first().copied()
}

//tests
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_addr() {
    assert_eq!(
      parse_addr("entrypoint.devnet.solana.com:8001"),
      Some(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(35, 197, 53, 105)),
        8001
      ))
    );

    assert_eq!(
      parse_addr("entrypoint.testnet.solana.com:8001"),
      Some(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(35, 203, 170, 30)),
        8001
      ))
    );

    assert_eq!(
      parse_addr("entrypoint.mainnet-beta.solana.com:8001"),
      Some(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(34, 83, 231, 102)),
        8001
      ))
    );

    assert_eq!(
      parse_addr("141.98.219.218:8000"),
      Some(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(141, 98, 219, 218)),
        8000
      ))
    );
  }

  #[test]
  fn test_parse_addr_invalid() {
    assert_eq!(
      parse_addr("host,8000"),
      None
    );
  }

}
