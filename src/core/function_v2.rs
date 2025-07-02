use wasmparser::{FunctionBody, Operator};
use anyhow::Result;
use crate::core::val::{WasmType, valtype_to_wasmtype};

use crate::core::module::Module;
use crate::core::opcode::OpInfo;

pub enum Function<'a> {
    ImportFunction(ImportFunction<'a>),
    BytecodeFunction(BytecodeFunction<'a>),
}

impl<'a> Function<'a> {
    pub fn module(&self) -> &Module<'a> {
        match self {
            Function::ImportFunction(f) => f.module,
            Function::BytecodeFunction(f) => f.module,
        }
    }
}

pub struct ImportFunction<'a> {
    pub module: &'a Module<'a>,
}

impl<'a> ImportFunction<'a> {
    pub fn new(module: &'a Module<'a>) -> Self {
        Self{module}
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
            .expect("Failed to get locals reader")
            .into_iter() {
            let (count, typ) = local.expect("Failed to read local");
            locals.extend(std::iter::repeat(valtype_to_wasmtype(&typ)).take(count as usize));
        }
        
        // debug
        println!("(fidx, local size): {:?}", (fidx, locals.len()));

        Self {
            module,
            body,
            locals,
        }
    }
    
    pub fn get_type_by_local(&self, local_idx: u32) -> &WasmType {
        return &self.locals[local_idx as usize];
    }
    
    pub fn create_stack_table(&self, _before_execution: bool) -> Result<Vec<CodePos>> {
        // 命令を取得
        let mut reader = self.body.get_operators_reader()?;
        let base_offset = reader.original_position() as u32;

        let mut stack = Stack::new();
        let mut stack_table = vec![];
        while !reader.eof() {
            let offset_before = reader.original_position() as u32 - base_offset;
            let op = reader.read()?;
            let opinfo = self.opinfo(&op);
            let offset_after = reader.original_position() as u32 - base_offset;

            // 入力適用
            stack_apply_input(&mut stack, &opinfo);

            // Call命令のときだけ、関数呼び出し直後の状態も特別に記録
            if matches!(op, Operator::Call { .. } | Operator::CallIndirect { .. }) {
                let call_site_offset = offset_before + 1;
                stack_table.push(CodePos::new(op.clone(), call_site_offset, stack.clone()));
            }

            // 出力適用
            stack_apply_output(&mut stack, &op, &opinfo);

            // 通常の命令記録（実行後の状態）
            stack_table.push(CodePos::new(op.clone(), offset_after, stack.clone()));
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

fn stack_apply_input<'a>(stack: &mut Stack<'a>, opinfo: &OpInfo) {
    let input = &opinfo.input;

    // pop
    let pop_len = input.len();
    let stack_len = stack.len();
    stack.inner.truncate(stack_len.saturating_sub(pop_len));
}

fn stack_apply_output<'a>(stack: &mut Stack<'a>, opcode: &Operator<'a>, opinfo: &OpInfo) {
    let output = &opinfo.output;

    // push
    for typ in output.iter() {
        // TODO: cloneを避ける
        stack.push((opcode.clone(), typ.clone()));
    }
}
