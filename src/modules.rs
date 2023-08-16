use core::panic;
use std::backtrace::Backtrace;

use crate::instructions::Instr;
use crate::types::{FuncType, GlobalType, Limits, MemType, RefType, TableType, ValType};
use crate::{Parse, IB};

pub type TypeIdx = u32;
pub type FuncIdx = u32;
pub type TableIdx = u32;
pub type MemIdx = u32;
pub type GlobalIdx = u32;
pub type ElemIdx = u32;
pub type DataIdx = u32;
pub type LocalIdx = u32;
pub type LabelIdx = u32;

#[derive(Debug)]
pub enum Section {
    Custom(CustomSec),
    Type(TypeSec),
    Import(ImportSec),
    Function(FuncSec),
    Table(TableSec),
    Memory(MemSec),
    Global(GlobalSec),
    Export(ExportSec),
    Start(StartSec),
    Element(ElemSec),
    Code(CodeSec),
    Data(DataSec),
    DataCountSection(DataCountSec),
    Unknown(Vec<u8>),
}

#[derive(Debug)]
pub struct CustomSec(pub String, pub Vec<u8>);
pub type TypeSec = Vec<FuncType>;
pub type ImportSec = Vec<Import>;
#[derive(Debug)]
pub struct Import {
    pub module: String,
    pub name: String,
    pub desc: ImportDesc,
}

#[derive(Debug)]
pub enum ImportDesc {
    TypeIdx(TypeIdx),
    TableType(TableType),
    MemType(MemType),
    GlobalType(GlobalType),
}

pub type FuncSec = Vec<TypeIdx>;
pub type TableSec = Vec<Table>;
pub type Table = TableType;

pub type MemSec = Vec<Mem>;
pub type Mem = MemType;

pub type GlobalSec = Vec<Global>;
pub type Global = (GlobalType, Expr);

pub type ExportSec = Vec<Export>;
pub type Export = (String, ExportDesc);
#[derive(Debug)]
pub enum ExportDesc {
    FuncIdx(FuncIdx),
    TableIdx(TableIdx),
    MemIdx(MemIdx),
    GlobalIdx(GlobalIdx),
}

pub type StartSec = Start;
pub type Start = FuncIdx;

pub type ElemSec = Vec<Elem>;
#[derive(Debug)]
pub enum Elem {
    A(Expr, Vec<FuncIdx>),
    B(u8, Vec<FuncIdx>),
    C(TableIdx, Expr, u8, Vec<FuncIdx>),
    D(u8, Vec<FuncIdx>),
    E(Expr, Vec<Expr>),
    F(RefType, Vec<Expr>),
    G(TableIdx, Expr, RefType, Vec<Expr>),
    H(RefType, Vec<Expr>),
}

pub type CodeSec = Vec<Code>;
#[derive(Debug)]
pub struct Code(u32, Func);
#[derive(Debug)]
pub struct Func(Vec<Locals>, Expr);
#[derive(Debug)]
pub struct Locals(u32, ValType);

#[derive(Debug)]
pub struct Expr(Vec<Instr>);

pub type DataSec = Vec<Data>;

#[derive(Debug)]
pub enum Data {
    A(Expr, Vec<u8>),
    B(Vec<u8>),
    C(MemIdx, Expr, Vec<u8>),
}

pub type DataCountSec = u32;

const MAGIC: u32 = 0x00_61_73_6D;
const VERSION: u32 = 0x01_00_00_00;

#[derive(Debug)]
pub struct Module {
    pub magic: u32,
    pub version: u32,
    pub sections: Vec<Section>,
}

impl Parse<&mut IB> for Expr {
    fn parse(data: &mut IB) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        let mut buffer = Vec::new();
        loop {
            if *data.first().unwrap() == 0x0B {
                println!("End");
                let _ = data.drain(..1).next().unwrap();
                break;
            }
            let i = Instr::parse(data)?;
            println!("Instr: {i:?}, until end: {}", data.len());
            buffer.push(i);
        }
        Ok(Self(buffer))
    }
}

impl Parse<&mut IB> for Import {
    fn parse(data: &mut IB) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        let module = String::parse(data)?;
        let name = String::parse(data)?;
        let desc = ImportDesc::parse(data)?;
        Ok(Self { module, name, desc })
    }
}

impl Parse<&mut IB> for ImportDesc {
    fn parse(data: &mut IB) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        if data.is_empty() {
            return Err(crate::Error::EndOfBuffer(Backtrace::capture()));
        }
        let byte = data.drain(..1).next().unwrap();
        Ok(match byte {
            0 => Self::TypeIdx(u32::parse(data)?),
            1 => Self::TableType(TableType::parse(data)?),
            2 => Self::MemType(Limits::parse(data)?),
            3 => Self::GlobalType(GlobalType::parse(data)?),
            _ => {
                unimplemented!("{byte}")
            }
        })
    }
}

