use wasmparser::{FunctionBody, Operator, ValType, FuncType, BlockType};
use anyhow::Result;

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
    pub locals: Vec<u8>,
    pub codes: Vec<CodePos<'a>>,

    module: &'a Module<'a>,
    else_blockty: BlockType,
    func_body: &'a FunctionBody<'a>,
}

#[derive(Clone)]
pub struct CodePos<'a> {
    pub opcode: Operator<'a>,
    pub offset: u32,
    pub type_stack: Vec<u8>,
    pub callee_return_size: u32,
}

impl<'a> BytecodeFunction<'a> {
    pub fn new(module: &'a Module<'a>, func_body: &'a FunctionBody<'a>, locals: Vec<u8>, else_blockty: BlockType, codes: Vec<CodePos<'a>>) -> Self {
        return Self{module, locals, else_blockty, func_body, codes};
    } 

    // pub fn construct(mut f: Function) -> Result<Vec<CodePos>> {
    pub fn construct(&mut self) -> Result<()> {
        let mut type_stack: Vec<u8> = Vec::new();
        let mut codes = Vec::<CodePos>::new();

        // funcを走査
        let mut reader = self.func_body.get_operators_reader()?;
        let base_offset = reader.original_position() as u32;

        // codesの先頭には、空のcodeposを入れておく. (offset=0を考慮するため)
        codes.push(CodePos{
            opcode: Operator::Nop,
            offset: 0,
            type_stack: vec![],
            callee_return_size: 0,
        });

        while !reader.eof() {
            let op = reader.read()?;
            let offset = reader.original_position() as u32 - base_offset;

            let result = self.dispatch(&mut type_stack, &op);
            if result.is_err() {
                log::error!("[ERROR]{:?}", result.err());
            }

            let mut callee_return_size: u32 = 0;

            // op=CALLのときのみ、[t1*]->[]の状態も出力する(リターンアドレスのため)
            match &op {
                Operator::Call{ function_index } => {
                    let callee_func_type: &FuncType = self.module.get_type_by_func(*function_index);
                    callee_return_size = callee_func_type.results().len() as u32;
                }

                _ => {}
            }

            // type stackを格納
            let codepos = CodePos {
                opcode: op,
                offset: offset,
                type_stack: type_stack.clone(),
                callee_return_size: callee_return_size,
            };

            codes.push(codepos);
        }

        self.codes = codes;

        return Ok(());
    }


