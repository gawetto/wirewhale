pub mod ipv4;
pub mod other;

use anyhow::{Context, Result};
use ipv4::read_ipv4;
use other::read_otherl3data;
use std::fmt::Debug;
use std::io::Read;

pub trait L3data: Debug + Send + Sync {
    fn text(&self) -> Vec<String>;
    fn line(&self) -> String;
}

pub fn read_l3data(read: &mut impl Read, len: u32, _type: u16) -> Result<Box<dyn L3data>> {
    match _type {
        8 => Ok(Box::new(read_ipv4(read, len, _type).context("read ipv4")?)),
        _ => Ok(Box::new(
            read_otherl3data(read, len, _type).context("read otherl3data")?,
        )),
    }
}
