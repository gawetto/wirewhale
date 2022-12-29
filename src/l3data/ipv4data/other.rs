use crate::l3data::ipv4data::Ipv4data;
use anyhow::Result;

#[derive(Debug)]
pub struct Other {
    _payload: Vec<u8>,
    _type: u8,
}

pub fn read_other(read: std::collections::VecDeque<u8>, _type: u8) -> Result<Other> {
    let _payload = Vec::from(read);
    Ok(Other { _payload, _type })
}

impl Ipv4data for Other {
    fn text(&self) -> Vec<String> {
        vec![format!("Ipv4Data {:x?}", self._payload)]
    }
    fn line(&self, src: &std::net::Ipv4Addr, dst: &std::net::Ipv4Addr) -> String {
        format!("{} -> {} protocol({})", src, dst, self._type)
    }
}
