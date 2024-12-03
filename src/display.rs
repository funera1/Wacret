use crate::core::function::{BytecodeFunction, Function, CodePos};
use crate::core::module;
use crate::compile::{self, FastBytecodeFunction, FastCodePos, WasmType};
use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::fs::File;

use camino::Utf8PathBuf;
use anyhow::Result;
use csv::Terminator;

// Byte order mark
const BOM: &[u8; 3] = &[0xEF, 0xBB, 0xBF]; // UTF-8

#[derive(Clone)]
struct ExtFastCodePos<'a> {
    codepos: FastCodePos<'a>,
    type_stack: Vec<WasmType>,
}

#[derive(Clone)]
struct AbsCodePos<'a> {
    // NOTE: vecで持ってるからややこしいが、あくまでこれは、あるoffsetについての等価なコード位置のもの
    offset: u32,
    codepos: Vec<CodePos<'a>>,
    fast_codepos: Vec<ExtFastCodePos<'a>>,
}


pub fn main(path: Utf8PathBuf) -> Result<()> {
    // pathからwasmコードを取得
    let buf: Vec<u8> = std::fs::read(&path).unwrap();
    let m = module::new_module(&buf)?;
    log::debug!("function size is {}", m.funcs.len());

    // wasmコードの型スタックを計算する
    let funcs = m.parse()?;

    let mut merged_funcs: Vec<Vec<AbsCodePos>> = Vec::new();
    // 高速バイトコードを配列に詰める。このとき、高速バイトコードの各命令は、wasmコードの命令と等価な位置(index)へ格納する
    for func in funcs {
        match func {
            Function::ImportFunction(_) => continue,
            Function::BytecodeFunction(b)=> {
                // wasmコードから高速バイトコードを生成する
                let compiled_func = FastBytecodeFunction::new(&m, &b);

                // 型スタックを計算
                let ext_compiled_code = calc_type_stack(compiled_func);

                // funcとcompiled_funcで同じオフセット同士を紐付ける
                // NOTE: 一つのオフセットに対して複数の命令が入る場合がある
                let all_codes: Vec<AbsCodePos> = merge_codes(&b.codes, &ext_compiled_code);
                merged_funcs.push(all_codes);
            },
        }
    }

    // それぞれのバイトコードと型スタックをhuman-friendryな形式で表示する
    print_csv(merged_funcs);

    return Ok(());
}

fn calc_type_stack(func: FastBytecodeFunction<'_>) -> Vec<ExtFastCodePos> {
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

// TODO: clone多用しているが、そのcloneが適切か考え直す
fn merge_codes<'a>(codes: &Vec<CodePos<'a>>, compiled_codes: &Vec<ExtFastCodePos<'a>>) -> Vec<AbsCodePos<'a>> {
    // codesの前処理
    let mut m1: HashMap<u32, Vec<CodePos>> = HashMap::new();
    let mut v1 = Vec::new();
    let mut now_offset = 0;
    for codepos in codes {
        if now_offset != codepos.offset {
            m1.insert(now_offset, v1);
            v1 = Vec::new();
            now_offset = codepos.offset;
        }
        v1.push(codepos.clone());
    }

    // compiled_codesの前処理
    let mut m2: HashMap<u32, Vec<ExtFastCodePos>> = HashMap::new();
    let mut v2: Vec<ExtFastCodePos> = Vec::new();
    now_offset = 0;
    for ext in compiled_codes {
        if now_offset != ext.codepos.offset {
            m2.insert(now_offset, v2);
            v2 = Vec::new();
            now_offset = ext.codepos.offset;
        }
        v2.push(ext.clone());
    }

    // Vec<AbsCodePos>の構築
    let mut all_codes = Vec::new();
    for codepos in codes {
        let offset = codepos.offset;
        let codepos = m1.get(&offset).expect("not found m1.get(offset)");
        let fast_codepos = m2.get(&offset).expect("not found m2.get(offset)");

        let e = AbsCodePos{
            offset: offset,
            codepos: codepos.clone(),
            fast_codepos: fast_codepos.clone(),
        };
        all_codes.push(e);
    }

   return all_codes; 
}

fn print_csv(funcs: Vec<Vec<AbsCodePos>>) {
    // create file
    let file = File::create("wasm.csv").expect("Failed to create file");
    let mut w = BufWriter::new(file);
    let _ = w.write_all(BOM);

    let mut w= csv::WriterBuilder::new()
                                .terminator(Terminator::CRLF)
                                .from_writer(w);
    
    // print header
    let _ = w.write_record(["standard code", "standard stack", "fast stack", "fast code"]);

    // print content
    for func in funcs {
        for code in func {
            // let standard_code = code.codepos[0].opcode;
            let standard_stack = std::str::from_utf8(&code.codepos[0].type_stack)
                                            .expect("Failed to convert standard stack to string");
            // let fast_code = code.fast_codepos[0].codepos.opcode;
            let fast_stack = &code.fast_codepos[0].type_stack
                                                .iter()
                                                .map(|e| e.to_string())
                                                .collect::<Vec<_>>()
                                                .join(", ");
            let _ = w.write_record(["hoge", standard_stack, fast_stack, "hoge"]).expect("Failed to write csv");
        }
    }
}