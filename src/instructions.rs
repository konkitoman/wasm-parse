use std::backtrace::Backtrace;

use crate::{
    modules::{DataIdx, ElemIdx, FuncIdx, GlobalIdx, LabelIdx, LocalIdx, TableIdx, TypeIdx},
    types::ValType,
    Buffer, Error, Parse, IB,
};

#[derive(Debug)]
pub enum BlockType {
    Empty,
    ValType(ValType),
    X(i64),
}

#[derive(Debug)]
pub enum Instr {
    UnReachable,
    Nop,
    Block(BlockType, Vec<Instr>),
    Loop(BlockType, Vec<Instr>),
    If(BlockType, Vec<Instr>),
    IfElse(BlockType, Vec<Instr>, Vec<Instr>),
    Br(LabelIdx),
    BrIf(LabelIdx),
    BrTable(Vec<LabelIdx>, LabelIdx),
    Return,
    Call(FuncIdx),
    CallIndirect(TypeIdx, TableIdx),
    RefNull(u32),
    RefIsNull,
    RefFunc(u32),
    Drop,
    Select,
    SelectType(Vec<ValType>),
    LocalGet(LocalIdx),
    LocalSet(LocalIdx),
    LocalTee(LocalIdx),
    GlobalGet(GlobalIdx),
    GlobalSet(GlobalIdx),
    TableGet(TableIdx),
    TableSet(TableIdx),
    TableInit(ElemIdx, TableIdx),
    ElemDrop(ElemIdx),
    TableCopy(TableIdx, TableIdx),
    TableGrow(TableIdx),
    TableSize(TableIdx),
    TableFill(TableIdx),
    I32Load(MemArg),
    I64Load(MemArg),
    F32Load(MemArg),
    F64Load(MemArg),
    I32load8S(MemArg),
    I32Load8_u(MemArg),
    I32Load16_s(MemArg),
    I32Load16_u(MemArg),
    I64Load8_s(MemArg),
    I64Load8_u(MemArg),
    I64Load16_s(MemArg),
    I64Load16_u(MemArg),
    I64Load32_s(MemArg),
    I64Load32_u(MemArg),
    I32Store(MemArg),
    I64Store(MemArg),
    F32Store(MemArg),
    F64Store(MemArg),
    I32Store8(MemArg),
    I32Store16(MemArg),
    I64Store8(MemArg),
    I64Store16(MemArg),
    I64Store32(MemArg),
    MemorySize,
    MemoryGrow,
    MemoryInit(DataIdx),
    DataDrop(DataIdx),
    MemoryCopy,
    MemoryFill,

    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),

    I32Eqz,
    I32Eq,
    I32Ne,
    I32Lts,
    I32Ltu,
    I32Gts,
    I32Gtu,
    I32Les,
    I32Leu,
    I32Ges,
    I32Geu,

    I64Eqz,
    I64Eq,
    I64Ne,
    I64Lts,
    I64Ltu,
    I64Gts,
    I64Gtu,
    I64Les,
    I64Leu,
    I64Ges,
    I64Geu,

    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,

    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,

    I32Clz,
    I32Ctz,
    I32PopcCnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32Divs,
    I32Divu,
    I32RemS,
    I32Remu,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32Shrs,
    I32Sgru,
    I32Rotl,
    I32Rotr,

    I64Clz,
    I64Ctz,
    I64PopcCnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64Divs,
    I64Divu,
    I64RemS,
    I64Remu,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64Shrs,
    I64Sgru,
    I64Rotl,
    I64Rotr,

    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32CopySig,

    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64CopySig,

    I32WrapI64,
    I32TruncF32S,
    I32TruncF32U,
    I32TruncF64S,
    I32TruncF64U,
    I64ExtendI32S,
    I64ExtendI32U,
    I64TruncF32S,
    I64TruncF32U,
    I64TruncF64S,
    I64TruncF64U,
    F32ConvertI32S,
    F32ConvertI32U,
    F32ConvertI64S,
    F32ConvertI64U,
    F32DenoteF64,
    F64ConvertI32S,
    F64ConvertI32U,
    F64ConvertI64S,
    F64ConvertI64U,
    F64PromoteF32,
    I32ReinterpetF32,
    I64ReinterpetF64,
    F32ReinterpetI32,
    F64RetineroetI64,

    I32Extend8S,
    I32Extend16S,
    I64Extend8S,
    I64Extend16S,
    I64Extend32S,

    I32TruncSatF32S,
    I32TruncSatF32U,
    I32TruncSatF64S,
    I32TruncSatF64U,
    I64TruncSatF32S,
    I64TruncSatF32U,
    I64TructSatF64S,
    I64TructSatF64U,

    V128_Load(MemArg),
    V128_Load_8x8_S(MemArg),
    V128_Load_8x8_U(MemArg),
    V128_Load_16x4_S(MemArg),
    V128_Load_16x4_U(MemArg),
    V128_Load_32x2_S(MemArg),
    V128_Load_32x2_U(MemArg),
    V128_Load_8_Splat(MemArg),
    V128_Load_16_Splat(MemArg),
    V128_Load_32_Splat(MemArg),
    V128_Load_64_Splat(MemArg),
    V128_Load_32_Zero(MemArg),
    V128_Load_64_Zero(MemArg),
    V128_Store(MemArg),
    V128_Load_8_Lane(MemArg, LaneIdx),
    V128_Load_16_Lane(MemArg, LaneIdx),
    V128_Load_32_Lane(MemArg, LaneIdx),
    V128_Load_64_Lane(MemArg, LaneIdx),
    V128_Store_8_Lane(MemArg, LaneIdx),
    V128_Store_16_Lane(MemArg, LaneIdx),
    V128_Store_32_Lane(MemArg, LaneIdx),
    V128_Store_64_Lane(MemArg, LaneIdx),

    V128_Const([u8; 16]),

    I8X16_Shuffle([LaneIdx; 16]),

    I8X16_Extract_Lane_S(LaneIdx),
    I8X16_Extract_Lane_U(LaneIdx),
    I8X16_Replace_Lane(LaneIdx),
    I16X8_Extract_Lane_S(LaneIdx),
    I16X8_Extract_Lane_U(LaneIdx),
    I16X8_Replace_Lane(LaneIdx),
    I32X4_Extract_Lane(LaneIdx),
    I32X4_Replace_Lane(LaneIdx),
    I64X2_Extract_Lane(LaneIdx),
    I64X2_Replace_Lane(LaneIdx),
    F32X4_Extract_Lane(LaneIdx),
    F32X4_Replace_Lane(LaneIdx),
    F64X2_Extract_Lane(LaneIdx),
    F64X2_Replace_Lane(LaneIdx),

    I8x16_Swizzle,
    I8X16_Splat,
    I16X8_Splat,
    I32X4_Splat,
    I64X2_Splat,
    F32X4_Splat,
    F64X2_Splat,

    I8X16_Eq, // TODO Continue: All other vector instructions are plain opcodes without any immediates.
}