impl Parse<&mut IB> for Elem {
    fn parse(data: &mut IB) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        if data.is_empty() {
            return Err(crate::Error::EndOfBuffer(Backtrace::capture()));
        }
        let byte = u32::parse(data)?;
        Ok(match byte {
            0 => Self::A(Expr::parse(data)?, Vec::parse(data)?),
            1 => {
                let byte = data.drain(..1).next().unwrap();
                Self::B(byte, Vec::parse(data)?)
            }
            2 => {
                let a = u32::parse(data)?;
                let b = Expr::parse(data)?;
                let c = data.drain(..1).next().unwrap();
                Self::C(a, b, c, Vec::parse(data)?)
            }
            3 => {
                let byte = data.drain(..1).next().unwrap();
                Self::D(byte, Vec::parse(data)?)
            }
            4 => Self::E(Expr::parse(&mut *data)?, Vec::parse(data)?),
            5 => Self::F(RefType::parse(&mut *data)?, Vec::parse(data)?),
            6 => Self::G(
                u32::parse(&mut *data)?,
                Expr::parse(&mut *data)?,
                RefType::parse(&mut *data)?,
                Vec::parse(data)?,
            ),
            7 => Self::H(RefType::parse(&mut *data)?, Vec::parse(data)?),
            _ => {
                unimplemented!("{byte}")
            }
        })
    }
}

impl Parse<&mut IB> for Data {
    fn parse(data: &mut IB) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        if data.is_empty() {
            return Err(crate::Error::EndOfBuffer(Backtrace::capture()));
        }

        let byte = u32::parse(data)?;
        Ok(match byte {
            0 => Self::A(Expr::parse(data)?, Vec::parse(data)?),
            1 => Self::B(Vec::parse(data)?),
            2 => Self::C(u32::parse(data)?, Expr::parse(data)?, Vec::parse(data)?),
            _ => unimplemented!("{byte}"),
        })
    }
}

impl Parse<&mut IB> for Locals {
    fn parse(data: &mut IB) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        let count = u32::parse(data)?;
        Ok(Self(count, ValType::parse(data)?))
    }
}
impl Parse<&mut IB> for Func {
    fn parse(data: &mut IB) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        Ok(Self(Vec::parse(&mut *data)?, Expr::parse(data)?))
    }
}
impl Parse<&mut IB> for Code {
    fn parse(data: &mut IB) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        Ok(Self(u32::parse(data)?, Func::parse(data)?))
    }
}

impl Parse<&mut IB> for Section {
    fn parse(data: &mut IB) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        let id = data.drain(..1).next().unwrap();
        println!("Id: {id}");
        let size = u32::parse(data)?;
        println!("Size: {size}");
        println!("Bytes: {}", data.len());

        Ok(match id {
            0 => {
                let current_size = data.len();
                let name = String::parse(data)?;
                let readed = current_size - data.len();
                let data = data.drain(..(size as usize) - readed).collect();
                Self::Custom(CustomSec(name, data))
            }
            1 => Self::Type(TypeSec::parse(data)?),
            2 => Self::Import(ImportSec::parse(data)?),
            3 => Self::Function(FuncSec::parse(data)?),
            4 => Self::Table(TableSec::parse(data)?),
            5 => Self::Memory(MemSec::parse(data)?),
            6 => Self::Global(GlobalSec::parse(data)?),
            7 => Self::Export(ExportSec::parse(data)?),
            8 => Self::Start(StartSec::parse(data)?),
            9 => Self::Element(ElemSec::parse(data)?),
            10 => Self::Code(CodeSec::parse(data)?),
            11 => Self::Data(DataSec::parse(data)?),
            12 => Self::DataCountSection(DataCountSec::parse(data)?),
            _ => Self::Unknown(data.drain(..size as usize).collect()),
        })
    }
}

impl Parse<&mut IB> for ExportDesc {
    fn parse(data: &mut IB) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        if data.is_empty() {
            return Err(crate::Error::EndOfBuffer(Backtrace::capture()));
        }
        let byte = data.drain(..1).next().unwrap();
        Ok(match byte {
            0 => Self::FuncIdx(u32::parse(data)?),
            1 => Self::TableIdx(u32::parse(data)?),
            2 => Self::MemIdx(u32::parse(data)?),
            3 => Self::GlobalIdx(u32::parse(data)?),
            _ => {
                unimplemented!("{byte}")
            }
        })
    }
}

impl Parse<&mut IB> for Module {
    fn parse(data: &mut IB) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        let magic;
        let version;
        {
            let mut drain = data.drain(0..8);
            magic = u32::from_le_bytes([
                drain.next().unwrap(),
                drain.next().unwrap(),
                drain.next().unwrap(),
                drain.next().unwrap(),
            ]);
            println!("Magic: {magic}");
            version = u32::from_le_bytes([
                drain.next().unwrap(),
                drain.next().unwrap(),
                drain.next().unwrap(),
                drain.next().unwrap(),
            ]);
            println!("Version: {version}");
        }

        let mut sections = Vec::new();

        loop {
            if data.is_empty() {
                break;
            }
            let section = Section::parse(data)?;
            println!("Section: {section:?}");
            sections.push(section);
        }

        Ok(Self {
            magic,
            version,
            sections,
        })
    }
}
