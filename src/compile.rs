use wasmparser::{FunctionBody, Operator, ValType, FuncType, BlockType};
use crate::core::function::{BytecodeFunction, Function, CodePos};
use crate::core::module::Module;
use anyhow::Result;

pub struct FastBytecodeFunction<'a> {
    pub locals: Vec<u8>,
    pub codes: Vec<FastCodePos<'a>>,

    // module: &'a Module<'a>,
    // else_blockty: BlockType,
    // func_body: &'a FunctionBody<'a>,
}


// const BYTE_U8: u8 = 1;
// const BYTE_U32: u8 = 4;
// const BYTE_U64: u8 = 8;
// const BYTE_U128: u8 = 16;

#[derive(Clone, Copy)]
enum WasmType {
    Any = 0,
    ByteU8 = 1,
    ByteU32 = 4,
    ByteU64 = 8,
    ByteU128 = 16,
}

fn valtype_to_wasmtype(valtype: ValType) -> WasmType {
    match valtype {
        ValType::I32 | ValType::F32 => return WasmType::ByteU32,
        ValType::I64 | ValType::F64 => return WasmType::ByteU64,
        // TODO: refはBYTE_U128じゃなさそう
        ValType::V128 | ValType::Ref(_)=> return WasmType::ByteU128,
    }
}

fn u8_to_wasmtype(num: u8) -> WasmType {
    match num {
        4 => return WasmType::ByteU32,
        8 => return WasmType::ByteU64,
        16 => return WasmType::ByteU128,
        _ => return WasmType::Any,
    }
}

#[derive(Clone)]
pub struct OpType {
    input: Vec<WasmType>,
    output: Vec<WasmType>,
}

#[derive(Clone)]
pub struct FastCodePos<'a> {
    pub opcode: Operator<'a>,
    // TODO: WasmTypeを使う
    pub optype: OpType,
    pub offset: u32,
}

impl<'a> FastBytecodeFunction<'a> {
    pub fn construct(module: Module, funcs: &Vec<Function<>>) -> Result<Vec<FastBytecodeFunction<'a>>> { 
        let mut compiled_funcs = Vec::new();

        // funcsを関数ごとにループする
        for func in funcs {
            match func {
                Function::ImportFunction(_) => continue,
                Function::BytecodeFunction(b)=> {
                    let fast_bytecode = Self::compile_fast_bytecode(module, b);
                    compiled_funcs.push(fast_bytecode);
                },
            }
        }


