pub mod ipv4;
pub mod l3dataerror;
pub mod other;

use ipv4::read_ipv4;
use l3dataerror::L3dataError;
use other::read_otherl3data;
use std::fmt::Debug;
use std::io::Read;

type Result<T> = std::result::Result<T, L3dataError>;

pub trait L3data: Debug + Send + Sync {
    fn text(&self) -> Vec<String>;
    fn line(&self) -> String;
}

pub fn read_l3data<T: Read>(read: &mut T, len: u32, _type: u16) -> Result<Box<dyn L3data>> {
    match _type {
        8 => Ok(Box::new(read_ipv4(read, len, _type)?)),
        _ => Ok(Box::new(read_otherl3data(read, len, _type)?)),
    }
}
