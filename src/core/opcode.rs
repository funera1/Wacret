use wasmparser::Operator;
use crate::core::val::WasmType;

pub struct OpInfo<'a> {
    pub input: Vec<WasmType>,
    pub output: Vec<WasmType>,
    pub opcode: &'a Operator<'a>,
}

pub fn opcode_info<'a>(op: &'a Operator) -> OpInfo<'a> {
    match op {
        Operator::Unreachable => {
            return OpInfo {
                input: vec![],
                output: vec![],
                opcode: op,
            };
        }
        Operator::Nop => {
            // skip_label
            return OpInfo {
                input: vec![],
                output: vec![],
                opcode: op,
            };
        }
        Operator::Block{ .. } => {
            // skip_label
            // TODO: COPY_STACKをemitする
            return OpInfo {
                input: vec![],
                output: vec![],
                opcode: op,
            };
        }
        Operator::Loop{ .. } => {
            // skip_label
            return OpInfo {
                input: vec![],
                output: vec![],
                opcode: op,
            };
        }
        Operator::If{ .. } => {
            return OpInfo {
                input: vec![],
                output: vec![],
                opcode: op,
            };
        }
        Operator::Else{ .. } => {
            return OpInfo {
                input: vec![],
                output: vec![],
                opcode: op,
            };
        }
        Operator::End{ .. } => {
            // TODO: 挙動をちゃんと調べる
            let i: Vec<WasmType> = vec![];
            let o: Vec<WasmType> = vec![];

            return OpInfo {
                input: vec![],
                output: vec![],
                opcode: op,
            };
        }
        Operator::Br{..} => {
            // [t1*, t*] -> [t2*]
            // NOTE: この位置では型スタックは変化しない
            let i: Vec<WasmType> = vec![];
            let o: Vec<WasmType> = vec![];

            return OpInfo {
                input: vec![],
                output: vec![],
                opcode: op,
            };
        }
        Operator::BrIf{..} => {
            // [t1*, I32] -> [t2*]
            return OpInfo {
                input: vec![WasmType::I32],
                output: vec![],
                opcode: op,
            };
        }
        Operator::BrTable{..} => {
            // [t1*, t*, I32] -> [t2*]
            return OpInfo {
                input: vec![WasmType::I32],
                output: vec![],
                opcode: op,
            };
        }
        Operator::Return{ .. } => {
            // TODO: ほんとにbreakで良いのか確認
            return OpInfo {
                input: vec![],
                output: vec![],
                opcode: op,
            };
        }
        Operator::Call{ function_index } => {
            // [Args*] -> [Rets*]
            let f = module.get_type_by_func(function_index);
            let params = f.params();
            let results = f.results();

            let i: Vec<WasmType> = params.iter().map(valtype_to_wasmtype).collect();
            let o: Vec<WasmType> = results.iter().map(valtype_to_wasmtype).collect();

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::CallIndirect{ type_index , .. } => {
            // [Args*, U32] -> [Rets*]
            let f = module.get_type_by_type(type_index);
            let params = f.params();
            let results = f.results();

            let i: Vec<WasmType> = params.iter().map(valtype_to_wasmtype)
                                         .chain(std::iter::once(WasmType::U32)).collect();
            let o: Vec<WasmType> = results.iter().map(valtype_to_wasmtype).collect();

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::Drop{ .. } => {
            // [Any] -> []
            // skip_label
            // let _ = stack.pop().expect("[ERROR] stack is empty");
            
            // TODO: skip_labelにする（ただし、stack.popするだけだとcalc_stackのときに反映されないのでどうするか考える)
            let i = vec![WasmType::Any];
            let o = vec![];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::Select{ .. } => {
            // [Any, Any, U32] -> [Any]
            let i = vec![WasmType::Any, WasmType::Any, WasmType::U32];
            let o = vec![WasmType::Any];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::TypedSelect{ .. } => {
            // [Any, Any, U32] -> [Any]
            let i = vec![WasmType::Any, WasmType::Any, WasmType::U32];
            let o = vec![WasmType::Any];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }

        Operator::LocalGet{ local_index } => {
            // [] -> [Any]
            // skip_label
            // local_indexの型をstackにpushする
            let local_type = module.get_type_by_local(local_index);
            return OpInfo {
                input: vec![],
                output: vec![local_type],
                opcode: op,
            };
        }
        Operator::LocalSet{ .. } => {
            // [Any] -> []
            // NOTE: 理解しやすくするために簡単にしている。
            // NOTE: preserveのためにCOPY命令が挿入されたり、LOCAL_SET命令が喪失したりするが一旦考慮しない
            return OpInfo {
                input: vec![WasmType::Any],
                output: vec![],
                opcode: op,
            };
        }
        Operator::LocalTee{ .. } => {
            // [Any] -> [Any]
            // TODO: local.getをskipすることで発生するごにょごにょを処理しないといけない
            let i = vec![WasmType::Any];
            let o = vec![WasmType::Any];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::GlobalGet{ global_index } => {
            // [] -> [Any]
            let i = vec![];
            let o = vec![WasmType::Any];

            let global_type = module.get_type_by_global(global_index);
            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::GlobalSet{ .. } => {
            // [Any] -> []
            let i = vec![WasmType::Any];
            let o = vec![];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::TableGet{ .. } => {
            unimplemented!("Not supported yet");
            // [U32] -> [Any]
            let i = vec![WasmType::I32];
            let o = vec![WasmType::Any];
        }
        Operator::TableSet{ .. } => {
            // [U32, Any] -> []
            let i = vec![WasmType::I32, WasmType::Any];
            let o = vec![];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }

        Operator::I32Load{ .. } | 
        Operator::I32Load8S{ .. } | Operator::I32Load8U{ .. } | 
        Operator::I32Load16S{ .. } | Operator::I32Load16U{ .. } => {
            // [I32] -> [I32]
            let i = vec![WasmType::I32];
            let o = vec![WasmType::I32];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::F32Load { .. } => {
            // [I32] -> [F32]
            let i = vec![WasmType::I32];
            let o = vec![WasmType::F32];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::I64Load{ .. } |
        Operator::I64Load8S{ .. } | Operator::I64Load8U{ .. } | 
        Operator::I64Load16S{ .. } | Operator::I64Load16U{ .. } | 
        Operator::I64Load32S{ .. } | Operator::I64Load32U{ .. } => {
            // [I32] -> [I64]
            let i = vec![WasmType::I32];
            let o = vec![WasmType::I64];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::F64Load { .. } => {
            // [I32] -> [F64]
            let i = vec![WasmType::I32];
            let o = vec![WasmType::F64];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }

        Operator::I32Store{ .. } | Operator::I64Store{ .. } | Operator::F32Store{ .. } | Operator::F64Store{ .. } |
        Operator::I32Store8{ .. } | Operator::I32Store16 { .. } | 
        Operator::I64Store8{ .. } | Operator::I64Store16 { .. } | Operator::I64Store32{ .. } => {
            // [U32, U32] -> []
            let i = vec![WasmType::Any, WasmType::Any];
            let o = vec![];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::MemorySize{ .. } => {
            // [] -> [I32]
            let i = vec![];
            let o = vec![WasmType::I32];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::MemoryGrow{ .. } => {
            // [U32] -> [U32]
            let i = vec![WasmType::I32];
            let o = vec![WasmType::I32];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }

        Operator::I32Const{ .. } => {
            return OpInfo {
                input: vec![],
                output: vec![WasmType::I32],
                opcode: op,
            };
        }
        Operator::I64Const { .. } => {
            return OpInfo {
                input: vec![],
                output: vec![WasmType::I64],
                opcode: op,
            };
        }
        Operator::F32Const { .. } => {
            return OpInfo {
                input: vec![],
                output: vec![WasmType::F32],
                opcode: op,
            };
        }
        Operator::F64Const { .. } => {
            return OpInfo {
                input: vec![],
                output: vec![WasmType::F64],
                opcode: op,
            };
        }
        Operator::I32Eqz{ .. } => {
            // [U32] -> [U32]
            let i = vec![WasmType::I32];
            let o = vec![WasmType::I32];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::I32Eq | Operator::I32Ne | Operator::I32LtS | Operator::I32LtU | Operator::I32GtS | Operator::I32GtU
        | Operator::I32LeS | Operator::I32LeU | Operator::I32GeS | Operator::I32GeU 
        | Operator::F32Eq | Operator::F32Ne | Operator::F32Lt | Operator::F32Gt | Operator::F32Le | Operator::F32Ge => {
            // [U32, U32] -> [U32]
            let i = vec![WasmType::I32, WasmType::I32];
            let o = vec![WasmType::I32];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::I64Eqz{ .. } => {
            // [U64] -> [U32]
            let i = vec![WasmType::I64];
            let o = vec![WasmType::I32];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::I64Eq | Operator::I64Ne | Operator::I64LtS | Operator::I64LtU | Operator::I64GtS | Operator::I64GtU
        | Operator::I64LeS | Operator::I64LeU | Operator::I64GeS | Operator::I64GeU
        | Operator::F64Eq | Operator::F64Ne | Operator::F64Lt | Operator::F64Gt | Operator::F64Le | Operator::F64Ge => {
            // [U64, U64] -> [U32]
            let i = vec![WasmType::I64, WasmType::I64];
            let o = vec![WasmType::I32];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::I32Clz | Operator::I32Ctz | Operator::I32Popcnt => {
            // [U32] -> [U32]
            let i = vec![WasmType::I32];
            let o = vec![WasmType::I32];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::I32Add | Operator::I32Sub | Operator::I32Mul | Operator::I32DivS | 
        Operator::I32DivU | Operator::I32RemS | Operator::I32RemU |
        Operator::I32And | Operator::I32Or | Operator::I32Xor | 
        Operator::I32Shl | Operator::I32ShrS | Operator::I32ShrU | 
        Operator::I32Rotl | Operator::I32Rotr => {
            // [U32, U32] -> [U32]
            let i = vec![WasmType::I32, WasmType::I32];
            let o = vec![WasmType::I32];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::I64Clz | Operator::I64Ctz | Operator::I64Popcnt => {
            // [U64] -> [U64]
            let i = vec![WasmType::I64];
            let o = vec![WasmType::I64];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::I64Add | Operator::I64Sub | Operator::I64Mul | 
        Operator::I64DivS | Operator::I64DivU | 
        Operator::I64RemS | Operator::I64RemU |
        Operator::I64And | Operator::I64Or | Operator::I64Xor | 
        Operator::I64Shl | Operator::I64ShrS | Operator::I64ShrU | 
        Operator::I64Rotl | Operator::I64Rotr => {
            // [U64, U64] -> [U64]
            let i = vec![WasmType::I64, WasmType::I64];
            let o = vec![WasmType::I64];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }

        Operator::F32Abs | Operator::F32Neg | 
        Operator::F32Ceil | Operator::F32Floor | 
        Operator::F32Trunc | Operator::F32Nearest | Operator::F32Sqrt => {
            // [F32] -> [F32]
            let i = vec![WasmType::F32];
            let o = vec![WasmType::F32];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::F32Add | Operator::F32Sub | Operator::F32Mul | Operator::F32Div | 
        Operator::F32Min | Operator::F32Max | Operator::F32Copysign => {
            // [f32 f32] -> [f32]
            let i = vec![WasmType::F32, WasmType::F32];
            let o = vec![WasmType::F32];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::F64Abs | Operator::F64Neg | 
        Operator::F64Ceil | Operator::F64Floor | 
        Operator::F64Trunc | Operator::F64Nearest | Operator::F64Sqrt => {
            // [f64] -> [f64]
            let i = vec![WasmType::F64];
            let o = vec![WasmType::F64];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::F64Add | Operator::F64Sub | Operator::F64Mul | Operator::F64Div | 
        Operator::F64Min | Operator::F64Max | Operator::F64Copysign => {
            // [f64 f64] -> [f64]
            let i = vec![WasmType::F64, WasmType::F64];
            let o = vec![WasmType::F64];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }

        Operator::I32WrapI64 => {
            // [i64] -> [i32]
            let i = vec![WasmType::I64];
            let o = vec![WasmType::I32];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::I32TruncF32S | Operator::I32TruncF32U => {
            // [f32] -> [i32]
            let i = vec![WasmType::F32];
            let o = vec![WasmType::I32];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::I32TruncF64S | Operator::I32TruncF64U => {
            // [f64] -> [i32]
            let i = vec![WasmType::F64];
            let o = vec![WasmType::I32];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::I64ExtendI32S | Operator::I64ExtendI32U => {
            // [i32] -> [i64]
            let i = vec![WasmType::I32];
            let o = vec![WasmType::I64];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::I64TruncF32S | Operator::I64TruncF32U => {
            // [f32] -> [i64]
            let i = vec![WasmType::F32];
            let o = vec![WasmType::I64];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::I64TruncF64S | Operator::I64TruncF64U => {
            // [f64] -> [i64]
            let i = vec![WasmType::F64];
            let o = vec![WasmType::I64];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::F32ConvertI32S | Operator::F32ConvertI32U => {
            // [i32] -> [f32]
            let i = vec![WasmType::I32];
            let o = vec![WasmType::F32];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::F32ConvertI64S | Operator::F32ConvertI64U => {
            // [i64] -> [f32]
            let i = vec![WasmType::I64];
            let o = vec![WasmType::F64];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::F32DemoteF64 => {
            // [i64] -> [f32]
            let i = vec![WasmType::I64];
            let o = vec![WasmType::F32];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::F64ConvertI32S | Operator::F64ConvertI32U => {
            // [i32] -> [f64]
            let i = vec![WasmType::I32];
            let o = vec![WasmType::F64];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::F64ConvertI64S | Operator::F64ConvertI64U => {
            // [i64] -> [f64]
            let i = vec![WasmType::I64];
            let o = vec![WasmType::F64];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::F64PromoteF32 => {
            // [f32] -> [f64]
            let i = vec![WasmType::F32];
            let o = vec![WasmType::F64];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::I32ReinterpretF32 => {
            // [f32] -> [i32]
            let i = vec![WasmType::F32];
            let o = vec![WasmType::I32];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::I64ReinterpretF64 => {
            // [f64] -> [i64]
            let i = vec![WasmType::F64];
            let o = vec![WasmType::I64];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::F32ReinterpretI32 => {
            // [i32] -> [f32]
            let i = vec![WasmType::I32];
            let o = vec![WasmType::F32];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::F64ReinterpretI64 => {
            // [i64] -> [f64]
            let i = vec![WasmType::I64];
            let o = vec![WasmType::F64];

            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::I32Extend8S | Operator::I32Extend16S => {
            // [i32] -> [i32]
            let i = vec![WasmType::I32];
            let o = vec![WasmType::I32];
            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::I64Extend8S | Operator::I64Extend16S | Operator::I64Extend32S => {
            // [i64] -> [i64]
            let i = vec![WasmType::I64];
            let o = vec![WasmType::I64];
            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        Operator::MemoryCopy{..} | Operator::MemoryFill { .. } => {
            // [i32 i32 i32] -> []
            let i = vec![WasmType::I32, WasmType::I32, WasmType::I32];
            let o = vec![];
            return OpInfo {
                input: i,
                output: o,
                opcode: op,
            };
        }
        ref _other => {
            unimplemented!("Unsupported operator: {:?}", op);
        }
    }
}