    // 命令を1つ進める
    pub fn dispatch(&mut self, v: &mut Vec<u8>, op: &Operator<'_>) -> Result<u32, String> {
        match op {
            Operator::Unreachable => {
            }
            Operator::Nop => {
            }
            Operator::Block{ .. } => {
            }
            Operator::Loop{ .. } => {
            }
            Operator::If{ blockty} => {
                // [t1* i32] -> [t2*]
                v.pop();
                self.else_blockty = blockty.clone();
            }
            Operator::Else{ .. } => {
                match self.else_blockty {
                    BlockType::Empty => {
                    }
                    BlockType::Type( .. )=> {
                        v.pop();
                    }
                    BlockType::FuncType(type_idx) => {
                        // 関数型を持ってくる
                        let func_type = self.module.get_type_by_type(type_idx);

                        // 関数型の逆操作をする
                        let params = func_type.params();
                        let results = func_type.results();

                        for _ in results {
                            v.pop();
                        }
                        for vt in params {
                            v.push(valtype_to_size(vt));
                        }
                    }
                }
            }
            Operator::End{ .. } => {
            }
            Operator::Br{ .. } => {
                // [t1* t*] -> [t2*]
                v.clear();
            }
            Operator::BrIf{ .. } => {
                // [t* i32] -> [t*]
                v.pop();
            }
            Operator::BrTable{ .. } => {
                // [t1* t* i32] -> [t2*]
                v.pop();
            }
            Operator::Return{ .. } => {
                // [t1* t*] -> [t2*]
                v.clear();
                for vi in &mut *v {
                    println!("{vi}")
                }
                if v.len() != 0 {
                    // TODO: error型を返す
                    log::error!("Error: returnの処理が間違えている");
                    std::process::exit(0);
                }
            }
            Operator::Call{ function_index } => {
                // [t1*] -> [t2*]
                let func_type: &FuncType = self.module.get_type_by_func(*function_index);
                for _ in func_type.params() {
                    v.pop();
                }
                for param in func_type.results() {
                    v.push(valtype_to_size(param));
                }
            }
            Operator::CallIndirect{ type_index , .. } => {
                // [t1* i32] -> [t2*]
                v.pop();
                let func_type: &FuncType = self.module.get_type_by_type(*type_index);
                for _ in func_type.params() {
                    v.pop();
                }
                for result in func_type.results() {
                    v.push(valtype_to_size(result));
                }
            }
            Operator::Drop{ .. } => {
                // [t] -> []
                v.pop();
            }
            Operator::Select{ .. } => {
                // [t t i32] -> [t]
                v.pop();
                v.pop();
            }
            Operator::TypedSelect{ .. } => {
                // [t t i32] -> [t]
                v.pop();
                v.pop();
            }

            Operator::LocalGet{ local_index } => {
                // [] -> [t]
                v.push(self.locals[*local_index as usize]);
            }
            Operator::LocalSet{ .. } => {
                // [t] -> []
                v.pop();
            }
            Operator::LocalTee{ .. } => {
                // [t] -> [t]
            }
            Operator::GlobalGet{ global_index } => {
                // [] -> [t]
                let valtype = self.module.get_type_by_global(*global_index);
                v.push(valtype_to_size(valtype));
            }
            Operator::GlobalSet{ .. } => {
                // [t] -> []
                v.pop();
            }
            Operator::TableGet{ .. } => {
                // [i32] -> [t]
                v.pop();
                // reftypeの場合は型スタックは255とする
                v.push(255);
            }
            Operator::TableSet{ .. } => {
                // [i32 t] -> []
                v.pop();
                v.pop();
            }

            Operator::I32Load{ .. } => {
                // [i32] -> [i32]
            }
            Operator::I64Load{ .. } => {
                // [i32] -> [i64]
                v.pop();
                v.push(2);
            }
            Operator::F32Load{ .. } => {
                // [i32] -> [f32]
            }
            Operator::F64Load{ .. } => {
                // [i32] -> [f64]
                v.pop();
                v.push(2);
            }
            Operator::I32Load8S{ .. } | Operator::I32Load8U{ .. } | Operator::I32Load16S{ .. } | Operator::I32Load16U{ .. } => {
                // [i32] -> [i32]
            }
            Operator::I64Load8S{ .. } | Operator::I64Load8U{ .. } | Operator::I64Load16S{ .. } | Operator::I64Load16U{ .. } | Operator::I64Load32S{ .. } | Operator::I64Load32U{ .. } => {
                // [i32] -> [i64]
                v.pop();
                v.push(2);
            }
            Operator::I32Store{ .. } | Operator::I64Store{ .. } | Operator::F32Store{ .. } | Operator::F64Store{ .. } 
            | Operator::I32Store8{ .. } | Operator::I32Store16 { .. } | Operator::I64Store8{ .. } | Operator::I64Store16 { .. } | Operator::I64Store32{ .. } => {
                // [t t] -> []
                v.pop();
                v.pop();
            }
            Operator::MemorySize{ .. } => {
                // [] -> [i32]
                v.push(1);
            }
            Operator::MemoryGrow{ .. } => {
                // [i32] -> [i32]
            }
            Operator::I32Const{ .. } | Operator::F32Const{ .. } => {
                // [] -> [i32|f32]
                v.push(1);
            }
            Operator::I64Const{ .. } | Operator::F64Const{ .. } => {
                // [] -> [i64|f64]
                v.push(2);
            }

            Operator::I32Eqz{ .. } => {
                // [i32] -> [i32]
            }
            Operator::I32Eq | Operator::I32Ne | Operator::I32LtS | Operator::I32LtU | Operator::I32GtS | Operator::I32GtU
            | Operator::I32LeS | Operator::I32LeU | Operator::I32GeS | Operator::I32GeU => {
                // [i32 i32] -> [i32]
                v.pop();
            }
            Operator::I64Eqz{ .. } => {
                // [i64] -> [i32]
                v.pop();
                v.push(1);
            }
            Operator::I64Eq | Operator::I64Ne | Operator::I64LtS | Operator::I64LtU | Operator::I64GtS | Operator::I64GtU
            | Operator::I64LeS | Operator::I64LeU | Operator::I64GeS | Operator::I64GeU => {
                // [i64 i64] -> [i32]
                v.pop();
                v.pop();
                v.push(1);
            }
            Operator::F32Eq | Operator::F32Ne | Operator::F32Lt | Operator::F32Gt | Operator::F32Le | Operator::F32Ge => {
                // [f32 f32] -> [i32]
                v.pop();
            }
            Operator::F64Eq | Operator::F64Ne | Operator::F64Lt | Operator::F64Gt | Operator::F64Le | Operator::F64Ge => {
                // [f64 f64] -> [i32]
                v.pop();
                v.pop();
                v.push(1);
            }

            Operator::I32Clz | Operator::I32Ctz | Operator::I32Popcnt => {
                // [i32] -> [i32]
            }
            Operator::I32Add | Operator::I32Sub | Operator::I32Mul | Operator::I32DivS | Operator::I32DivU | Operator::I32RemS | Operator::I32RemU => {
                // [i32 i32] -> [i32]
                v.pop();
            }
            Operator::I32And | Operator::I32Or | Operator::I32Xor | Operator::I32Shl | Operator::I32ShrS | Operator::I32ShrU | Operator::I32Rotl | Operator::I32Rotr => {
                // [i32 i32] -> [i32]
                v.pop();
            }
            Operator::I64Clz | Operator::I64Ctz | Operator::I64Popcnt => {
                // [i64] -> [i64]
            }
            Operator::I64Add | Operator::I64Sub | Operator::I64Mul | Operator::I64DivS | Operator::I64DivU | Operator::I64RemS | Operator::I64RemU => {
                // [i64 i64] -> [i64]
                v.pop();
            }
            Operator::I64And | Operator::I64Or | Operator::I64Xor | Operator::I64Shl | Operator::I64ShrS | Operator::I64ShrU | Operator::I64Rotl | Operator::I64Rotr => {
                // [i64 i64] -> [i64]
                v.pop();
            }

            Operator::F32Abs | Operator::F32Neg | Operator::F32Ceil | Operator::F32Floor | Operator::F32Trunc | Operator::F32Nearest | Operator::F32Sqrt => {
                // [f32] -> [f32]
            }
            Operator::F32Add | Operator::F32Sub | Operator::F32Mul | Operator::F32Div | Operator::F32Min | Operator::F32Max | Operator::F32Copysign => {
                // [f32 f32] -> [f32]
                v.pop();
            }
            Operator::F64Abs | Operator::F64Neg | Operator::F64Ceil | Operator::F64Floor | Operator::F64Trunc | Operator::F64Nearest | Operator::F64Sqrt => {
                // [f64] -> [f64]
            }
            Operator::F64Add | Operator::F64Sub | Operator::F64Mul | Operator::F64Div | Operator::F64Min | Operator::F64Max | Operator::F64Copysign => {
                // [f64 f64] -> [f64]
                v.pop();
            }

            Operator::I32WrapI64 => {
                // [i64] -> [i32]
                v.pop();
                v.push(1);
            }
            Operator::I32TruncF32S | Operator::I32TruncF32U => {
                // [f32] -> [i32]
            }
            Operator::I32TruncF64S | Operator::I32TruncF64U => {
                // [f64] -> [i32]
                v.pop();
                v.push(1);
            }
            Operator::I64ExtendI32S | Operator::I64ExtendI32U => {
                // [i32] -> [i64]
                v.pop();
                v.push(2);
            }
            Operator::I64TruncF32S | Operator::I64TruncF32U => {
                // [f32] -> [i64]
                v.pop();
                v.push(2);
            }
            Operator::I64TruncF64S | Operator::I64TruncF64U => {
                // [f64] -> [i64]
            }
            Operator::F32ConvertI32S | Operator::F32ConvertI32U => {
                // [i32] -> [f32]
            }
            Operator::F32ConvertI64S | Operator::F32ConvertI64U => {
                // [i64] -> [f32]
                v.pop();
                v.push(1);
            }
            Operator::F32DemoteF64 => {
                // [i64] -> [f32]
                v.pop();
                v.push(1);
            }
            Operator::F64ConvertI32S | Operator::F64ConvertI32U => {
                // [i32] -> [f64]
                v.pop();
                v.push(2);
            }
            Operator::F64ConvertI64S | Operator::F64ConvertI64U => {
                // [i64] -> [f64]
            }
            Operator::F64PromoteF32 => {
                // [f32] -> [f64]
                v.pop();
                v.push(2);
            }
            Operator::I32ReinterpretF32 | Operator::I64ReinterpretF64 | Operator::F32ReinterpretI32 | Operator::F64ReinterpretI64 => {
                // [t] -> [t]
            }
            Operator::I32Extend8S | Operator::I32Extend16S | Operator::I64Extend8S | Operator::I64Extend16S | Operator::I64Extend32S => {
                // [t] -> [t]
            }
            
            Operator::MemoryCopy{..} => {
                // [i32 i32 i32] -> []
                v.pop();
                v.pop();
                v.pop();
            }
            Operator::MemoryFill{..} => {
                // [i32 i32 i32] -> []
                v.pop();
                v.pop();
                v.pop();
            }

            ref _other => {
                // println!("[WARN]: {:?}", op);
                return Err("Unsupprted operator".to_string());
            }
        }
        return Ok(0);
    }
}

pub fn valtype_to_size(valtype: &ValType) -> u8 {
    match valtype {
        ValType::I32 | ValType::F32 => return 1,
        ValType::I64 | ValType::F64 => return 2,
        ValType::V128 => return 4,
        ValType::Ref( .. ) => return 255,
    }
}
