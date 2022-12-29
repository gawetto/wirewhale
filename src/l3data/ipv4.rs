use crate::l3data::ipv4data::Ipv4data;
use crate::l3data::{L3data, Result};
use byteorder::{NetworkEndian, ReadBytesExt};
use std::io::Read;

use super::ipv4data::read_ipv4data;

#[derive(Debug)]
pub struct Ipv4 {
    _header: Ipv4Header,
    payload: Box<dyn Ipv4data>,
}

#[derive(Debug)]
struct Ipv4Header {
    _header_len: u8,
    _service_type: u8,
    _packet_len: u16,
    _id: u16,
    _df: bool,
    _mf: bool,
    _fragment_offset: u16,
    _ttl: u8,
    _protocol: u8,
    _header_checksum: u16,
    _src: std::net::Ipv4Addr,
    _dst: std::net::Ipv4Addr,
    _option: Vec<u8>,
}

impl L3data for Ipv4 {
    fn text(&self) -> Vec<String> {
        let mut ans = vec![format!("{:?}", self._header)];
        ans.append(self.payload.text().as_mut());
        ans
    }
    fn line(&self) -> String {
        self.payload.line(&self._header._src, &self._header._dst)
    }
}

fn read_ipv4_header<T: Read>(read: &mut T) -> Result<Ipv4Header> {
    let _header_len = read.read_u8()? - (4 * 16) as u8;
    let service_type = read.read_u8()?;
    let packet_len = read.read_u16::<NetworkEndian>()?;
    let id = read.read_u16::<NetworkEndian>()?;
    let tmp = read.read_u16::<NetworkEndian>()?;
    let df = tmp & 1u16 << 14 == 1u16 << 14;
    let mf = tmp & 1u16 << 13 == 1u16 << 13;
    let fragment_offset = tmp << 3 >> 3;
    let ttl = read.read_u8()?;
    let protocol = read.read_u8()?;
    let header_checksum = read.read_u16::<NetworkEndian>()?;
    let src = std::net::Ipv4Addr::new(
        read.read_u8()?,
        read.read_u8()?,
        read.read_u8()?,
        read.read_u8()?,
    );
    let dst = std::net::Ipv4Addr::new(
        read.read_u8()?,
        read.read_u8()?,
        read.read_u8()?,
        read.read_u8()?,
    );
    let mut option = vec![0; _header_len as usize * 4 - 20];
    read.read_exact(option.as_mut_slice())?;
    Ok(Ipv4Header {
        _header_len,
        _service_type: service_type,
        _packet_len: packet_len,
        _id: id,
        _df: df,
        _mf: mf,
        _fragment_offset: fragment_offset,
        _ttl: ttl,
        _protocol: protocol,
        _header_checksum: header_checksum,
        _src: src,
        _dst: dst,
        _option: option,
    })
}

pub fn read_ipv4(mut read: std::collections::VecDeque<u8>) -> Result<Ipv4> {
    let header = read_ipv4_header(&mut read)?;
    let payload = read_ipv4data(read, header._protocol)?;
    Ok(Ipv4 {
        _header: header,
        payload,
    })
}
