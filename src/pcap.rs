use std::io::Read;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PcapError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("not pcap format")]
    NotPcap,
}

type Result<T> = std::result::Result<T, PcapError>;

#[derive(Default)]
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
    let mut ans: PcapHeader = Default::default();
    read.read_exact(&mut ans.tcpdump_magic)?;
    if !is_pcap_magic(&ans.tcpdump_magic) {
        return Err(PcapError::NotPcap);
    }
    read.read_exact(&mut ans.major_version)?;
    read.read_exact(&mut ans.minor_version)?;
    read.read_exact(&mut ans.time_zone)?;
    read.read_exact(&mut ans.sigfigs)?;
    read.read_exact(&mut ans.scaplen)?;
    read.read_exact(&mut ans.link_type)?;
    Ok(ans)
}

fn is_bigendian_pcap_magic(buf: &[u8; 4]) -> bool {
    *buf == [0xd4, 0xc3, 0xb2, 0xa1]
}

fn is_littleendian_pcap_magic(buf: &[u8; 4]) -> bool {
    *buf == [0xa1, 0xb2, 0xc3, 0xd4]
}

fn is_pcap_magic(buf: &[u8; 4]) -> bool {
    is_bigendian_pcap_magic(buf) || is_littleendian_pcap_magic(buf)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_pcap_magic() {
        let a = [0xd4, 0xc3, 0xb2, 0xa1];
        assert!(is_pcap_magic(&a));
        let b = [0xd5, 0xc3, 0xb2, 0xa1];
        assert!(!is_pcap_magic(&b));
    }
}
