use std::{
    net::SocketAddr,
    time::{SystemTime, UNIX_EPOCH},
};

use solana_sdk::signature::Keypair;

use crate::errors::Result;
use crate::protocol::{CrdsData, CrdsFilter, CrdsValue, LegacyContactInfo, Ping, Pong, Protocol};
use crate::wire::Payload;

#[allow(clippy::cast_possible_truncation)]
pub fn since_the_epoch_millis() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    since_the_epoch.as_millis() as u64
}

pub fn create_pull_request(
    contact_info: LegacyContactInfo,
    keypair: &Keypair,
    entrypoint_addr: SocketAddr,
) -> Result<Payload> {
    let crds_data = CrdsData::LegacyContactInfo(Box::new(contact_info));
    let crds_value = CrdsValue::new_signed(crds_data, keypair);
    let crds_filter = CrdsFilter::default();

    let protocol = Protocol::PullRequest(crds_filter, crds_value);

    let mut payload = Payload::default();
    payload.populate_packet(Some(entrypoint_addr), &protocol)?;

    Ok(payload)
}

pub fn create_pong_response(
    ping: &Ping,
    from_addr: SocketAddr,
    keypair: &Keypair,
) -> Result<Payload> {
    let value = Pong::new(ping, keypair)?;
    let protocol = Protocol::PongMessage(value);

    let mut payload = Payload::default();
    payload.populate_packet(Some(from_addr), &protocol)?;

    Ok(payload)
}
