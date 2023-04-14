use std::time::Duration;

use tokio::time::timeout;

use solana_gossip_proto::{
    protocol::{CrdsData, LegacyContactInfo, Protocol},
    utils::{create_pong_response, create_pull_request, since_the_epoch_millis},
};
use solana_sdk::{signature::Keypair, signer::Signer};

use crate::connection::Connection;

const UDP_TIMEOUT: u64 = 200; // 200msec

pub async fn handshake(
    conn: &mut Connection,
) -> Result<Option<Box<LegacyContactInfo>>, Box<dyn std::error::Error>> {
    let keypair = Keypair::new();
    let shred_version: u16 = 0;

    // let entrypoint_addr = conn.entrypoint_addr();
    let local_addr = conn.local_addr();
    let entrypoint_addr = conn.entrypoint_addr();

    println!("[handshake] local_addr:{local_addr:?} entrypoint_addr:{entrypoint_addr:?}");

    let contact_info = LegacyContactInfo {
        id: keypair.pubkey(),
        gossip: local_addr,
        wallclock: since_the_epoch_millis(),
        shred_version,
        ..LegacyContactInfo::default()
    };

    let payload = create_pull_request(contact_info.clone(), &keypair, entrypoint_addr)?;

    conn.send(payload).await?;

    if let Some(payload) = conn.receive().await? {
        if let Ok(Protocol::PingMessage(ping)) = payload.deserialize_slice(..) {
            let pong_payload = create_pong_response(&ping, entrypoint_addr, &keypair)?;

            conn.send(pong_payload).await?;

            println!("[handshake] pong has been sended.");

            loop {

                if let Ok(Ok(Some(payload))) = timeout(Duration::from_millis(UDP_TIMEOUT), conn.receive()).await {
                    let after_pong_protocol: Result<Protocol, Box<dyn std::error::Error>> =
                        payload.deserialize_slice(..);

                    if let Ok(Protocol::PullResponse(_, values)) = after_pong_protocol {
                        for value in values {
                            println!("[handshake] message {value} has been reveived.");

                            if let CrdsData::LegacyContactInfo(info) = value.data {
                                return Ok(Some(info));
                            }
                        }
                    }
                }

                let payload = create_pull_request(contact_info.clone(), &keypair, entrypoint_addr)?;

                conn.send(payload).await?;
            }
        }
    }

    Ok(None)
}
