use std::{io, net::SocketAddr, slice::SliceIndex};

use bincode::Options;
use serde::Serialize;

use crate::errors::Result;

/// Maximum over-the-wire size of a Transaction
///   1280 is IPv6 minimum MTU
///   40 bytes is the size of the IPv6 header
///   8 bytes is the size of the fragment header
pub const PACKET_DATA_SIZE: usize = 1280 - 40 - 8;

#[derive(Debug)]
pub struct Payload {
    pub len: usize,
    pub buf: [u8; PACKET_DATA_SIZE],
    pub addr: Option<SocketAddr>,
}

impl Default for Payload {
    fn default() -> Self {
        Payload {
            len: 0,
            buf: [0; PACKET_DATA_SIZE],
            addr: None,
        }
    }
}

impl Payload {
    #[allow(clippy::cast_possible_truncation)]
    pub fn populate_packet<T: Serialize>(
        &mut self,
        dest: Option<SocketAddr>,
        data: &T,
    ) -> Result<()> {
        let mut wr = io::Cursor::new(self.buffer_mut());
        let r = bincode::serialize_into(&mut wr, data);
        match r {
            Ok(_) => {
                self.len = wr.position() as usize;
                self.addr = dest;
            }
            Err(err) => {
                return Err(err.into());
            }
        }

        Ok(())
    }

    pub fn deserialize_slice<T, I>(&self, index: I) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        I: SliceIndex<[u8], Output = [u8]>,
    {
        let bytes = self
            .data(index)
            .ok_or(bincode::Error::from(bincode::ErrorKind::SizeLimit))?;
        bincode::options()
            .with_limit(PACKET_DATA_SIZE as u64)
            .with_fixint_encoding()
            .reject_trailing_bytes()
            .deserialize(bytes)
            .map_err(std::convert::Into::into)
    }

    #[inline]
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buf[..]
    }

    #[inline]
    pub fn data<I>(&self, index: I) -> Option<&<I as SliceIndex<[u8]>>::Output>
    where
        I: SliceIndex<[u8]>,
    {
        self.buf.get(..self.len)?.get(index)
    }
}
