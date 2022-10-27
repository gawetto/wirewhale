use crate::l3data::L3data;
use anyhow::{Context, Result};
use std::io::Read;

#[derive(Debug)]
pub struct OtherL3data {
    _type: u16,
    _payload: Vec<u8>,
}

impl L3data for OtherL3data {
    fn text(&self) -> Vec<String> {
        vec!["this is OtherL3data".to_string()]
    }

    fn line(&self) -> String {
        format!("OtherData type({})", self._type)
    }
}

pub fn read_otherl3data<T: Read>(read: &mut T, len: u32, _type: u16) -> Result<OtherL3data> {
    let mut payload = vec![0; len as usize];
    read.read_exact(payload.as_mut_slice())
        .context("read payload")?;
    Ok(OtherL3data {
        _type,
        _payload: payload,
    })
}