pub type LaneIdx = u8;

#[derive(Debug)]
pub struct MemArg(pub u32, pub u32);

impl Parse<&mut IB> for MemArg {
    fn parse(value: &mut IB) -> Result<Self, Error> {
        let a = value.read_uleb128(32) as u32;
        let b = value.read_uleb128(32) as u32;
        Ok(Self(a, b))
    }
}

impl Parse<&mut IB> for BlockType {
    fn parse(value: &mut IB) -> Result<Self, Error> {
        if value.is_empty() {
            return Err(Error::EndOfBuffer(Backtrace::capture()));
        }

        if *value.first().unwrap() == 0x40 {
            value.drain(..1).next().unwrap();
            return Ok(Self::Empty);
        }

        if let Ok(valtype) = ValType::parse(&mut *value) {
            return Ok(Self::ValType(valtype));
        }

        if value.len() < 3 {
            return Err(Error::EndOfBuffer(Backtrace::capture()));
        }

        Ok(Self::X(value.read_sleb128(33)))
    }
}

impl Parse<&mut IB> for Instr {
    fn parse(value: &mut IB) -> Result<Self, Error> {
        if value.is_empty() {
            return Err(Error::EndOfBuffer(Backtrace::capture()));
        }

        let byte = value.drain(0..1).next().unwrap();

        Ok(match byte {
            0x00 => Self::UnReachable,
            0x01 => Self::Nop,
            0x02 => {
                let block_type = BlockType::parse(&mut *value)?;
                let mut buffer = Vec::new();
                loop {
                    if *value.first().unwrap() == 0x0B {
                        value.drain(..1).next().unwrap();
                        break;
                    }
                    buffer.push(Instr::parse(&mut *value)?);
                }
                Self::Block(block_type, buffer)
            }
            0x03 => {
                let block_type = BlockType::parse(&mut *value)?;
                let mut buffer = Vec::new();
                loop {
                    if *value.first().unwrap() == 0x0B {
                        value.drain(..1).next().unwrap();
                        break;
                    }
                    buffer.push(Instr::parse(&mut *value)?);
                }
                Self::Loop(block_type, buffer)
            }
            0x04 => {
                let block_type = BlockType::parse(&mut *value)?;
                let mut buffer = Vec::new();
                loop {
                    match *value.first().unwrap() {
                        0x0B => {
                            value.drain(..1).next().unwrap();
                            return Ok(Self::If(block_type, buffer));
                        }
                        0x05 => {
                            value.drain(..1).next().unwrap();
                            let mut buffer2 = Vec::new();
                            loop {
                                if *value.first().unwrap() == 0x0B {
                                    value.drain(..1).next().unwrap();
                                    break;
                                }
                                buffer2.push(Instr::parse(&mut *value)?);
                            }
                            return Ok(Self::IfElse(block_type, buffer, buffer2));
                        }
                        _ => {}
                    }

                    buffer.push(Instr::parse(&mut *value)?);
                }
            }
            0x0C => {
                let label_idx = u32::parse(value)?;
                Self::Br(label_idx)
            }
            0x0D => {
                let label_idx = u32::parse(value)?;
                Self::BrIf(label_idx)
            }
            0x0E => {
                let l: Vec<u32> = Vec::<u32>::parse(&mut *value)?;
                let l1 = u32::parse(&mut *value)?;
                Self::BrTable(l, l1)
            }
            0x0F => Self::Return,
            0x10 => {
                let u = u32::parse(value)?;
                Self::Call(u)
            }
            0x11 => {
                let u = u32::parse(&mut *value)?;
                let t = u32::parse(&mut *value)?;
                Self::CallIndirect(u, t)
            }
            0xD0 => {
                let u = u32::parse(&mut *value)?;
                Self::RefNull(u)
            }
            0xD1 => Self::RefIsNull,
            0x0D2 => {
                let u = u32::parse(value)?;
                Self::RefFunc(u)
            }
            0x1A => Self::Drop,
            0x1B => Self::Select,
            0x1C => {
                let t = Vec::parse(value)?;
                Self::SelectType(t)
            }
            0x20 => {
                let u = u32::parse(value)?;
                Self::LocalGet(u)
            }
            0x21 => {
                let u = u32::parse(value)?;
                Self::LocalSet(u)
            }
            0x22 => {
                let u = u32::parse(value)?;
                Self::LocalTee(u)
            }
            0x23 => {
                let u = u32::parse(value)?;
                Self::GlobalGet(u)
            }
            0x24 => {
                let u = u32::parse(value)?;
                Self::GlobalSet(u)
            }
            0x25 => {
                let u = u32::parse(value)?;
                Self::TableGet(u)
            }
            0x26 => {
                let u = u32::parse(value)?;
                Self::TableSet(u)
            }
            0xFC => {
                let u = u32::parse(value)?;
                match u {
                    0 => Self::I32TruncSatF32S,
                    1 => Self::I32TruncSatF32U,
                    2 => Self::I32TruncSatF64S,
                    3 => Self::I32TruncSatF64U,
                    4 => Self::I64TruncSatF32S,
                    5 => Self::I64TruncSatF32U,
                    6 => Self::I64TructSatF64S,
                    7 => Self::I64TructSatF64U,
                    8 => {
                        let a = u32::parse(value)?;
                        let byte = value.drain(0..1).next().unwrap();
                        if byte == 0x00 {
                            Self::MemoryInit(a)
                        } else {
                            unimplemented!()
                        }
                    }
                    9 => {
                        let a = u32::parse(value)?;
                        Self::DataDrop(a)
                    }
                    10 => {
                        let mut drain = value.drain(0..2);
                        let byte1 = drain.next().unwrap();
                        let byte2 = drain.next().unwrap();
                        if byte1 == 0 && byte2 == 0 {
                            Self::MemoryCopy
                        } else {
                            unimplemented!()
                        }
                    }
                    11 => {
                        let byte = value.drain(0..1).next().unwrap();
                        if byte == 0 {
                            Self::MemoryFill
                        } else {
                            unimplemented!()
                        }
                    }
                    12 => {
                        let a = u32::parse(value)?;
                        let b = u32::parse(value)?;
                        Self::TableInit(a, b)
                    }
                    13 => {
                        let a = u32::parse(value)?;
                        Self::ElemDrop(a)
                    }
                    14 => {
                        let a = u32::parse(value)?;
                        let b = u32::parse(value)?;
                        Self::TableCopy(a, b)
                    }
                    15 => {
                        let a = u32::parse(value)?;
                        Self::TableGrow(a)
                    }
                    16 => {
                        let a = u32::parse(value)?;
                        Self::TableSize(a)
                    }
                    17 => {
                        let a = u32::parse(value)?;
                        Self::TableFill(a)
                    }
                    _ => {
                        unimplemented!()
                    }
                }
            }

            // Quick
            0x28..=0x3E => {
                let a = MemArg::parse(value)?;
                Self::I32Load(a)
                // TODO Unimlemented
            }

            0x3F | 0x40 => {
                let byte = value.drain(0..1).next().unwrap();
                if byte == 0x00 {
                    Self::MemorySize
                    // TODO Unimplemented
                } else {
                    Self::parse(value)?
                }
            }

            0x41 => Self::I32Const(i32::parse(value)?),
            0x42 => Self::I64Const(i64::parse(value)?),
            0x43 => Self::F32Const(f32::parse(value)?),
            0x44 => Self::F64Const(f64::parse(value)?),

            0x45..=0xC4 => {
                // TODO
                Self::I32Eqz
            }

            0xFD => {
                let byte = u32::parse(value)?;
                match byte {
                    0..=11 | 92 | 93 => {
                        // TODO
                        let a = MemArg::parse(value)?;
                        Self::V128_Load(a)
                    }
                    12 => {
                        let mut drain = value.drain(0..16);
                        Self::V128_Const([
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                        ])
                    }
                    13 => {
                        let mut drain = value.drain(0..16);
                        Self::I8X16_Shuffle([
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                            drain.next().unwrap(),
                        ])
                    }
                    14..=20 => {
                        // TODO
                        Self::I8x16_Swizzle
                    }
                    21..=34 => {
                        // TODO
                        let byte = value.drain(..1).next().unwrap();
                        Self::I8X16_Extract_Lane_S(byte)
                    }
                    35..=255 => {
                        // TODO
                        Self::I8X16_Eq
                    }
                    84..=91 => {
                        let a = MemArg::parse(value)?;
                        let b = value.drain(..1).next().unwrap();
                        Self::V128_Load_8_Lane(a, b)
                    }
                    _ => {
                        unimplemented!("{byte}")
                    }
                }
            }

            _ => {
                panic!("Unimplemented instruction: {byte}")
            }
        })
    }
}
