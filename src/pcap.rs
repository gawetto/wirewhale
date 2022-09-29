use crate::packet::PacketError;
use crate::read::{read_until_full, UntilReadError};
use std::io::Read;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PcapError {
    #[error(transparent)]
    Io(#[from] UntilReadError),
    #[error(transparent)]
    Packet(#[from] PacketError),
    #[error("not pcap format")]
    NotPcap,
}

type Result<T> = std::result::Result<T, PcapError>;

pub struct PcapHeader {
    tcpdump_magic: [u8; 4],
    major_version: [u8; 2],
    minor_version: [u8; 2],
    time_zone: [u8; 4],
    sigfigs: [u8; 4],
    scaplen: [u8; 4],
    link_type: [u8; 4],
}

pub fn read_pcap_header<T: Read>(read: &mut T) -> Result<PcapHeader> {
    let mut ans = PcapHeader {
        tcpdump_magic: [0u8; 4],
        major_version: [0u8; 2],
        minor_version: [0u8; 2],
        time_zone: [0u8; 4],
        sigfigs: [0u8; 4],
        scaplen: [0u8; 4],
        link_type: [0u8; 4],
    };
    read_until_full(read, &mut ans.tcpdump_magic)?;
    if !is_pcap_magic(&ans.tcpdump_magic) {
        return Err(PcapError::NotPcap);
    }
    read_until_full(read, &mut ans.major_version)?;
    read_until_full(read, &mut ans.minor_version)?;
    read_until_full(read, &mut ans.time_zone)?;
    read_until_full(read, &mut ans.sigfigs)?;
    read_until_full(read, &mut ans.scaplen)?;
    read_until_full(read, &mut ans.link_type)?;
    Ok(ans)
}

fn is_pcap_magic(buf: &[u8; 4]) -> bool {
    *buf == [
        u8::from_str_radix("D4", 16).unwrap(),
        u8::from_str_radix("C3", 16).unwrap(),
        u8::from_str_radix("B2", 16).unwrap(),
        u8::from_str_radix("A1", 16).unwrap(),
    ]
}
