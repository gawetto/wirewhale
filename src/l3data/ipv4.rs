use crate::l3data::{L3data, Result};
use crate::read::{read_u16, read_u8, read_until_full};
use std::io::Read;

#[derive(Debug)]
pub struct Ipv4 {
    _header: Ipv4Header,
    _payload: Vec<u8>,
}

#[derive(Debug)]
struct Ipv4Header {
    header_len: u8,
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
        vec!["this is ipv4".to_string(), "yeeeeee".to_string()]
    }
    fn line(&self) -> String {
        format!("ipv4 {} => {}", self._header._src, self._header._dst)
    }
}

fn read_ipv4_header<T: Read>(read: &mut T) -> Result<Ipv4Header> {
    let header_len = read_u8(read)? - (4 * 16) as u8;
    let service_type = read_u8(read)?;
    let packet_len = read_u8(read)? as u16 * 256 + read_u8(read)? as u16;
    let id = read_u8(read)? as u16 * 256 + read_u8(read)? as u16;
    let tmp = read_u8(read)? as u16 * 256 + read_u8(read)? as u16;
    let df = tmp & 1u16 << 14 == 1u16 << 14;
    let mf = tmp & 1u16 << 13 == 1u16 << 13;
    let fragment_offset = tmp << 3 >> 3;
    let ttl = read_u8(read)?;
    let protocol = read_u8(read)?;
    let header_checksum = read_u16(read)?;
    let src = std::net::Ipv4Addr::new(
        read_u8(read)?,
        read_u8(read)?,
        read_u8(read)?,
        read_u8(read)?,
    );
    let dst = std::net::Ipv4Addr::new(
        read_u8(read)?,
        read_u8(read)?,
        read_u8(read)?,
        read_u8(read)?,
    );
    let mut option = Vec::with_capacity(header_len as usize * 4 - 20);
    option.resize(header_len as usize * 4 - 20, 0);
    read_until_full(read, option.as_mut_slice())?;
    Ok(Ipv4Header {
        header_len,
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

pub fn read_ipv4<T: Read>(read: &mut T, len: u32, _type: u16) -> Result<Ipv4> {
    let header = read_ipv4_header(read)?;
    let payload_len = len as usize - header.header_len as usize * 4;
    let mut payload = Vec::with_capacity(payload_len);
    payload.resize(payload_len, 0);
    read_until_full(read, payload.as_mut_slice())?;
    Ok(Ipv4 {
        _header: header,
        _payload: payload,
    })
}
