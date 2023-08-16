use std::backtrace::Backtrace;

use crate::{Error, Parse, IB};

#[derive(Debug)]
pub enum NumType {
    I32,
    I64,
    F32,
    F64,
}

#[derive(Debug)]
pub enum VecType {
    V128,
}

#[derive(Debug)]
pub enum RefType {
    FuncRef,
    ExternRef,
}

#[derive(Debug)]
pub enum ValType {
    NumType(NumType),
    VecType(VecType),
    RefType(RefType),
}

pub type ResultType = Vec<ValType>;

#[derive(Debug)]
pub struct FuncType(ResultType, ResultType);

#[derive(Debug)]
pub struct Limits(u32, Option<u32>);

pub type MemType = Limits;

#[derive(Debug)]
pub struct TableType(RefType, Limits);

#[derive(Debug)]
pub struct GlobalType(bool, ValType);

impl Parse<u8> for NumType {
    fn parse(value: u8) -> Result<Self, Error> {
        Ok(match value {
            0x7F => Self::I32,
            0x7E => Self::I64,
            0x7D => Self::F32,
            0x7C => Self::F64,
            _ => return Err(Error::InvalidNumType(Backtrace::capture(), value)),
        })
    }
}

impl Parse<u8> for VecType {
    fn parse(value: u8) -> Result<Self, Error> {
        if value == 0x7B {
            Ok(Self::V128)
        } else {
            Err(Error::InvalidVecType(Backtrace::capture(), value))
        }
    }
}

impl Parse<u8> for RefType {
    fn parse(value: u8) -> Result<Self, Error> {
        Ok(match value {
            0x70 => Self::FuncRef,
            0x6F => Self::ExternRef,
            _ => return Err(Error::InvalidRefType(Backtrace::capture(), value)),
        })
    }
}

impl Parse<u8> for ValType {
    fn parse(value: u8) -> Result<Self, Error> {
        Ok(match value {
            0x6F..=0x70 => Self::RefType(RefType::parse(value)?),
            0x7B => Self::VecType(VecType::V128),
            0x7C..=0x7F => Self::NumType(NumType::parse(value)?),

            _ => return Err(Error::InvalidValType(Backtrace::capture(), value)),
        })
    }
}

impl<T: for<'a> Parse<&'a mut IB>> Parse<&mut IB> for Vec<T> {
    fn parse(value: &mut IB) -> Result<Self, Error> {
        let len = u32::parse(&mut *value)?;
        let mut buffer = std::vec::Vec::<T>::with_capacity(len as usize);
        for _ in 0..len {
            buffer.push(T::parse(value)?)
        }
        Ok(buffer)
    }
}

impl Parse<&mut IB> for FuncType {
    fn parse(value: &mut IB) -> Result<Self, Error> {
        if value.is_empty() {
            return Err(Error::EndOfBuffer(Backtrace::capture()));
        }

        let byte = value.drain(0..1).next().unwrap();

        if byte == 0x60 {
            Ok(Self(
                ResultType::parse(&mut *value)?,
                ResultType::parse(value)?,
            ))
        } else {
            Err(Error::InvalidFuncType(Backtrace::capture(), byte))
        }
    }
}

impl Parse<&mut IB> for Limits {
    fn parse(value: &mut IB) -> Result<Self, Error> {
        if value.is_empty() {
            return Err(Error::EndOfBuffer(Backtrace::capture()));
        }

        let byte = value.drain(0..1).next().unwrap();

        Ok(match byte {
            0 => Self(u32::parse(value)?, None),
            1 => Self(u32::parse(&mut *value)?, Some(u32::parse(value)?)),
            _ => return Err(Error::InvalidLimits(Backtrace::capture(), byte)),
        })
    }
}

impl Parse<&mut IB> for TableType {
    fn parse(value: &mut IB) -> Result<Self, Error> {
        Ok(Self(RefType::parse(&mut *value)?, Limits::parse(value)?))
    }
}

impl Parse<&mut IB> for GlobalType {
    fn parse(value: &mut IB) -> Result<Self, Error> {
        let valtype = ValType::parse(&mut *value)?;
        if value.is_empty() {
            return Err(Error::EndOfBuffer(Backtrace::capture()));
        }
        let byte = value.drain(0..1).next().unwrap();
        Ok(Self(byte > 0, valtype))
    }
}
