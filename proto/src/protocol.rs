use std::{
    collections::BTreeSet,
    fmt,
    net::{Ipv4Addr, SocketAddr},
};

use bincode::serialize;
use bv::BitVec;
use serde::Serialize as SerdeSerialize;
use serde_derive::{Deserialize, Serialize};

use solana_bloom::bloom::Bloom;
use solana_sdk::{
    hash::{self, Hash},
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};

use crate::errors::Result;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct LegacyContactInfo {
    pub id: Pubkey,
    /// gossip address
    pub gossip: SocketAddr,
    /// address to connect to for replication
    pub tvu: SocketAddr,
    /// address to forward shreds to
    pub tvu_forwards: SocketAddr,
    /// address to send repair responses to
    pub repair: SocketAddr,
    /// transactions address
    pub tpu: SocketAddr,
    /// address to forward unprocessed transactions to
    pub tpu_forwards: SocketAddr,
    /// address to which to send bank state requests
    pub tpu_vote: SocketAddr,
    /// address to which to send JSON-RPC requests
    pub rpc: SocketAddr,
    /// websocket for JSON-RPC push notifications
    pub rpc_pubsub: SocketAddr,
    /// address to send repair requests to
    pub serve_repair: SocketAddr,
    /// latest wallclock picked
    pub wallclock: u64,
    /// node shred version
    pub shred_version: u16,
}

fn socketaddr_default() -> SocketAddr {
    SocketAddr::from((Ipv4Addr::from(0), 0))
}

impl Default for LegacyContactInfo {
    fn default() -> Self {
        LegacyContactInfo {
            id: Pubkey::default(),
            gossip: socketaddr_default(),
            tvu: socketaddr_default(),
            tvu_forwards: socketaddr_default(),
            repair: socketaddr_default(),
            tpu: socketaddr_default(),
            tpu_forwards: socketaddr_default(),
            tpu_vote: socketaddr_default(),
            rpc: socketaddr_default(),
            rpc_pubsub: socketaddr_default(),
            serve_repair: socketaddr_default(),
            wallclock: 0,
            shred_version: 0,
        }
    }
}

pub type VoteIndex = u8;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Vote {
    pub(crate) from: Pubkey,
    transaction: Transaction,
    pub(crate) wallclock: u64,
}