        return Ok(compiled_funcs);
    }

    fn emit_label(codepos: &CodePos<'a>, input: Vec<WasmType> , output: Vec<WasmType>) -> FastCodePos<'a> {
        let f = FastCodePos {
            opcode: codepos.opcode.clone(),
            optype: OpType { input: input, output: output},
            offset: codepos.offset,
        };

        return f;
    }

    pub fn compile_fast_bytecode(module: &Module, func: &'a BytecodeFunction) {
        // 仮スタック
        let mut stack: Vec<WasmType> = Vec::new();
        let mut fast_bytecode: Vec<FastCodePos> = Vec::new();

        for codepos in &func.codes {
            match codepos.opcode {
                Operator::Unreachable => {
                }
                Operator::Nop => {
                    // skip_label
                }
                Operator::Block{ .. } => {
                    // skip_label
                }
                Operator::Loop{ .. } => {
                    // skip_label
                }
                Operator::If{ blockty} => {
                }
                Operator::Else{ .. } => {
                }
                Operator::End{ .. } => {
                    // TODO: 挙動をちゃんと調べる
                }
                Operator::Br{ .. } => {
                }
                Operator::BrIf{ .. } => {
                }
                Operator::BrTable{ .. } => {
                }
                Operator::Return{ .. } => {
                }
                Operator::Call{ function_index } => {
                }
                Operator::CallIndirect{ type_index , .. } => {
                }
                Operator::Drop{ .. } => {
                    // skip_label
                }
                Operator::Select{ .. } => {
                }
                Operator::TypedSelect{ .. } => {
                }

                Operator::LocalGet{ local_index } => {
                    // skip_label
                    // local_indexの型をstackにpushする
                    stack.push(u8_to_wasmtype(func.locals[local_index as usize]));
                }
                Operator::LocalSet{ .. } => {
                    // TODO: local.getをskipすることで発生するごにょごにょを処理しないといけない
                }
                Operator::LocalTee{ .. } => {
                    // TODO: local.getをskipすることで発生するごにょごにょを処理しないといけない
                }
                Operator::GlobalGet{ global_index } => {
                    fast_bytecode.push(
                        Self::emit_label(codepos, vec![], 
                            vec![valtype_to_wasmtype(module.globals[global_index as usize].content_type)]));
                }
                Operator::GlobalSet{ .. } => {
                    fast_bytecode.push(Self::emit_label(codepos, vec![WasmType::Any], vec![]));
                }
                Operator::TableGet{ .. } => {
                    fast_bytecode.push(Self::emit_label(codepos, vec![WasmType::Any], vec![]));
                }
                Operator::TableSet{ .. } => {
                }

                Operator::I32Load{ .. } => {
                }
                Operator::I64Load{ .. } => {
                }
                Operator::F32Load{ .. } => {
                }
                Operator::F64Load{ .. } => {
                }
                Operator::I32Load8S{ .. } | Operator::I32Load8U{ .. } | Operator::I32Load16S{ .. } | Operator::I32Load16U{ .. } => {
                }
                Operator::I64Load8S{ .. } | Operator::I64Load8U{ .. } | Operator::I64Load16S{ .. } | Operator::I64Load16U{ .. } | Operator::I64Load32S{ .. } | Operator::I64Load32U{ .. } => {
                }
                Operator::I32Store{ .. } | Operator::I64Store{ .. } | Operator::F32Store{ .. } | Operator::F64Store{ .. } 
                | Operator::I32Store8{ .. } | Operator::I32Store16 { .. } | Operator::I64Store8{ .. } | Operator::I64Store16 { .. } | Operator::I64Store32{ .. } => {
                }
                Operator::MemorySize{ .. } => {
                }
                Operator::MemoryGrow{ .. } => {
                }

                Operator::I32Const{ .. } | Operator::F32Const{ .. } => {
                    // skip_label
                    // TODO: magic numberをやめる
                    stack.push(WasmType::ByteU32);
                }
                Operator::I64Const{ .. } | Operator::F64Const{ .. } => {
                    // skip_label
                    // TODO: magic numberをやめる
                    stack.push(WasmType::ByteU64);
                }

                Operator::I32Eqz{ .. } => {
                }
                Operator::I32Eq | Operator::I32Ne | Operator::I32LtS | Operator::I32LtU | Operator::I32GtS | Operator::I32GtU
                | Operator::I32LeS | Operator::I32LeU | Operator::I32GeS | Operator::I32GeU => {
                }
                Operator::I64Eqz{ .. } => {
                }
                Operator::I64Eq | Operator::I64Ne | Operator::I64LtS | Operator::I64LtU | Operator::I64GtS | Operator::I64GtU
                | Operator::I64LeS | Operator::I64LeU | Operator::I64GeS | Operator::I64GeU => {
                }
                Operator::F32Eq | Operator::F32Ne | Operator::F32Lt | Operator::F32Gt | Operator::F32Le | Operator::F32Ge => {
                }
                Operator::F64Eq | Operator::F64Ne | Operator::F64Lt | Operator::F64Gt | Operator::F64Le | Operator::F64Ge => {
                }

                Operator::I32Clz | Operator::I32Ctz | Operator::I32Popcnt => {
                }
                Operator::I32Add | Operator::I32Sub | Operator::I32Mul | Operator::I32DivS | Operator::I32DivU | Operator::I32RemS | Operator::I32RemU => {
                }
                Operator::I32And | Operator::I32Or | Operator::I32Xor | Operator::I32Shl | Operator::I32ShrS | Operator::I32ShrU | Operator::I32Rotl | Operator::I32Rotr => {
                }
                Operator::I64Clz | Operator::I64Ctz | Operator::I64Popcnt => {
                }
                Operator::I64Add | Operator::I64Sub | Operator::I64Mul | Operator::I64DivS | Operator::I64DivU | Operator::I64RemS | Operator::I64RemU => {
                }
                Operator::I64And | Operator::I64Or | Operator::I64Xor | Operator::I64Shl | Operator::I64ShrS | Operator::I64ShrU | Operator::I64Rotl | Operator::I64Rotr => {
                }

                Operator::F32Abs | Operator::F32Neg | Operator::F32Ceil | Operator::F32Floor | Operator::F32Trunc | Operator::F32Nearest | Operator::F32Sqrt => {
                }
                Operator::F32Add | Operator::F32Sub | Operator::F32Mul | Operator::F32Div | Operator::F32Min | Operator::F32Max | Operator::F32Copysign => {
                }
                Operator::F64Abs | Operator::F64Neg | Operator::F64Ceil | Operator::F64Floor | Operator::F64Trunc | Operator::F64Nearest | Operator::F64Sqrt => {
                }
                Operator::F64Add | Operator::F64Sub | Operator::F64Mul | Operator::F64Div | Operator::F64Min | Operator::F64Max | Operator::F64Copysign => {
                }

                Operator::I32WrapI64 => {
                }
                Operator::I32TruncF32S | Operator::I32TruncF32U => {
                }
                Operator::I32TruncF64S | Operator::I32TruncF64U => {
                }
                Operator::I64ExtendI32S | Operator::I64ExtendI32U => {
                }
                Operator::I64TruncF32S | Operator::I64TruncF32U => {
                }
                Operator::I64TruncF64S | Operator::I64TruncF64U => {
                }
                Operator::F32ConvertI32S | Operator::F32ConvertI32U => {
                }
                Operator::F32ConvertI64S | Operator::F32ConvertI64U => {
                }
                Operator::F32DemoteF64 => {
                }
                Operator::F64ConvertI32S | Operator::F64ConvertI32U => {
                }
                Operator::F64ConvertI64S | Operator::F64ConvertI64U => {
                }
                Operator::F64PromoteF32 => {
                }
                Operator::I32ReinterpretF32 | Operator::I64ReinterpretF64 | Operator::F32ReinterpretI32 | Operator::F64ReinterpretI64 => {
                }
                Operator::I32Extend8S | Operator::I32Extend16S | Operator::I64Extend8S | Operator::I64Extend16S | Operator::I64Extend32S => {
                }
                
                Operator::MemoryCopy{..} => {
                }
                Operator::MemoryFill{..} => {
                }

                ref _other => {
                    // println!("[WARN]: {:?}", op);
                    log::error!("Unsupprted operator");
                }
            }
        }
    }
}

