use std::{
    backtrace::Backtrace,
    fmt::{Debug, Display},
};

pub mod instructions;
pub mod modules;
pub mod types;

pub(crate) type IB = std::vec::Vec<u8>;

// #[derive(Debug)]
pub enum Error {
    InvalidNumType(Backtrace, u8),
    InvalidVecType(Backtrace, u8),
    InvalidRefType(Backtrace, u8),
    InvalidValType(Backtrace, u8),
    InvalidLimits(Backtrace, u8),
    InvalidFuncType(Backtrace, u8),
    InvalidGlobalType(Backtrace, u8),
    EndOfBuffer(Backtrace),
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidNumType(b, _)
            | Error::InvalidVecType(b, _)
            | Error::InvalidRefType(b, _)
            | Error::InvalidValType(b, _)
            | Error::InvalidLimits(b, _)
            | Error::InvalidFuncType(b, _)
            | Error::InvalidGlobalType(b, _)
            | Error::EndOfBuffer(b) => Display::fmt(b, f),
        }
    }
}

pub trait Parse<T> {
    fn parse(data: T) -> Result<Self, Error>
    where
        Self: Sized;
}

impl Parse<&mut IB> for i32 {
    fn parse(data: &mut IB) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Ok(data.read_sleb128(32) as i32)
    }
}

impl Parse<&mut IB> for u32 {
    fn parse(data: &mut IB) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Ok(data.read_uleb128(32) as u32)
    }
}
impl Parse<&mut IB> for i64 {
    fn parse(data: &mut IB) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Ok(data.read_sleb128(64))
    }
}
impl Parse<&mut IB> for u64 {
    fn parse(data: &mut IB) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Ok(data.read_uleb128(64))
    }
}

impl Parse<&mut IB> for f32 {
    fn parse(data: &mut IB) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if data.len() < 4 {
            return Err(Error::EndOfBuffer(Backtrace::capture()));
        }
        let mut drain = data.drain(0..4);
        Ok(Self::from_le_bytes([
            drain.next().unwrap(),
            drain.next().unwrap(),
            drain.next().unwrap(),
            drain.next().unwrap(),
        ]))
    }
}
impl Parse<&mut IB> for f64 {
    fn parse(data: &mut IB) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if data.len() < 8 {
            return Err(Error::EndOfBuffer(Backtrace::capture()));
        }
        let mut drain = data.drain(0..8);
        Ok(Self::from_le_bytes([
            drain.next().unwrap(),
            drain.next().unwrap(),
            drain.next().unwrap(),
            drain.next().unwrap(),
            drain.next().unwrap(),
            drain.next().unwrap(),
            drain.next().unwrap(),
            drain.next().unwrap(),
        ]))
    }
}

impl<T: Parse<u8>> Parse<&mut IB> for T {
    fn parse(data: &mut IB) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if data.is_empty() {
            return Err(Error::EndOfBuffer(Backtrace::capture()));
        }

        let byte = data.drain(..1).next().unwrap();
        T::parse(byte)
    }
}

impl Parse<&mut IB> for String {
    fn parse(data: &mut IB) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut len = data.read_uleb128(32);
        println!("{len}");

        let buffer = data.drain(..len as usize).collect();

        Ok(String::from_utf8(buffer).unwrap())
    }
}

impl Parse<&mut IB> for Vec<u8> {
    fn parse(data: &mut IB) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let len = data.read_uleb128(32);
        Ok(data.drain(..len as usize).collect())
    }
}

impl<T1: for<'a> Parse<&'a mut IB>, T2: for<'a> Parse<&'a mut IB>> Parse<&mut IB> for (T1, T2) {
    fn parse(data: &mut IB) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let t1 = T1::parse(data)?;
        let t2 = T2::parse(data)?;

        Ok((t1, t2))
    }
}

