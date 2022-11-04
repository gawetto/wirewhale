use crate::l3data::{read_l3data, L3data};
use anyhow::{Context, Result};
use async_std::io::ReadExt;
use byteorder::LittleEndian;
use byteorder::ReadBytesExt;

#[derive(Debug, Default)]
pub struct TimeStamp {
    unix_time: u32,
    micro_sec: u32,
}

#[derive(Debug, Default)]
struct PacketHeader {
    timestanp: TimeStamp,
    caplen: u32,
    len: u32,
}

struct Macaddr([u8; 6]);
impl std::fmt::Debug for Macaddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{:x}:{:x}:{:x}:{:x}:{:x}:{:x}]",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

#[derive(Debug)]
struct PacketBody {
    _src: Macaddr,
    _dst: Macaddr,
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
        let mut ans = vec![format!(
            "{:?} => {:?} len({})",
            self._body._src, self._body._dst, self.header.len
        )];
        ans.append(self._body._data.text().as_mut());
        ans
    }
    pub fn line(&self) -> String {
        format!("len({}) {}", self.header.len, self._body._data.line())
    }
}

//pub async fn read_pcap_header<T: async_std::io::ReadExt + Unpin>(
//    read: &mut T,
//) -> Result<PcapHeader> {

fn read_packet_body(buf: &[u8], len: u32) -> Result<PacketBody> {
    let mut slice: &[u8] = buf;
    let mut src = [0u8; 6];
    let mut dst = [0u8; 6];
    std::io::Read::read_exact(&mut slice, &mut src).context("read src")?;
    std::io::Read::read_exact(&mut slice, &mut dst).context("read dst")?;
    let type_len = slice.read_u16::<LittleEndian>().context("read type_len")?;
    let data = read_l3data(&mut slice, len - 14, type_len).context("read data")?;
    let ans = PacketBody {
        _src: Macaddr(src),
        _dst: Macaddr(dst),
        _type_len: type_len,
        _data: data,
    };
    Ok(ans)
}

fn read_packet_header(buf: &[u8]) -> Result<PacketHeader> {
    let mut slice: &[u8] = buf;
    let mut ans = PacketHeader {
        timestanp: TimeStamp {
            unix_time: 0,
            micro_sec: 0,
        },
        caplen: 0,
        len: 0,
    };
    ans.timestanp.unix_time = slice.read_u32::<LittleEndian>()?;
    ans.timestanp.micro_sec = slice.read_u32::<LittleEndian>()?;
    ans.caplen = slice.read_u32::<LittleEndian>()?;
    ans.len = slice.read_u32::<LittleEndian>()?;
    Ok(ans)
}

pub async fn read_packet(read: &mut (impl ReadExt + Unpin)) -> Result<Packet> {
    let mut header_buf = [0u8; 16];
    read.read_exact(&mut header_buf).await?;
    let header = read_packet_header(&header_buf[..])?;
    let mut body_buf = vec![0u8; header.len.try_into().unwrap()];
    read.read_exact(&mut body_buf).await?;
    let body = read_packet_body(&body_buf[..], header.len)?;
    Ok(Packet {
        header,
        _body: body,
    })
}
