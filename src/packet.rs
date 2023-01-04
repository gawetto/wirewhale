use crate::l3data::{read_l3data, L3data};
use anyhow::{Context, Result};
use async_std::io::ReadExt;
use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use chrono::{DateTime, Local, TimeZone};
//use std::fmt::Display;
use crate::filtable::Filtable;

#[derive(Debug, Default, Clone, Copy)]
struct PacketHeader {
    _timestamp: DateTime<Local>,
    _caplen: u32,
    len: u32,
}

#[derive(Clone, Copy)]
struct Macaddr([u8; 6]);
impl std::fmt::Debug for Macaddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{:0>2x}:{:0>2x}:{:0>2x}:{:0>2x}:{:0>2x}:{:0>2x}]",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}
#[derive(Debug, Clone, Copy)]
struct FrameHeader {
    _src: Macaddr,
    _dst: Macaddr,
    _type_len: u16,
}

#[derive(Debug)]
struct PacketBody {
    _header: FrameHeader,
    _data: Box<dyn L3data>,
}

#[derive(Debug)]
pub struct Packet {
    header: PacketHeader,
    _body: PacketBody,
}

impl Packet {
    pub fn text(&self) -> Vec<String> {
        let mut ans = vec![
            format!("{:?}", self.header),
            format!("{:?}", self._body._header),
        ];
        ans.append(self._body._data.text().as_mut());
        ans
    }
    pub fn line(&self) -> String {
        format!(
            "{}{:5} {}",
            self.header._timestamp.format("%H:%M:%S"),
            self.header.len,
            self._body._data.line()
        )
    }
}

impl Filtable for Packet {
    fn is_match(&self, s: &str) -> bool {
        self.line().contains(s)
    }
}

//impl Display for Packet{
//    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//        write!(f, "{:?}\n{}", self.header, self._body)
//    }
//}
//
//impl Display for PacketBody{
//    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//        write!(f, "{:?} {:?} {}\n{:#?}", self._src, self._dst, self._type_len, self._data)
//    }
//}

fn read_packet_body(mut bytes: std::collections::VecDeque<u8>) -> Result<PacketBody> {
    let mut src = [0u8; 6];
    let mut dst = [0u8; 6];
    std::io::Read::read_exact(&mut bytes, &mut src).context("read src")?;
    std::io::Read::read_exact(&mut bytes, &mut dst).context("read dst")?;
    let type_len = bytes.read_u16::<LittleEndian>().context("read type_len")?;
    let header = FrameHeader {
        _src: Macaddr(src),
        _dst: Macaddr(dst),
        _type_len: type_len,
    };
    let data = read_l3data(bytes, type_len).context("read data")?;
    let ans = PacketBody {
        _header: header,
        _data: data,
    };
    Ok(ans)
}

fn read_packet_header(buf: &[u8]) -> Result<PacketHeader> {
    let mut slice: &[u8] = buf;
    let unix_time = slice.read_u32::<LittleEndian>()?;
    let micro_sec = slice.read_u32::<LittleEndian>()?;
    let _timestamp = Local
        .timestamp_opt(unix_time as i64, micro_sec)
        .earliest()
        .context("time parse err")?;
    let _caplen = slice.read_u32::<LittleEndian>()?;
    let len = slice.read_u32::<LittleEndian>()?;
    Ok(PacketHeader {
        _timestamp,
        _caplen,
        len,
    })
}

pub async fn read_packet(read: &mut (impl ReadExt + Unpin)) -> Result<Packet> {
    let header_buf_len = 16;
    let mut header_buf = Vec::with_capacity(header_buf_len);
    read.take(header_buf_len as u64)
        .read_to_end(&mut header_buf)
        .await?;
    let header = read_packet_header(header_buf.as_slice())?;
    let mut body_buf = Vec::with_capacity(header.len.try_into()?);
    read.take(header.len.try_into()?)
        .read_to_end(&mut body_buf)
        .await?;
    let body = read_packet_body(body_buf.into())?;
    Ok(Packet {
        header,
        _body: body,
    })
}
