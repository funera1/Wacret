use crate::core::function::{BytecodeFunction, Function, CodePos};
use crate::core::module;
use crate::compile::{self, FastBytecodeFunction, FastCodePos, WasmType};
use std::collections::HashMap;

use camino::Utf8PathBuf;
use anyhow::Result;

// TODO: CodePosとFastCodePosを統合できれば2つも構造体いらん
struct ExtCodePos<'a> {
    codepos: CodePos<'a>,
    type_stack: Vec<WasmType>,
}

struct ExtFastCodePos<'a> {
    codepos: FastCodePos<'a>,
    type_stack: Vec<WasmType>,
}

struct EqOps<'a> {
    b: Vec<ExtCodePos<'a>>,
    f: Vec<ExtFastCodePos<'a>>,
}

pub fn main(path: Utf8PathBuf) -> Result<()> {
    // pathからwasmコードを取得
    let buf: Vec<u8> = std::fs::read(&path).unwrap();
    let m = module::new_module(&buf)?;
    log::debug!("function size is {}", m.funcs.len());

    // wasmコードの型スタックを計算する
    let funcs = m.parse()?;


    let mut compiled_funcs: Vec<FastBytecodeFunction<'_>> = Vec::new();
    // 高速バイトコードを配列に詰める。このとき、高速バイトコードの各命令は、wasmコードの命令と等価な位置(index)へ格納する
    for func in funcs {
        match func {
            Function::ImportFunction(_) => continue,
            Function::BytecodeFunction(b)=> {
                // wasmコードから高速バイトコードを生成する
                let compiled_func = compile::compile_fast_bytecode_function(&m, &b).expect("Failed to compile funcs");

                // 型スタックを計算
                let fast_type_stack = calc_type_stack(compiled_func);

                // funcとcompiled_funcで同じオフセット同士を紐付ける
                // NOTE: 一つのオフセットに対して複数の命令が入る場合がある
                let map: HashMap<u32, EqOps>  = correspond_semantic_equations(b, compiled_func);
                
                // let fast_bytecode = FastBytecodeFunction::compile_fast_bytecode(module, b);
                // compiled_funcs.push(FastBytecodeFunction::new(b.locals.clone(), fast_bytecode));
            },
        }
    }

    // それぞれのバイトコードと型スタックをhuman-friendryな形式で表示する
    return Ok(());
}

pub fn calc_type_stack(func: FastBytecodeFunction<'_>) -> Vec<ExtFastCodePos> {
    let mut codes = Vec::new();

    let mut type_stack = Vec::new();
    for codepos in func.codes {
        let optype = &codepos.optype;
        // pop
        for _ in 0..optype.input.len() {
            type_stack.pop();
        }
        // push
        for i in 0..optype.output.len() {
            type_stack.push(optype.output[i]);
        }

        codes.push(
            ExtFastCodePos{
                codepos: codepos, 
                type_stack: type_stack.clone(),
        });
    }
    return codes;
}

pub fn correspond_semantic_equations(funcs: BytecodeFunction, compiled_funcs: FastBytecodeFunction) -> HashMap<u32, EqOps> {
    let mut m1: HashMap<u32, Vec<ExtCodePos>> = HashMap::new();
    for codepos in funcs.codes {

        m1[&codepos.offset].push()
    }

   return HashMap::new(); 
}