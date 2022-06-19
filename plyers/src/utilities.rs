use std::io::Read;

use super::Error;

pub fn read_byte<R>(file: &mut R, offset: &mut usize) -> Result<u8, Error>
where
    R: Read,
{
    let mut byte_buf = [0u8; 1];
    let n = file.read(&mut byte_buf)?;
    if n == 0 {
        return Err(Error::UnexpectedEndOfFile(*offset));
    }
    *offset = *offset + n;

    Ok(byte_buf[0])
}