pub trait Buffer {
    fn read_uleb128(&mut self, n: u8) -> u64;
    fn write_uleb128(&mut self, value: u64);
    fn read_sleb128(&mut self, n: u8) -> i64;
    fn write_sleb128(&mut self, value: i64);
}

impl Buffer for IB {
    fn read_uleb128(&mut self, n: u8) -> u64 {
        match self.first() {
            Some(byte) if *byte < 128 && (*byte as u64) < (1 << n as u64) => {
                self.drain(..1).next().unwrap() as u64
            }
            Some(byte) if *byte >= 128 && n > 7 => {
                let byte = self.drain(..1).next().unwrap() as u64;
                (128 * self.read_uleb128(n - 7)) + (byte - 128)
            }
            _ => 0,
        }
    }

    fn write_uleb128(&mut self, mut value: u64) {
        loop {
            let mut byte = value & !(1 << 7);
            value >>= 7;
            if value != 0 {
                byte |= 1 << 7
            }

            self.push(byte as u8);
            if value == 0 {
                return;
            }
        }
    }

    fn read_sleb128(&mut self, n: u8) -> i64 {
        let byte = self.drain(..1).next();
        match byte {
            Some(byte) if byte < 64 && (byte as i64) < (1 << (n - 1) as i64) => byte as i64,
            Some(byte)
                if (64..128).contains(&byte)
                    && (byte as i64) >= (128i64 - (2 ^ (n - 1) as i64)) =>
            {
                let byte = byte as i64;
                byte - 128
            }
            Some(byte) if byte >= 128 && n > 7 => {
                let byte = byte as i64;
                (128 * self.read_sleb128(n - 7)) + (byte - 128)
            }
            _ => 0,
        }
    }

    fn write_sleb128(&mut self, mut value: i64) {
        loop {
            let mut byte = value as u8;
            value >>= 6;
            let done = value == 0 || value == -1;
            if done {
                byte &= !128;
            } else {
                value >>= 1;
                byte |= 128;
            }
            self.push(byte);

            if done {
                return;
            }
        }
    }
}

#[cfg(test)]
mod leb128 {
    use crate::Buffer;

    #[test]
    fn u32() {
        let mut buffer = Vec::<u8>::new();
        buffer.write_uleb128(2121);
        assert_eq!(&buffer, &[201, 16]);

        assert_eq!(buffer.read_uleb128(32), 2121);

        buffer.write_uleb128(u32::MAX as u64);
        assert_eq!(buffer.read_uleb128(32), u32::MAX as u64);
    }

    #[test]
    fn i32() {
        let mut buffer = Vec::<u8>::new();
        buffer.write_sleb128(-2121);
        assert_eq!(&buffer, &[183, 111]);

        assert_eq!(buffer.read_sleb128(32), -2121);

        buffer.write_sleb128(i32::MAX as i64);
        assert_eq!(buffer.read_sleb128(32), i32::MAX as i64);
        buffer.write_sleb128(i32::MIN as i64);
        assert_eq!(buffer.read_sleb128(32), i32::MIN as i64);
    }

    #[test]
    fn u64() {
        let mut buffer = Vec::<u8>::new();
        buffer.write_uleb128(2121);
        assert_eq!(&buffer, &[201, 16]);

        assert_eq!(buffer.read_uleb128(64), 2121);

        buffer.write_uleb128(u64::MAX);
        assert_eq!(buffer.read_uleb128(64), u64::MAX);
    }

    #[test]
    fn i64() {
        let mut buffer = Vec::<u8>::new();
        buffer.write_sleb128(-2121);
        assert_eq!(&buffer, &[183, 111]);

        assert_eq!(buffer.read_sleb128(64), -2121);

        buffer.write_sleb128(i64::MAX);
        assert_eq!(buffer.read_sleb128(64), i64::MAX);
        buffer.write_sleb128(i64::MIN);
        assert_eq!(buffer.read_sleb128(64), i64::MIN);
    }
}
