use wasmparser::{FunctionBody, Operator};
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

pub struct CodePos<'a> {
    // TODO: opecodeの扱いを考え直す. 
    pub op: Operator<'a>,
    pub offset: u32,
    pub stack: Vec<WasmType>,
}

impl<'a> CodePos<'a> {
    pub fn new(op: Operator<'a>, offset: u32, stack: Vec<WasmType>) -> Self {
        Self {op, offset, stack}
    }
}

impl<'a> BytecodeFunction<'a> {
    pub fn new(module: &'a Module<'a>, body: &'a FunctionBody<'a>, fidx: u32) -> Self {
        let mut locals = module.get_type_by_func(fidx)
            .params()
            .iter()
            .map(valtype_to_wasmtype)
            .collect::<Vec<_>>();

        for local in body.get_locals_reader()
            .expect("failed to get locals reader")
            .into_iter() {
            let (count, typ) = local.expect("failed to get local");
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
    
    pub fn create_stack_table(&self) -> Result<Vec<CodePos>> {
        let mut reader = self.body.get_operators_reader()?;
        let base_offset = reader.original_position() as u32;
        
        let mut stack = vec![];
        let mut stack_table = vec![];
        while !reader.eof() {
            let op = reader.read()?;
            let offset = reader.original_position() as u32 - base_offset;
            
            let opinfo = self.opinfo(&op);
            stack_apply(&mut stack, &opinfo.input, &opinfo.output);
            
            // stackをcopy
            stack_table.push(CodePos::new(op.clone(), offset, stack.clone()));
        }

        Ok(stack_table)
    }
}

fn stack_apply(stack: &mut Vec<WasmType>, input: &Vec<WasmType>, output: &Vec<WasmType>) {
    // pop
    let pop_len = input.len();
    let stack_len = stack.len();
    stack.truncate(stack_len.saturating_sub(pop_len));
    
    // push
    stack.extend(output);
}