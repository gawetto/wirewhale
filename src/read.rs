use std::io::Read;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UntilReadError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("close")]
    Close,
}

type Result<T> = std::result::Result<T, UntilReadError>;

pub fn read_until_full<T: Read>(
    read: &mut T,
    buf: &mut [u8],
) -> std::result::Result<usize, UntilReadError> {
    let mut readed = 0;
    while readed < buf.len() {
        let n = read.read(&mut buf[readed..])?;
        if n == 0 {
            return Err(UntilReadError::Close);
        }
        readed += n;
    }
    return Ok(readed);
}

pub fn read_u32<T: Read>(read: &mut T) -> Result<u32> {
    let mut buf = [0u8; 4];
    read_until_full(read, &mut buf)?;
    let ans = buf
        .iter()
        .rev()
        .fold(0, |acc, &x| acc as u32 * 256 + x as u32);
    Ok(ans)
}

pub fn read_u16<T: Read>(read: &mut T) -> Result<u16> {
    let mut buf = [0u8; 2];
    read_until_full(read, &mut buf)?;
    let ans = buf
        .iter()
        .rev()
        .fold(0, |acc, &x| acc as u16 * 256 + x as u16);
    Ok(ans)
}

pub fn read_u8<T: Read>(read: &mut T) -> Result<u8> {
    let mut buf = [0u8; 1];
    read_until_full(read, &mut buf)?;
    Ok(buf[0])
}
