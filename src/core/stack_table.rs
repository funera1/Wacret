use crate::core::function_v2::{CodePos, Function};
use crate::core::val::WasmType;
use serde::{Serialize, Deserialize};
use wasmparser::Operator;
use anyhow::Result;

#[derive(Serialize, Deserialize)]
pub enum CompiledOp {
    LocalGet(u32),
    I32Const(i32),
    F32Const(u32),
    I64Const(i64),
    F64Const(u64),
    Other,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct StackTable {
    pub inner: Vec<StEntry>,
}

impl StackTable {
    pub fn new(inner: Vec<StEntry>) -> Self {
        Self {inner}
    }
}

#[derive(Serialize, Deserialize)]
pub struct StackTables(pub Vec<StackTable>);

impl StackTables {
    pub fn from_func(funcs: Vec<Function<'_>>) -> Result<Self> {
        let stack_tables= funcs.iter().map(|f| {
            match f {
                Function::ImportFunction(_) => {
                    return Vec::new();
                }
                Function::BytecodeFunction(f) => {
                    return f.create_stack_table().expect("failed to create stack table");
                }
            }
        }).collect::<Vec<_>>();

        // Vec<Vec<CodePos>>からStackTableに変換
        let stack_tables: Vec<StackTable> = stack_tables
            .into_iter()
            .map(|st| {
                let inner = st.into_iter().map(|entry| StEntry::from_codepos(entry)).collect();
                StackTable::new(inner)
            })
            .collect();
        
        Ok(StackTables(stack_tables))
    }

    pub fn serialize(&self) -> Vec<u8> {
        rmp_serde::to_vec_named(self).unwrap()
    }
    pub fn deserialize(data: &[u8]) -> Self {
        rmp_serde::from_slice(data).unwrap()
    } 
}

