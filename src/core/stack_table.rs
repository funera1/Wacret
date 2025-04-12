use crate::core::function_v2::{CodePos, Stack};
use crate::core::val::WasmType;
use serde::Serialize;
use wasmparser::Operator;

#[derive(Serialize)]
pub enum CompiledOp {
    LocalGet(u32),
    I32Const(i32),
    F32Const(u32),
    I64Const(i64),
    F64Const(u64),
    Other,
}

#[derive(Serialize)]
pub struct StEntry {
    // pub opecode: u8,
    // pub operand: Option<CompiledOp>,
    pub offset: u32,
    pub stack: Vec<(CompiledOp, WasmType)>,
}

impl StEntry {
    // pub fn new(operand: Option<CompiledOp>, offset: u32, stack: Vec<WasmType>) -> Self {
    //     Self {operand, offset, stack}
    // }
    
    pub fn from_codepos(codepos: CodePos) -> Self {
        Self {
            offset: codepos.offset,
            stack: codepos.stack.inner.into_iter().map(|(op, typ)| {
                let op = match op {
                    Operator::LocalGet { local_index } => CompiledOp::LocalGet(local_index),
                    Operator::I32Const { value } => CompiledOp::I32Const(value),
                    Operator::F32Const { value } => CompiledOp::F32Const(value.bits()),
                    Operator::I64Const { value } => CompiledOp::I64Const(value),
                    Operator::F64Const { value } => CompiledOp::F64Const(value.bits()),
                    _ => CompiledOp::Other,
                };
                (op, typ)
            }).collect(),
        }
    }
}

#[derive(Serialize)]
pub struct StackTable {
    pub inner: Vec<StEntry>,
}

impl StackTable {
    pub fn new(inner: Vec<StEntry>) -> Self {
        Self {inner}
    }
}