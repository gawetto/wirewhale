use crate::l3data::L3data;
use anyhow::Result;

#[derive(Debug)]
pub struct OtherL3data {
    _type: u16,
    _payload: Vec<u8>,
}

impl L3data for OtherL3data {
    fn text(&self) -> Vec<String> {
        vec![format!("L3Data {:x?}", self._payload)]
    }
    fn line(&self) -> String {
        format!("type({})", self._type)
    }
}

pub fn read_otherl3data(read: std::collections::VecDeque<u8>, _type: u16) -> Result<OtherL3data> {
    let _payload = Vec::from(read);
    Ok(OtherL3data { _type, _payload })
}
