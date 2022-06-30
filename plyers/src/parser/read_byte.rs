use std::io::{Read, Seek};
use crate::parser::error::{AddressWrapper, StreamError};

pub trait ReadByte: Read + Seek {
    fn read_byte(&mut self) -> Result<(u8, u64), AddressWrapper<StreamError>> {
        let position = self.stream_position()
            .map_err(|e| AddressWrapper { source: StreamError::PositionRead(e), address: u64::MAX })?;

        self.read_byte_untracked()
            .map(|b| (b, position))
            .map_err(|e| AddressWrapper { source: e, address: position })
    }

    fn read_byte_untracked(&mut self) -> Result<u8, StreamError> {
        let mut byte_buf = [0u8; 1];
        let n = self.read(&mut byte_buf)
            .map_err(|e| StreamError::Read(e))?;
        if n == 0 {
            return Err(StreamError::EndOfStream);
        }

        Ok(byte_buf[0])
    }
}

impl<T> ReadByte for T
where
    T: Read + Seek,
{}
