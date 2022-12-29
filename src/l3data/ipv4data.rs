pub mod other;
pub mod udp;

use anyhow::{Context, Result};
use other::read_other;
use std::fmt::Debug;
use udp::read_udp;

pub trait Ipv4data: Debug + Send + Sync {
    fn text(&self) -> Vec<String>;
    fn line(&self, src: &std::net::Ipv4Addr, dst: &std::net::Ipv4Addr) -> String;
}

pub fn read_ipv4data(
    read: std::collections::VecDeque<u8>,
    protocol: u8,
) -> Result<Box<dyn Ipv4data>> {
    match protocol {
        17 => Ok(Box::new(read_udp(read).context("read udp")?)),
        _type => Ok(Box::new(read_other(read, protocol)?)),
    }
}
