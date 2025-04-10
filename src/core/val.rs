use wasmparser::ValType;
use strum_macros::{Display, EnumString};

#[repr(u8)]
#[derive(Clone, Copy, EnumString)]
pub enum WasmType {
    Any = 0,
    U8 = 1,
    U32 = 4,
    U64 = 8,
    U128 = 16,
}

impl WasmType {
    pub fn to_string(&self) -> &str {
        match self {
            WasmType::Any => return "Any",
            WasmType::U8 => return "U8",
            WasmType::U32 => return "U32",
            WasmType::U64 => return "U64",
            WasmType::U128 => return "U128",
        }
    }
}

pub fn valtype_to_wasmtype(valtype: &ValType) -> WasmType {
    match valtype {
        ValType::I32 | ValType::F32 | ValType::Ref(_) => return WasmType::U32,
        ValType::I64 | ValType::F64 => return WasmType::U64,
        ValType::V128 => return WasmType::U128,
    }
}

pub fn u8_to_wasmtype(num: u8) -> WasmType {
    match num {
        1 => return WasmType::U32,
        2 => return WasmType::U64,
        4 => return WasmType::U128,
        _ => return WasmType::Any,
    }
}

#[derive(Clone, Copy)]
pub enum SpaceKind {
    Static,
    Dynamic,
}

#[derive(Clone, Copy)]
pub struct ValInfo {
    // ty: WasmType,
    // pos: u32,
    pub space_kind : SpaceKind,
}

impl ValInfo {
    pub fn new(space_kind: SpaceKind) -> ValInfo {
        return ValInfo {
            space_kind: space_kind,
        };
    }
}

