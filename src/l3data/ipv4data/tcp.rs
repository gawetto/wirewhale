use anyhow::{Context, Result};
use std::io::Read;

pub struct Tcp{
}

fn read_tcp(&mut read: impl Read) -> Result<Tcp>{
    match protocol {
        6 => Ok(Box::new(read_ipv4(read, len, _type).context("read ipv4")?)),
        _ => Ok(Box::new(
            read_otherl3data(read, len, _type).context("read otherl3data")?,
        )),
    }
}
