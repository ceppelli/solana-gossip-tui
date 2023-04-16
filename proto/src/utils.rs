use std::{
    net::{SocketAddr, ToSocketAddrs},
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

pub fn parse_addr(addr: &str) -> Option<SocketAddr> {
    let addrs = addr
        .to_socket_addrs()
        .unwrap_or(Vec::new().into_iter())
        .collect::<Vec<SocketAddr>>();
    addrs.first().copied()
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

//tests
#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

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
        assert_eq!(parse_addr("host,8000"), None);
    }
}
