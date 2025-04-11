use wasmparser::{FunctionBody, Operator, ValType, FuncType, BlockType};
use anyhow::Result;
use crate::core::val::{WasmType, valtype_to_wasmtype};

use crate::core::module::Module;

pub enum Function<'a> {
    ImportFunction(ImportFunction),
    BytecodeFunction(BytecodeFunction<'a>),
}

pub struct ImportFunction {

}

impl ImportFunction {
    pub fn new() -> Self {
        Self{}
    }
}

#[derive(Clone)]
pub struct BytecodeFunction<'a> {
    pub module: &'a Module<'a>,
    pub body: &'a FunctionBody<'a>,
    pub locals: Vec<WasmType>,
}

impl<'a> BytecodeFunction<'a> {
    pub fn new(module: &'a Module<'a>, body: &'a FunctionBody<'a>, fidx: u32) -> Self {
        let mut locals = module.get_type_by_func(fidx)
            .params()
            .iter()
            .map(valtype_to_wasmtype)
            .collect::<Vec<_>>();

        for local in body.get_locals_reader()?.into_iter() {
            let (count, typ) = local?;
            locals.extend(std::iter::repeat(valtype_to_wasmtype(&typ)).take(count as usize));
        }

        Self {
            module,
            body,
            locals,
        }
    }
    
    pub fn get_type_by_local(&self, local_idx: u32) -> &WasmType {
        return &self.locals[local_idx as usize];
    }
}