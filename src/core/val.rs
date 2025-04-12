use wasmparser::ValType;
use strum_macros::EnumString;
use serde::Serialize;

#[repr(u8)]
#[derive(Clone, Copy, EnumString, Serialize)]
pub enum WasmType {
    Any = 0,
    U8 = 1,
    I32,
    F32,
    I64,
    F64,
    V128,
    Ref,
}

impl WasmType {
    pub fn to_string(&self) -> &str {
        match self {
            WasmType::Any => return "Any",
            WasmType::U8 => return "U8",
            WasmType::I32 => return "I32",
            WasmType::F32 => return "F32",
            WasmType::I64 => return "I64",
            WasmType::F64 => return "F64",
            WasmType::V128 => return "V128",
            WasmType::Ref => return "Ref",
        }
    }
    
    pub fn size(&self) -> u8 {
        match self {
            WasmType::Any => return 0,
            WasmType::U8 => return 1,
            WasmType::I32 | WasmType::F32 => return 4,
            WasmType::I64 | WasmType::F64 => return 8,
            WasmType::V128 => return 16,
            WasmType::Ref => return 255,
        }
    }
}

pub fn valtype_to_wasmtype(valtype: &ValType) -> WasmType {
    match valtype {
        ValType::I32 => return WasmType::I32,
        ValType::I64 => return WasmType::I64,
        ValType::F32 => return WasmType::F32,
        ValType::F64 => return WasmType::F64,
        ValType::V128 => return WasmType::V128,
        ValType::Ref(..) => return WasmType::Ref,
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

