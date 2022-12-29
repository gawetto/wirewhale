use anyhow::{Context, Result};
//use std::io::Read;
use crate::l3data::ipv4data::Ipv4data;
use byteorder::NetworkEndian;
use byteorder::ReadBytesExt;

#[derive(Debug)]
pub struct UdpHeader {
    src_port: u16,
    dst_port: u16,
    _len: u16,
    _checksum: u16,
}

#[derive(Debug)]
pub struct Udp {
    header: UdpHeader,
    data: Vec<u8>,
}

pub fn read_udp(mut read: std::collections::VecDeque<u8>) -> Result<Udp> {
    let src_port = read
        .read_u16::<NetworkEndian>()
        .context("read src_port failed")?;
    let dst_port = read
        .read_u16::<NetworkEndian>()
        .context("read dst_port failed")?;
    let _len = read.read_u16::<NetworkEndian>()?;
    let _checksum = read.read_u16::<NetworkEndian>()?;
    let header = UdpHeader {
        src_port,
        dst_port,
        _len,
        _checksum,
    };
    let data = Vec::from(read);
    Ok(Udp { header, data })
}

impl Ipv4data for Udp {
    fn text(&self) -> Vec<String> {
        vec![
            format!("{:?}", self.header),
            format!("UdpData {:?}", self.data),
        ]
    }
    fn line(&self, src: &std::net::Ipv4Addr, dst: &std::net::Ipv4Addr) -> String {
        format!(
            "{}:{} → {}:{} UDP",
            src, self.header.src_port, dst, self.header.dst_port
        )
    }
}
