use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UdpSocket,
};
use udpflow::UdpStreamRemote;

use solana_gossip_proto::wire::{Payload, PACKET_DATA_SIZE};

use crate::errors::Result;

pub struct Connection {
    socket: UdpStreamRemote,
}

impl Connection {
    pub async fn connect(addr: SocketAddr) -> Result<Connection> {
        let local_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
        let local_socket = UdpSocket::bind(local_addr).await?;
        let socket = UdpStreamRemote::new(local_socket, addr);
        Ok(Self { socket })
    }

    pub async fn receive(&mut self) -> Result<Option<Payload>> {
        let mut buf = [0; PACKET_DATA_SIZE];

        let len = self.socket.read(&mut buf).await?;

        if len > 0 {
            Ok(Some(Payload {
                len,
                buf,
                addr: None,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn send(&mut self, payload: Payload) -> Result<()> {
        if let Some(buf) = payload.data(..) {
            self.socket.write_all(buf).await?;
        }

        Ok(())
    }

    #[allow(unused)]
    pub fn entrypoint_addr(&self) -> SocketAddr {
        self.socket.peer_addr()
    }
    #[allow(unused)]
    pub fn local_addr(&self) -> SocketAddr {
        self.socket.local_addr()
    }
}
