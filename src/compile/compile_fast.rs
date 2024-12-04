use wasmparser::Operator;
use crate::core::function::{BytecodeFunction, CodePos};
use crate::core::module::Module;

use crate::compile::compile::*;

#[derive(Clone)]
pub struct FastCodePos<'a> {
    pub opcode: Operator<'a>,
    pub optype: OpType,
    pub offset: u32,
}

pub struct FastBytecodeFunction<'a> {
    pub locals: Vec<u8>,
    pub codes: Vec<FastCodePos<'a>>,

    // module: &'a Module<'a>,
    // else_blockty: BlockType,
    // func_body: &'a FunctionBody<'a>,
}

impl<'a> FastBytecodeFunction<'a> {
    pub fn new(module: &Module, func: &BytecodeFunction<'a>) -> Self { 
        return FastBytecodeFunction {
            locals: func.locals.clone(),
            codes: Self::compile_fast_bytecode(module, &func.codes),
        };
    }

    fn emit_label(codepos: &CodePos<'a>, input: Vec<WasmType> , output: Vec<WasmType>) -> FastCodePos<'a> {
        let f = FastCodePos {
            opcode: codepos.opcode.clone(),
            optype: OpType { input: input, output: output},
            offset: codepos.offset,
        };

        return f;
    }

    // TODO: わかりやすい名前に変える
    fn popf(stack: &mut Vec<ValInfo>, default_type: Vec<WasmType>) -> Vec<WasmType> {
        let mut input = vec![];
        for t in default_type {
            let val_info = stack.pop().expect("[ERROR] stack is empty");
            match val_info.space_kind {
                SpaceKind::Dynamic => input.push(t),
                SpaceKind::Static => {},
            }
        }
        return input;
    }

    // TODO: わかりやすい名前に変える
    fn pushf(stack: &mut Vec<ValInfo>, default_type: Vec<WasmType>) -> Vec<WasmType> {
        for _ in 0..default_type.len() {
            stack.push(ValInfo::new(SpaceKind::Dynamic));
        }
        
        return default_type;
    }

    pub fn compile_fast_bytecode(module: &Module, codes: &Vec<CodePos<'a>>) -> Vec<FastCodePos<'a>> {
        // 仮スタック
        // TODO: 多分stackにはコード位置も入る
        let mut stack: Vec<ValInfo> = Vec::new();
        let mut fast_bytecode: Vec<FastCodePos> = Vec::new();

        for codepos in codes {
            match codepos.opcode {
                Operator::Unreachable => {
                }
                Operator::Nop => {
                    // skip_label
                }
                Operator::Block{ .. } => {
                    // skip_label
                    // TODO: COPY_STACKをemitする
                }
                Operator::Loop{ .. } => {
                    // skip_label
                }
                Operator::If{ .. } => {
                }
                Operator::Else{ .. } => {
                }
                Operator::End{ .. } => {
                    // TODO: 挙動をちゃんと調べる
                }
                Operator::Br{..} => {
                    // [t1*, t*] -> [t2*]
                    // NOTE: この位置では型スタックは変化しない
                    let i: Vec<WasmType> = vec![];
                    let o: Vec<WasmType> = vec![];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::BrIf{..} => {
                    // [t1*, U32] -> [t2*]
                    let i: Vec<WasmType> = vec![WasmType::U32];
                    let o: Vec<WasmType> = vec![];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::BrTable{..} => {
                    // [t1*, t*, U32] -> [t2*]
                    let i: Vec<WasmType> = vec![WasmType::U32];
                    let o: Vec<WasmType> = vec![];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::Return{ .. } => {
                    // TODO: ほんとにbreakで良いのか確認
                    break;
                }
                Operator::Call{ function_index } => {
                    // [Args*] -> [Rets*]
                    let f = module.get_type_by_func(function_index);
                    let params = f.params();
                    let results = f.results();

                    let i: Vec<WasmType> = params.iter().map(valtype_to_wasmtype).collect();
                    let o: Vec<WasmType> = results.iter().map(valtype_to_wasmtype).collect();

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::CallIndirect{ type_index , .. } => {
                    // [Args*, U32] -> [Rets*]
                    let f = module.get_type_by_type(type_index);
                    let params = f.params();
                    let results = f.results();

                    let i: Vec<WasmType> = params.iter().map(valtype_to_wasmtype)
                                                 .chain(std::iter::once(WasmType::U32)).collect();
                    let o: Vec<WasmType> = results.iter().map(valtype_to_wasmtype).collect();

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::Drop{ .. } => {
                    // [Any] -> []
                    // skip_label
                    // let _ = stack.pop().expect("[ERROR] stack is empty");
                    
                    // TODO: skip_labelにする（ただし、stack.popするだけだとcalc_stackのときに反映されないのでどうするか考える)
                    let i = vec![WasmType::Any];
                    let o = vec![];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::Select{ .. } => {
                    // [Any, Any, U32] -> [Any]
                    let i = vec![WasmType::Any, WasmType::Any, WasmType::U32];
                    let o = vec![WasmType::Any];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::TypedSelect{ .. } => {
                    // [Any, Any, U32] -> [Any]
                    let i = vec![WasmType::Any, WasmType::Any, WasmType::U32];
                    let o = vec![WasmType::Any];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }

                Operator::LocalGet{ local_index } => {
                    // [] -> [Any]
                    // skip_label
                    // local_indexの型をstackにpushする
                    stack.push(ValInfo::new(SpaceKind::Static));
                }
                Operator::LocalSet{ .. } => {
                    // [Any] -> []
                    // NOTE: 理解しやすくするために簡単にしている。
                    // NOTE: preserveのためにCOPY命令が挿入されたり、LOCAL_SET命令が喪失したりするが一旦考慮しない
                    let i = vec![WasmType::Any];
                    let o = vec![];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::LocalTee{ .. } => {
                    // [Any] -> [Any]
                    // TODO: local.getをskipすることで発生するごにょごにょを処理しないといけない
                    let i = vec![WasmType::Any];
                    let o = vec![WasmType::Any];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::GlobalGet{ global_index } => {
                    // [] -> [Any]
                    let i = vec![];
                    let o = vec![WasmType::Any];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::GlobalSet{ .. } => {
                    // [Any] -> []
                    let i = vec![WasmType::Any];
                    let o = vec![];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::TableGet{ .. } => {
                    // [U32] -> [Any]
                    let i = vec![WasmType::U32];
                    let o = vec![WasmType::Any];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::TableSet{ .. } => {
                    // [U32, Any] -> []
                    let i = vec![WasmType::U32, WasmType::Any];
                    let o = vec![];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }

                Operator::I32Load{ .. } | Operator::F32Load{ .. } | 
                Operator::I32Load8S{ .. } | Operator::I32Load8U{ .. } | Operator::I32Load16S{ .. } | Operator::I32Load16U{ .. } => {
                    // [U32] -> [U32]
                    let i = vec![WasmType::U32];
                    let o = vec![WasmType::U32];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::I64Load{ .. } | Operator::F64Load{ .. } |
                Operator::I64Load8S{ .. } | Operator::I64Load8U{ .. } | Operator::I64Load16S{ .. } | Operator::I64Load16U{ .. } | Operator::I64Load32S{ .. } | Operator::I64Load32U{ .. } => {
                    // [U32] -> [U64]
                    let i = vec![WasmType::U32];
                    let o = vec![WasmType::U64];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }

                Operator::I32Store{ .. } | Operator::I64Store{ .. } | Operator::F32Store{ .. } | Operator::F64Store{ .. } 
                | Operator::I32Store8{ .. } | Operator::I32Store16 { .. } | Operator::I64Store8{ .. } | Operator::I64Store16 { .. } | Operator::I64Store32{ .. } => {
                    // [U32, U32] -> []
                    let i = vec![WasmType::U32, WasmType::U32];
                    let o = vec![];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::MemorySize{ .. } => {
                    // [] -> [U32]
                    let i = vec![];
                    let o = vec![WasmType::U32];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::MemoryGrow{ .. } => {
                    // [U32] -> [U32]
                    let i = vec![WasmType::U32];
                    let o = vec![WasmType::U32];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }

                Operator::I32Const{ .. } | Operator::F32Const{ .. } |
                Operator::I64Const{ .. } | Operator::F64Const{ .. } => {
                    // skip_label
                    stack.push(ValInfo::new(SpaceKind::Static));
                }

                Operator::I32Eqz{ .. } => {
                    // [U32] -> [U32]
                    let i = vec![WasmType::U32];
                    let o = vec![WasmType::U32];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::I32Eq | Operator::I32Ne | Operator::I32LtS | Operator::I32LtU | Operator::I32GtS | Operator::I32GtU
                | Operator::I32LeS | Operator::I32LeU | Operator::I32GeS | Operator::I32GeU 
                | Operator::F32Eq | Operator::F32Ne | Operator::F32Lt | Operator::F32Gt | Operator::F32Le | Operator::F32Ge => {
                    // [U32, U32] -> [U32]
                    let i = vec![WasmType::U32, WasmType::U32];
                    let o = vec![WasmType::U32];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::I64Eqz{ .. } => {
                    // [U64] -> [U32]
                    let i = vec![WasmType::U64];
                    let o = vec![WasmType::U32];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::I64Eq | Operator::I64Ne | Operator::I64LtS | Operator::I64LtU | Operator::I64GtS | Operator::I64GtU
                | Operator::I64LeS | Operator::I64LeU | Operator::I64GeS | Operator::I64GeU
                | Operator::F64Eq | Operator::F64Ne | Operator::F64Lt | Operator::F64Gt | Operator::F64Le | Operator::F64Ge => {
                    // [U64, U64] -> [U32]
                    let i = vec![WasmType::U64, WasmType::U64];
                    let o = vec![WasmType::U32];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::I32Clz | Operator::I32Ctz | Operator::I32Popcnt => {
                    // [U32] -> [U32]
                    let i = vec![WasmType::U32];
                    let o = vec![WasmType::U32];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::I32Add | Operator::I32Sub | Operator::I32Mul | Operator::I32DivS | Operator::I32DivU | Operator::I32RemS | Operator::I32RemU => {
                    // [U32, U32] -> [U32]
                    let i = vec![WasmType::U32, WasmType::U32];
                    let o = vec![WasmType::U32];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::I32And | Operator::I32Or | Operator::I32Xor | Operator::I32Shl | Operator::I32ShrS | Operator::I32ShrU | Operator::I32Rotl | Operator::I32Rotr => {
                    // [U32, U32] -> [U32]
                    let i = vec![WasmType::U32, WasmType::U32];
                    let o = vec![WasmType::U32];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::I64Clz | Operator::I64Ctz | Operator::I64Popcnt => {
                    // [U64] -> [U64]
                    let i = vec![WasmType::U64];
                    let o = vec![WasmType::U64];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::I64Add | Operator::I64Sub | Operator::I64Mul | Operator::I64DivS | Operator::I64DivU | Operator::I64RemS | Operator::I64RemU => {
                    // [U64, U64] -> [U64]
                    let i = vec![WasmType::U64, WasmType::U64];
                    let o = vec![WasmType::U64];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::I64And | Operator::I64Or | Operator::I64Xor | Operator::I64Shl | Operator::I64ShrS | Operator::I64ShrU | Operator::I64Rotl | Operator::I64Rotr => {
                    // [U64, U64] -> [U64]
                    let i = vec![WasmType::U64, WasmType::U64];
                    let o = vec![WasmType::U64];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }

                Operator::F32Abs | Operator::F32Neg | Operator::F32Ceil | Operator::F32Floor | Operator::F32Trunc | Operator::F32Nearest | Operator::F32Sqrt => {
                    // [U32] -> [U32]
                    let i = vec![WasmType::U32];
                    let o = vec![WasmType::U32];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::F32Add | Operator::F32Sub | Operator::F32Mul | Operator::F32Div | Operator::F32Min | Operator::F32Max | Operator::F32Copysign => {
                    // [f32 f32] -> [f32]
                    let i = vec![WasmType::U32, WasmType::U32];
                    let o = vec![WasmType::U32];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::F64Abs | Operator::F64Neg | Operator::F64Ceil | Operator::F64Floor | Operator::F64Trunc | Operator::F64Nearest | Operator::F64Sqrt => {
                    // [f64] -> [f64]
                    let i = vec![WasmType::U64];
                    let o = vec![WasmType::U64];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::F64Add | Operator::F64Sub | Operator::F64Mul | Operator::F64Div | Operator::F64Min | Operator::F64Max | Operator::F64Copysign => {
                    // [f64 f64] -> [f64]
                    let i = vec![WasmType::U64, WasmType::U64];
                    let o = vec![WasmType::U64];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }

                Operator::I32WrapI64 => {
                    // [i64] -> [i32]
                    let i = vec![WasmType::U64];
                    let o = vec![WasmType::U32];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::I32TruncF32S | Operator::I32TruncF32U => {
                    // [f32] -> [i32]
                    let i = vec![WasmType::U32];
                    let o = vec![WasmType::U32];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::I32TruncF64S | Operator::I32TruncF64U => {
                    // [f64] -> [i32]
                    let i = vec![WasmType::U64];
                    let o = vec![WasmType::U32];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::I64ExtendI32S | Operator::I64ExtendI32U => {
                    // [i32] -> [i64]
                    let i = vec![WasmType::U32];
                    let o = vec![WasmType::U64];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::I64TruncF32S | Operator::I64TruncF32U => {
                    // [f32] -> [i64]
                    let i = vec![WasmType::U32];
                    let o = vec![WasmType::U64];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::I64TruncF64S | Operator::I64TruncF64U => {
                    // [f64] -> [i64]
                    let i = vec![WasmType::U64];
                    let o = vec![WasmType::U64];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::F32ConvertI32S | Operator::F32ConvertI32U => {
                    // [i32] -> [f32]
                    let i = vec![WasmType::U32];
                    let o = vec![WasmType::U32];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::F32ConvertI64S | Operator::F32ConvertI64U => {
                    // [i64] -> [f32]
                    let i = vec![WasmType::U64];
                    let o = vec![WasmType::U64];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::F32DemoteF64 => {
                    // [i64] -> [f32]
                    let i = vec![WasmType::U64];
                    let o = vec![WasmType::U32];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::F64ConvertI32S | Operator::F64ConvertI32U => {
                    // [i32] -> [f64]
                    let i = vec![WasmType::U32];
                    let o = vec![WasmType::U64];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::F64ConvertI64S | Operator::F64ConvertI64U => {
                    // [i64] -> [f64]
                    let i = vec![WasmType::U64];
                    let o = vec![WasmType::U64];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::F64PromoteF32 => {
                    // [f32] -> [f64]
                    let i = vec![WasmType::U32];
                    let o = vec![WasmType::U64];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::I32ReinterpretF32 | Operator::I64ReinterpretF64 | Operator::F32ReinterpretI32 | Operator::F64ReinterpretI64 => {
                    // [t] -> [t]
                    let i = vec![WasmType::Any];
                    let o = vec![WasmType::Any];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::I32Extend8S | Operator::I32Extend16S | Operator::I64Extend8S | Operator::I64Extend16S | Operator::I64Extend32S => {
                    // [t] -> [t]
                    let i = vec![WasmType::Any];
                    let o = vec![WasmType::Any];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                
                Operator::MemoryCopy{..} => {
                    // [i32 i32 i32] -> []
                    let i = vec![WasmType::U32, WasmType::U32, WasmType::U32];
                    let o = vec![];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }
                Operator::MemoryFill{..} => {
                    // [i32 i32 i32] -> []
                    let i = vec![WasmType::U32, WasmType::U32, WasmType::U32];
                    let o = vec![];

                    fast_bytecode.push(
                        Self::emit_label(codepos, Self::popf(&mut stack, i), Self::pushf(&mut stack, o))
                    );
                }

                ref _other => {
                    // println!("[WARN]: {:?}", op);
                    log::error!("Unsupprted operator");
                }
            }
        }

        return fast_bytecode;
    }
}