pub type Slot = u64;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct SnapshotHashes {
    pub from: Pubkey,
    pub hashes: Vec<(Slot, Hash)>,
    pub wallclock: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct LegacyVersion1 {
    major: u16,
    minor: u16,
    patch: u16,
    commit: Option<u32>, // first 4 bytes of the sha1 commit hash
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct LegacyVersion {
    pub from: Pubkey,
    pub wallclock: u64,
    pub version: LegacyVersion1,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct LegacyVersion2 {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub commit: Option<u32>,
    pub feature_set: u32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Version {
    pub from: Pubkey,
    pub wallclock: u64,
    pub version: LegacyVersion2,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct NodeInstance {
    pub from: Pubkey,
    pub wallclock: u64,
    pub timestamp: u64,
    pub token: u64,
}

pub type EpochSlotsIndex = u8;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Flate2 {
    pub first_slot: Slot,
    pub num: usize,
    pub compressed: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Uncompressed {
    pub first_slot: Slot,
    pub num: usize,
    pub slots: BitVec<u8>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum CompressedSlots {
    Flate2(Flate2),
    Uncompressed(Uncompressed),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct EpochSlots {
    pub from: Pubkey,
    pub slots: Vec<CompressedSlots>,
    pub wallclock: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
enum DeprecatedCompressionType {
    Uncompressed,
    GZip,
    BZip2,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub(crate) struct DeprecatedEpochIncompleteSlots {
    first: Slot,
    compression: DeprecatedCompressionType,
    compressed_list: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct LowestSlot {
    pub from: Pubkey,
    root: Slot,
    pub lowest: Slot,
    slots: BTreeSet<Slot>,
    stash: Vec<DeprecatedEpochIncompleteSlots>,
    pub wallclock: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct IncrementalSnapshotHashes {
    pub from: Pubkey,
    pub base: (Slot, Hash),
    pub hashes: Vec<(Slot, Hash)>,
    pub wallclock: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum CrdsData {
    LegacyContactInfo(Box<LegacyContactInfo>), // OK len:254
    Vote(VoteIndex, Vote),                     // OK len:472
    LowestSlot(u8, LowestSlot),                // OK len:185
    SnapshotHashes(SnapshotHashes),            // OK len:240
    AccountsHashes(SnapshotHashes),            // OK len:800
    EpochSlots(EpochSlotsIndex, EpochSlots),   // OK len:1049
    LegacyVersion(LegacyVersion),              // OK len:163
    Version(Version),                          // OK len:167
    NodeInstance(NodeInstance),                // OK len:168
    DuplicateShred(),                          // ??
    IncrementalSnapshotHashes(IncrementalSnapshotHashes), // OK len:360
    ContactInfo(),                             // ??
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct CrdsValue {
    pub signature: Signature,
    pub data: CrdsData,
}

impl CrdsValue {
    pub fn new_signed(data: CrdsData, keypair: &Keypair) -> Self {
        let signable_data = serialize(&data).expect("failed to serialize CrdsData");
        let signature = keypair.sign_message(&signable_data);
        Self { signature, data }
    }
}

impl fmt::Display for CrdsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.data {
            CrdsData::LegacyContactInfo(_) => write!(f, "LegacyContactInfo"),
            CrdsData::Vote(_, _) => write!(f, "Vote"),
            CrdsData::LowestSlot(_, _) => write!(f, "LowestSlot"),
            CrdsData::SnapshotHashes(_) => write!(f, "SnapshotHashes"),
            CrdsData::AccountsHashes(_) => write!(f, "AccountsHashes"),
            CrdsData::EpochSlots(_, _) => write!(f, "EpochSlots"),
            CrdsData::LegacyVersion(_) => write!(f, "LegacyVersion"),
            CrdsData::Version(_) => write!(f, "Version"),
            CrdsData::NodeInstance(_) => write!(f, "NodeInstance"),
            CrdsData::DuplicateShred() => write!(f, "DuplicateShred"),
            CrdsData::IncrementalSnapshotHashes(_) => write!(f, "IncrementalSnapshotHashes"),
            CrdsData::ContactInfo() => write!(f, "ContactInfo"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct CrdsFilter {
    pub filter: Bloom<Hash>,
    pub mask: u64,
    pub mask_bits: u32,
}

impl Default for CrdsFilter {
    fn default() -> Self {
        fn compute_mask(seed: u64, mask_bits: u32) -> u64 {
            assert!(seed <= 2u64.pow(mask_bits));
            let seed: u64 = seed.checked_shl(64 - mask_bits).unwrap_or(0x0);
            seed | (!0u64).checked_shr(mask_bits).unwrap_or(!0x0)
        }

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        fn mask_bits(num_items: f64, max_items: f64) -> u32 {
            // for small ratios this can result in a negative number, ensure it returns 0 instead
            ((num_items / max_items).log2().ceil()).max(0.0) as u32
        }

        let max_items: u32 = 1287;
        let num_items: u32 = 512;
        let false_rate: f64 = 0.1f64;
        let max_bits = 7424u32;
        let mask_bits = mask_bits(f64::from(num_items), f64::from(max_items));

        let bloom: Bloom<Hash> = Bloom::random(max_items as usize, false_rate, max_bits as usize);

        CrdsFilter {
            filter: bloom,
            mask: compute_mask(0_u64, mask_bits),
            mask_bits,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct PingGeneric<T> {
    pub from: Pubkey,
    token: T,
    signature: Signature,
}

/// Number of bytes in the randomly generated token sent with ping messages.
const GOSSIP_PING_TOKEN_SIZE: usize = 32;

pub type Ping = PingGeneric<[u8; GOSSIP_PING_TOKEN_SIZE]>;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Pong {
    from: Pubkey,
    hash: Hash, // Hash of received ping token.
    signature: Signature,
}

const PING_PONG_HASH_PREFIX: &[u8] = "SOLANA_PING_PONG".as_bytes();

impl Pong {
    pub fn new<T: SerdeSerialize>(ping: &PingGeneric<T>, keypair: &Keypair) -> Result<Self> {
        let token = serialize(&ping.token)?;
        let hash = hash::hashv(&[PING_PONG_HASH_PREFIX, &token]);
        let pong_response = Pong {
            from: keypair.pubkey(),
            hash,
            signature: keypair.sign_message(hash.as_ref()),
        };
        Ok(pong_response)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Protocol {
    PullRequest(CrdsFilter, CrdsValue),
    PullResponse(Pubkey, Vec<CrdsValue>),
    PushMessage(Pubkey, Vec<CrdsValue>),
    PruneMessage(Pubkey),
    PingMessage(Ping),
    PongMessage(Pong),
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Protocol::PullRequest(_, _) => write!(f, "PullRequest"),
            Protocol::PullResponse(_, _) => write!(f, "PullResponse"),
            Protocol::PushMessage(_, _) => write!(f, "PushMessage"),
            Protocol::PruneMessage(_) => write!(f, "PruneMessage"),
            Protocol::PingMessage(_) => write!(f, "PingMessage"),
            Protocol::PongMessage(_) => write!(f, "PongMessage"),
        }
    }
}

//tests
#[cfg(test)]
mod tests {

    use crate::{
        utils::parse_addr,
        wire::{Payload, PACKET_DATA_SIZE},
    };

    use super::*;

    #[test]
    fn test_sigh_crds_data() {
        let keypair = Keypair::new();

        let crds_data = CrdsData::LegacyContactInfo(Box::new(LegacyContactInfo::default()));
        let crds_value = CrdsValue::new_signed(crds_data.clone(), &keypair);

        let pubkey = keypair.pubkey();
        let message_bytes = serialize(&crds_data).expect("failed to serialize CrdsData");
        assert_eq!(
            crds_value.signature.verify(pubkey.as_ref(), &message_bytes),
            true
        );
    }

    #[test]
    fn test_crds_filter() {
        let crds_filter = CrdsFilter::default();
        assert_eq!(crds_filter.filter.keys.len(), 3);
        assert_eq!(crds_filter.filter.bits.len(), 6168);
    }

    fn create_payload(message: &[u8]) -> Payload {
        let mut buf = [0; PACKET_DATA_SIZE];

        for i in 0..message.len() {
            buf[i] = message[i];
        }

        Payload {
            len: message.len(),
            buf: buf,
            addr: None,
        }
    }

    #[test]
    fn test_parse_ping_message() {
        let data: [u8; 132] = [
            4, 0, 0, 0, 45, 131, 47, 219, 149, 49, 155, 106, 97, 226, 218, 35, 162, 99, 206, 62,
            17, 16, 64, 97, 253, 253, 30, 222, 252, 76, 178, 6, 138, 52, 128, 179, 38, 30, 158, 50,
            165, 43, 25, 99, 111, 86, 255, 205, 9, 26, 172, 148, 39, 156, 77, 29, 249, 24, 215,
            131, 25, 118, 137, 235, 115, 151, 92, 213, 245, 63, 206, 10, 124, 58, 104, 123, 10, 32,
            125, 1, 213, 224, 191, 85, 226, 252, 58, 47, 7, 196, 237, 134, 67, 108, 179, 237, 117,
            190, 149, 223, 197, 234, 29, 57, 254, 0, 99, 108, 107, 18, 62, 97, 139, 191, 68, 203,
            139, 145, 26, 3, 244, 197, 183, 237, 161, 215, 95, 61, 41, 40, 240, 9,
        ];

        let ping_payload = create_payload(&data);

        let protocol: Protocol = ping_payload.deserialize_slice(..).unwrap();

        assert!(matches!(protocol, Protocol::PingMessage(_)));

        if let Protocol::PingMessage(ping) = protocol {

            assert_eq!(
                ping.from.to_string(),
                "44fNPdtMtRDhRcfsNqxa5d5ZjifbM1WRjUxszxwFuY2W"
            );

            assert_eq!(
                ping.token.as_ref(),
                [
                    38, 30, 158, 50, 165, 43, 25, 99, 111, 86, 255, 205, 9, 26, 172, 148, 39, 156,
                    77, 29, 249, 24, 215, 131, 25, 118, 137, 235, 115, 151, 92, 213
                ]
            );

            assert_eq!(
                ping.signature.to_string(),
                "5uPm96J4wQtzSH6ZNmGpKzquVyn6bxxWxhPAT7dKXfgwHPHccP9r58mNDkcYY4cE2Aq5z2EDWpYRdMxcqnxGQ7Jp"
            );
        }
    }

    #[test]
    fn test_parse_pong_message() {
        let data: [u8; 132] = [
            5, 0, 0, 0, 70, 169, 196, 250, 151, 211, 95, 114, 127, 13, 26, 92, 75, 254, 147, 166,
            226, 150, 61, 171, 211, 234, 162, 18, 32, 116, 139, 166, 18, 174, 253, 4, 38, 30, 158,
            50, 165, 43, 25, 99, 111, 86, 255, 205, 9, 26, 172, 148, 39, 156, 77, 29, 249, 24, 215,
            131, 25, 118, 137, 235, 115, 151, 92, 213, 200, 133, 90, 144, 232, 103, 227, 143, 86,
            114, 147, 53, 96, 79, 89, 202, 181, 214, 161, 91, 97, 87, 191, 228, 14, 176, 112, 215,
            44, 35, 85, 214, 171, 251, 159, 61, 11, 126, 205, 109, 121, 193, 237, 15, 116, 115, 8,
            103, 169, 148, 238, 32, 53, 113, 144, 171, 85, 209, 62, 173, 103, 61, 11, 13,
        ];

        let pong_payload = create_payload(&data);

        let protocol: Protocol = pong_payload.deserialize_slice(..).unwrap();

        assert!(matches!(protocol, Protocol::PongMessage(_)));

        if let Protocol::PongMessage(pong) = protocol {

            assert_eq!(
                pong.from.to_string(),
                "5kqgfKSazLt43S4n7rXUh61gn53iphQEam6bPaC5sFSs"
            );

            assert_eq!(
                pong.hash.as_ref(),
                [
                    38, 30, 158, 50, 165, 43, 25, 99, 111, 86, 255, 205, 9, 26, 172, 148, 39, 156,
                    77, 29, 249, 24, 215, 131, 25, 118, 137, 235, 115, 151, 92, 213
                ]
            );

            assert_eq!(
                pong.signature.to_string(),
                "51XToRs3vtodBVWAEzSRBqe9GmuhHD2DLgSuNFbSw1DvwwWyoFfxHMLtgHEYYVg2wGP9pnJnyjatDUsKSGd8hj48"
            );
        }
    }

    #[test]
    fn test_parse_pull_request_legacy_contact_info_message() {
        let data: [u8; 1059] = [
            0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 126, 197, 47, 13, 227, 109, 122, 142, 229, 56, 81,
            25, 196, 11, 29, 6, 214, 122, 73, 198, 169, 82, 146, 145, 1, 97, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 24, 24, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 80,
            42, 249, 175, 45, 221, 168, 114, 243, 212, 66, 241, 218, 160, 254, 182, 128, 223, 59,
            253, 34, 71, 131, 25, 253, 76, 112, 50, 59, 142, 172, 130, 155, 85, 5, 44, 232, 125,
            46, 185, 76, 215, 230, 234, 24, 246, 98, 58, 36, 127, 28, 208, 97, 121, 248, 212, 151,
            20, 172, 33, 70, 112, 186, 2, 0, 0, 0, 0, 155, 254, 11, 8, 32, 208, 97, 52, 188, 96,
            144, 20, 246, 12, 209, 9, 12, 235, 69, 54, 59, 23, 123, 8, 166, 154, 73, 15, 28, 252,
            111, 210, 0, 0, 0, 0, 10, 20, 30, 40, 40, 35, 0, 0, 0, 0, 0, 0, 0, 0, 41, 35, 0, 0, 0,
            0, 0, 0, 0, 0, 42, 35, 0, 0, 0, 0, 0, 0, 0, 0, 43, 35, 0, 0, 0, 0, 0, 0, 0, 0, 44, 35,
            0, 0, 0, 0, 0, 0, 0, 0, 45, 35, 0, 0, 0, 0, 0, 0, 0, 0, 46, 35, 0, 0, 0, 0, 0, 0, 0, 0,
            47, 35, 0, 0, 0, 0, 0, 0, 0, 0, 48, 35, 0, 0, 0, 0, 0, 0, 0, 0, 49, 35, 227, 189, 238,
            143, 135, 1, 0, 0, 0, 0,
        ];

        let info_payload = create_payload(&data);

        let protocol: Protocol = info_payload.deserialize_slice(..).unwrap();

        assert!(matches!(protocol, Protocol::PullRequest(_, _)));

        if let Protocol::PullRequest(_, crds_value) = protocol {
            let crds_data = crds_value.data;
            assert!(matches!(crds_data, CrdsData::LegacyContactInfo(_,)));

            if let CrdsData::LegacyContactInfo(info) = crds_data {

                assert_eq!(
                    info.id.to_string(),
                    "BVvsUC7bcugkAE71bpDpDNpZuwsqY35syesvPtjShPDs"
                );

                assert_eq!(info.gossip, parse_addr("10.20.30.40:9000").unwrap());
                assert_eq!(info.tvu, parse_addr("0.0.0.0:9001").unwrap());
                assert_eq!(info.tvu_forwards, parse_addr("0.0.0.0:9002").unwrap());
                assert_eq!(info.repair, parse_addr("0.0.0.0:9003").unwrap());
                assert_eq!(info.tpu, parse_addr("0.0.0.0:9004").unwrap());
                assert_eq!(info.tpu_forwards, parse_addr("0.0.0.0:9005").unwrap());
                assert_eq!(info.tpu_vote, parse_addr("0.0.0.0:9006").unwrap());
                assert_eq!(info.rpc, parse_addr("0.0.0.0:9007").unwrap());
                assert_eq!(info.rpc_pubsub, parse_addr("0.0.0.0:9008").unwrap());
                assert_eq!(info.serve_repair, parse_addr("0.0.0.0:9009").unwrap());
                assert_eq!(info.wallclock, 1681747000803);
                assert_eq!(info.shred_version, 0);
            }
        }
    }

    #[test]
    fn test_parse_pull_response_legacy_contact_info_message() {
        let data: [u8; 254] = [
            1, 0, 0, 0, 112, 26, 219, 83, 31, 191, 215, 27, 61, 28, 154, 238, 134, 84, 53, 138,
            195, 64, 71, 69, 95, 125, 193, 73, 179, 255, 150, 187, 36, 104, 203, 159, 1, 0, 0, 0,
            0, 0, 0, 0, 172, 61, 20, 10, 235, 51, 78, 30, 32, 0, 105, 177, 9, 21, 226, 180, 29, 16,
            180, 75, 163, 224, 125, 153, 119, 46, 90, 32, 81, 151, 51, 224, 124, 232, 126, 52, 44,
            79, 157, 174, 46, 105, 221, 63, 234, 190, 12, 84, 97, 71, 141, 68, 26, 61, 82, 14, 196,
            148, 193, 107, 54, 79, 142, 15, 0, 0, 0, 0, 112, 26, 219, 83, 31, 191, 215, 27, 61, 28,
            154, 238, 134, 84, 53, 138, 195, 64, 71, 69, 95, 125, 193, 73, 179, 255, 150, 187, 36,
            104, 203, 159, 0, 0, 0, 0, 10, 20, 30, 40, 40, 35, 0, 0, 0, 0, 0, 0, 0, 0, 41, 35, 0,
            0, 0, 0, 0, 0, 0, 0, 42, 35, 0, 0, 0, 0, 0, 0, 0, 0, 43, 35, 0, 0, 0, 0, 0, 0, 0, 0,
            44, 35, 0, 0, 0, 0, 0, 0, 0, 0, 45, 35, 0, 0, 0, 0, 0, 0, 0, 0, 46, 35, 0, 0, 0, 0, 0,
            0, 0, 0, 47, 35, 0, 0, 0, 0, 0, 0, 0, 0, 48, 35, 0, 0, 0, 0, 0, 0, 0, 0, 49, 35, 128,
            43, 246, 143, 135, 1, 0, 0, 0, 0,
        ];

        let info_payload = create_payload(&data);

        let protocol: Protocol = info_payload.deserialize_slice(..).unwrap();

        assert!(matches!(protocol, Protocol::PullResponse(_, _)));

        if let Protocol::PullResponse(pubkey, crds_values) = protocol {
            assert_eq!(
                pubkey.to_string(),
                "8YcR2zEgUXYkKBtnWCSWM3Hbycu6RMqNvi9sGJmvezQE"
            );

            let crds_data = &crds_values[0].data;
            assert!(matches!(crds_data, CrdsData::LegacyContactInfo(_,)));

            if let CrdsData::LegacyContactInfo(info) = crds_data {

                assert_eq!(
                    info.id.to_string(),
                    "8YcR2zEgUXYkKBtnWCSWM3Hbycu6RMqNvi9sGJmvezQE"
                );

                assert_eq!(info.gossip, parse_addr("10.20.30.40:9000").unwrap());
                assert_eq!(info.tvu, parse_addr("0.0.0.0:9001").unwrap());
                assert_eq!(info.tvu_forwards, parse_addr("0.0.0.0:9002").unwrap());
                assert_eq!(info.repair, parse_addr("0.0.0.0:9003").unwrap());
                assert_eq!(info.tpu, parse_addr("0.0.0.0:9004").unwrap());
                assert_eq!(info.tpu_forwards, parse_addr("0.0.0.0:9005").unwrap());
                assert_eq!(info.tpu_vote, parse_addr("0.0.0.0:9006").unwrap());
                assert_eq!(info.rpc, parse_addr("0.0.0.0:9007").unwrap());
                assert_eq!(info.rpc_pubsub, parse_addr("0.0.0.0:9008").unwrap());
                assert_eq!(info.serve_repair, parse_addr("0.0.0.0:9009").unwrap());
                assert_eq!(info.wallclock, 1681747487616);
                assert_eq!(info.shred_version, 0);
            }
        }
    }
}
