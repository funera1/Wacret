use crate::core::function::{BytecodeFunction, Function, CodePos};
use crate::core::module;
use crate::compile::{self, FastBytecodeFunction, FastCodePos, WasmType};
use std::collections::HashMap;

use camino::Utf8PathBuf;
use anyhow::Result;

struct ExtFastCodePos<'a> {
    codepos: FastCodePos<'a>,
    type_stack: Vec<WasmType>,
}

struct EqOps<'a> {
    b: &'a Vec<&'a CodePos<'a>>,
    f: &'a Vec<&'a ExtFastCodePos<'a>>,
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
                let ext_compiled_code = calc_type_stack(compiled_func);

                // funcとcompiled_funcで同じオフセット同士を紐付ける
                // NOTE: 一つのオフセットに対して複数の命令が入る場合がある
                let all_codes: Vec<EqOps>  = correspond_semantic_equations(&b.codes, &ext_compiled_code);
                
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

pub fn correspond_semantic_equations<'a>(codes: &'a Vec<CodePos<'a>>, compiled_codes: &'a Vec<ExtFastCodePos<'a>>) -> Vec<EqOps<'a>> {
    // codesの前処理
    let mut m1: HashMap<u32, Vec<&CodePos>> = HashMap::new();
    let mut v1 = Vec::new();
    let mut now_offset = 0;
    for codepos in codes {
        if now_offset != codepos.offset {
            m1.insert(now_offset, v1);
            v1 = Vec::new();
            now_offset = codepos.offset;
        }
        v1.push(codepos);
    }

    // compiled_codesの前処理
    let mut m2: HashMap<u32, Vec<&ExtFastCodePos>> = HashMap::new();
    let mut v2: Vec<&ExtFastCodePos> = Vec::new();
    now_offset = 0;
    for ext in compiled_codes {
        if now_offset != ext.codepos.offset {
            m2.insert(now_offset, v2);
            v2 = Vec::new();
            now_offset = ext.codepos.offset;
        }
        v2.push(ext);
    }

    // Vec<EqOps>の構築
    let mut all_codes = Vec::new();
    for codepos in codes {
        let offset = codepos.offset;
        let e = EqOps{
            b: m1.get(&offset).expect("not found m1.get(offset)"),
            f: m2.get(&offset).expect("not found m2.get(offset)"),
        };
        all_codes.push(e);
    }

   return all_codes; 
}