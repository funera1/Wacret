use anyhow::Result;
use wasmparser::{Parser, Payload, TypeRef};
use wasmparser::{FunctionBody, FuncType, GlobalType, BlockType, ValType};

use crate::core::function::{Function, BytecodeFunction, ImportFunction, CodePos, valtype_to_size};
use crate::core::function_v2;

pub struct Fn<'a> {
    pub fidx: u32,
    pub body: Option<FunctionBody<'a>>,
}

pub struct Module<'a> {
    pub types: Vec<FuncType>,
    pub funcs: Vec<Fn<'a>>,
    pub globals: Vec<GlobalType>,
}

impl<'a> Module<'a> {
    pub fn new(types: Vec<FuncType>, funcs: Vec<Fn<'a>>, globals: Vec<GlobalType>) -> Self {
        Self {
            types,
            funcs,
            globals,
        }
    }

    pub fn get_type_by_func(&self, func_idx: u32) -> &FuncType {
        return &self.types[self.funcs[func_idx as usize].fidx as usize];
    }

    pub fn get_type_by_type(&self, type_idx: u32) -> &FuncType {
        return &self.types[type_idx as usize];
    }

    pub fn get_type_by_global(&self, global_idx: u32) -> &ValType {
        return &self.globals[global_idx as usize].content_type;
    }

    pub fn parse(&self) -> Result<Vec<Function>> {
        let mut ret : Vec<Function> = vec![];

        for i in 0..self.funcs.len() as u32 {
            let body = &self.funcs[i as usize].body;
            match body {
                Some(body) => {
                    let else_blockty = BlockType::Empty;
                    let locals = self.get_locals(i)?;
                    log::debug!("local size in {}th function: {}", i, locals.to_vec().len());

                    let v: Vec<CodePos<'_>> = vec![];
                    let mut f = BytecodeFunction::new(&self, &body, locals.to_vec(), else_blockty, v.to_vec());
                    let _ = f.construct();
                    ret.push(Function::BytecodeFunction(f));
                }
                None => {
                    log::debug!("{}th function is import_function", i);
                    let f = ImportFunction::new();
                    ret.push(Function::ImportFunction(f));
                },
            };
        }
        return Ok(ret);
    }

    pub fn new_function_v2(&self) -> Result<Vec<function_v2::Function>> {
        let ret: Vec<function_v2::Function> = self
            .funcs
            .iter()
            .enumerate()
            .map(|(i, func)| {
                match &func.body {
                    Some(body) => {
                        let f = function_v2::BytecodeFunction::new(self, body, i as u32);
                        function_v2::Function::BytecodeFunction(f)
                    }
                    None => {
                        log::debug!("{}th function is import_function", i);
                        let f = function_v2::ImportFunction::new();
                        function_v2::Function::ImportFunction(f)
                    }
                }
            })
            .collect();
        Ok(ret)    
    }

    pub fn get_locals(&self, fidx: u32) -> Result<Vec<u8>> {
        // ローカルのVecを生成
        let mut locals: Vec<u8> = Vec::new();

        // 引数をpush
        let func_type = self.get_type_by_func(fidx);
        let params = func_type.params();
        for param in params {
            locals.push(valtype_to_size(&param));
        }

        // ローカルをpush
        let body = &self.funcs[fidx as usize].body;
        // let body = &self.bodies[fidx as usize];
        match body {
            Some(b) => {
                let locals_iter = b.get_locals_reader()?.into_iter();
                for local in locals_iter {
                    let (count, typ) = local?;
                    for _ in 0..count {
                        locals.push(valtype_to_size(&typ));
                    }
                }

                return Ok(locals);
            }
            None => return Ok(vec![]),
        }
    }

}

pub fn new_module(buf: &Vec<u8>) -> Result<Module> {
    let mut globals: Vec<GlobalType> = Vec::new();
    let mut codes: Vec<FunctionBody> = Vec::new();
    let mut types: Vec<FuncType> = Vec::new();
    let mut bytecode_funcs: Vec<u32> = Vec::new();
    let mut import_funcs: Vec<u32> = Vec::new();

    for payload in Parser::new(0).parse_all(&buf) {
        match payload? {
            Payload::TypeSection(type_reader) => {
                let type_iter = type_reader.into_iter_err_on_gc_types();
                for func_type in type_iter {
                    let res = func_type?;
                    types.push(res);
                }
            }
            Payload::FunctionSection(func_reader) => {
                let func_iter = func_reader.into_iter_with_offsets();
                for func in func_iter {
                    // func: (usize, u32)
                    // u32の方は型インデックスぽい
                    // func_idx -> type_idx -> 引数と返り値のペアがわかったので、Callを展開できる
                    let (_, type_idx) = func?;
                    bytecode_funcs.push(type_idx);
                }

            }
            Payload::ImportSection(import_reader) => {
                let import_iter = import_reader.into_iter_with_offsets();
                for import in import_iter {
                    let (_, imp) = import?;
                    let type_ref = imp.ty;

                    match type_ref {
                        TypeRef::Func(func_idx) => {
                            import_funcs.push(func_idx);
                        }
                        _other => {
                        }
                    }
                }
            }
            Payload::GlobalSection(global_reader) => {
                let global_iter = global_reader.into_iter_with_offsets();
                for global in global_iter {
                    let (_, glob) = global?;
                    globals.push(glob.ty);
                }
            }
            Payload::CodeSectionEntry(body) => {
                codes.push(body);
            }
            _other => {
            }
        }
    }

    // import関数とbytecode関数をマージ
    let mut funcs: Vec<Fn<'_>> = Vec::new();
    for func_idx in 0..import_funcs.len() {
        let type_idx = import_funcs[func_idx];
        funcs.push(Fn{fidx: type_idx, body: None});
    }
    for func_idx in 0..bytecode_funcs.len() {
        let type_idx = bytecode_funcs[func_idx];
        // TODO: cloneしているが、本当にそれしかないのか?
        funcs.push(Fn{fidx: type_idx, body: Some(codes[func_idx].clone())});
    }

    return Ok(Module::new(types, funcs, globals));
}