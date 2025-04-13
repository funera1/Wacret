use wasmparser::{FunctionBody, Operator};
use anyhow::Result;
use crate::core::val::{WasmType, valtype_to_wasmtype};

use crate::core::module::Module;
use crate::core::opcode::OpInfo;

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
    pub stack: Stack<'a>,
}

impl<'a> CodePos<'a> {
    pub fn new(op: Operator<'a>, offset: u32, stack: Stack<'a>) -> Self {
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
        // offset取得
        let original_reader = self.body.get_binary_reader();
        let base_offset = original_reader.original_position() as u32;

        // 命令を取得
        let mut reader = self.body.get_operators_reader()?;
        
        let mut stack = Stack::new();
        let mut stack_table = vec![];
        while !reader.eof() {
            let offset = reader.original_position() as u32 - base_offset;
            let op = reader.read()?;
            
            let opinfo = self.opinfo(&op);
            stack_apply(&mut stack, &op, &opinfo);
            
            // stackをcopy
            stack_table.push(CodePos::new(op.clone(), offset, stack.clone()));
        }

        Ok(stack_table)
    }
}

#[derive(Clone)]
pub struct Stack<'a> {
    pub inner: Vec<(Operator<'a>, WasmType)>,
}

impl<'a> Stack<'a> {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }
    
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    
    pub fn push(&mut self, entry: (Operator<'a>, WasmType)) {
        self.inner.push(entry);
    }
}

fn stack_apply<'a>(stack: &mut Stack<'a>, opcode: &Operator<'a>, opinfo: &OpInfo) {
    let input = &opinfo.input;
    let output = &opinfo.output;

    // pop
    let pop_len = input.len();
    let stack_len = stack.len();
    stack.inner.truncate(stack_len.saturating_sub(pop_len));
    
    // push
    for typ in output.iter() {
        // TODO: cloneを避ける
        stack.push((opcode.clone(), typ.clone()));
    }
}