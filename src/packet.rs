use crate::l3data::{l3dataerror::L3dataError, read_l3data, L3data};
use crate::read::{read_u16, read_u32, read_until_full, UntilReadError};
use std::io::Read;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PacketError {
    #[error(transparent)]
    Io(#[from] UntilReadError),
    #[error(transparent)]
    L3(#[from] L3dataError),
}

type Result<T> = std::result::Result<T, PacketError>;

#[derive(Debug)]
pub struct TimeStamp {
    unix_time: u32,
    micro_sec: u32,
}

#[derive(Debug)]
struct PacketHeader {
    timestanp: TimeStamp,
    caplen: u32,
    len: u32,
}

#[derive(Debug)]
struct PacketBody {
    _src: [u8; 6],
    _dst: [u8; 6],
    _type_len: u16,
    _data: Box<dyn L3data>,
}

#[derive(Debug)]
pub struct Packet {
    header: PacketHeader,
    _body: PacketBody,
}

impl Packet {
    pub fn text(&self) -> Vec<String> {
        let to_hex = |x: [u8; 6]| {
            x.into_iter()
                .map(|x| format!("{:<02x}", x))
                .collect::<Vec<String>>()
                .join(":")
        };
        let src = to_hex(self._body._src);
        let dst = to_hex(self._body._dst);
        let mut ans = vec![format!("{} => {} len({})", src, dst, self.header.len)];
        ans.append(self._body._data.text().as_mut());
        ans
    }
    pub fn line(&self) -> String {
        format!("len({}) {}", self.header.len, self._body._data.line())
    }
}

fn read_packet_body<T: Read>(read: &mut T, len: u32) -> Result<PacketBody> {
    let mut src = [0u8; 6];
    let mut dst = [0u8; 6];
    read_until_full(read, &mut src)?;
    read_until_full(read, &mut dst)?;
    let type_len = read_u16(read)?;
    let data = read_l3data(read, len - 14, type_len)?;
    let ans = PacketBody {
        _src: src,
        _dst: dst,
        _type_len: type_len,
        _data: data,
    };
    return Ok(ans);
}

fn read_packet_header<T: Read>(read: &mut T) -> Result<PacketHeader> {
    let mut ans = PacketHeader {
        timestanp: TimeStamp {
            unix_time: 0,
            micro_sec: 0,
        },
        caplen: 0,
        len: 0,
    };
    ans.timestanp.unix_time = read_u32(read)?;
    ans.timestanp.micro_sec = read_u32(read)?;
    ans.caplen = read_u32(read)?;
    ans.len = read_u32(read)?;
    Ok(ans)
}

pub fn read_packet<T: Read>(read: &mut T) -> Result<Packet> {
    let header = read_packet_header(read)?;
    let body = read_packet_body(read, header.len)?;
    Ok(Packet {
        header,
        _body: body,
    })
}
