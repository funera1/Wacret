use anyhow::Result;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use wasmparser::Operator;

use crate::core::function_v2::{CodePos, Function};
use crate::core::val::WasmType;

#[derive(Serialize, Deserialize)]
pub enum CompiledOp {
    LocalGet(u32),
    I32Const(i32),
    F32Const(u32),
    I64Const(i64),
    F64Const(u64),
    Other(WasmType),
}


pub type Offset = u32;
pub type Stack = Vec<(CompiledOp, WasmType)>;
#[derive(Serialize, Deserialize)]
pub struct StackTable {
    inner: IndexMap<Offset, Stack>,
}

impl StackTable {
    pub fn new(inner: IndexMap<Offset, Stack>) -> Self {
        Self { inner }
    }
}

#[derive(Serialize, Deserialize)]
pub struct StackTables(pub Vec<StackTable>);

impl StackTables {
    /// 関数リストから StackTables を構築する
    pub fn from_func(funcs: Vec<Function<'_>>) -> Result<Self> {
        let stack_tables = funcs
            .iter()
            .map(|f| match f {
                Function::ImportFunction(_) => Vec::new(),
                Function::BytecodeFunction(f) => {
                    f.create_stack_table().expect("Failed to create stack table from BytecodeFunction")
                }
            })
            .collect::<Vec<_>>();

        // Vec<Vec<CodePos>> → Vec<StackTable> に変換
        let stack_tables = stack_tables
            .into_iter()
            .map(|st| {
                let inner = st.into_iter().map(from_codepos).collect();
                StackTable::new(inner)
            })
            .collect();

        Ok(StackTables(stack_tables))
    }

    pub fn serialize(&self) -> Vec<u8> {
        rmp_serde::to_vec_named(self).expect("Failed to serialize StackTables")
    }

    pub fn deserialize(data: &[u8]) -> Self {
        rmp_serde::from_slice(data).expect("Failed to deserialize StackTables")
    }
    
    pub fn get_stack(&self, fidx: usize, offset: u32) -> Result<&Stack> {
        let s = &self.0[fidx];
        s.inner
            .get(&offset)
            .ok_or_else(|| anyhow::anyhow!("Stack not found for offset {}", offset))
    }

    pub fn get_stack_nth(&self, fidx: usize, n: usize) -> Result<&Stack> {
        let s = &self.0[fidx];
        let a = s.inner
            .get_index(n);
        if let Some((_, stack)) = a {
            Ok(stack)
        } else {
            Err(anyhow::anyhow!("Stack not found for index {}", n))
        }
    }
}

/// CodePos → (Offset, Stack) に変換
pub fn from_codepos(codepos: CodePos) -> (Offset, Stack) {
    let offset = codepos.offset;

    let stack_vec = codepos
        .stack
        .inner
        .into_iter()
        .map(|(op, typ)| {
            let op = match op {
                Operator::LocalGet { local_index } => CompiledOp::LocalGet(local_index),
                Operator::I32Const { value } => CompiledOp::I32Const(value),
                Operator::F32Const { value } => CompiledOp::F32Const(value.bits()),
                Operator::I64Const { value } => CompiledOp::I64Const(value),
                Operator::F64Const { value } => CompiledOp::F64Const(value.bits()),
                _ => CompiledOp::Other(typ),
            };
            (op, typ)
        })
        .collect();

    (offset, stack_vec)
}