use crate::core::function::{BytecodeFunction, Function, CodePos};
use crate::core::module;
use crate::compile::compile::{WasmType, u8_to_wasmtype, CompiledBytecodeFunction, CompiledCodePos};

use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::fs::File;

use camino::Utf8PathBuf;
use anyhow::Result;
use csv::Terminator;

// Byte order mark
const BOM: &[u8; 3] = &[0xEF, 0xBB, 0xBF]; // UTF-8

#[derive(Clone)]
struct ExtCompiledCodePos<'a> {
    codepos: CompiledCodePos<'a>,
    position_stack: Vec<u32>,
    type_stack: Vec<WasmType>,
}

#[derive(Clone)]
struct AbsCodePos<'a> {
    // NOTE: vecで持ってるからややこしいが、あくまでこれは、あるoffsetについての等価なコード位置のもの
    offset: u32,
    codepos: Vec<CodePos<'a>>,
    fast_codepos: Vec<ExtCompiledCodePos<'a>>,
}


impl<'a> AbsCodePos<'a> {
    fn to_string_standard_code(&self) -> Vec<String> {
        let codepos = &self.codepos;

        let mut result = vec![];
        for code in codepos {
            result.push(code.opcode.to_string());
        }

        return result;
    }

    fn to_string_fast_code(&self) -> Vec<String> {
        let codepos = &self.fast_codepos;

        let mut result = vec![];
        for code in codepos {
            result.push(code.codepos.opcode.to_string());
        }

        return result;
    }

    fn to_string_standard_stack(&self) -> Vec<String> {
        let codepos = &self.codepos;

        let mut result = vec![];
        for code in codepos {
            // TODO: 実装が冗長なので治す
            let standard_stack = code.type_stack
                                            .iter()
                                            .map(|e| u8_to_wasmtype(*e))
                                            .collect::<Vec<_>>();

            let standard_stack = standard_stack
                                        .iter()
                                        .map(|e| e.to_string())
                                        .collect::<Vec<_>>()
                                        .join(", ");
            result.push(format!("[{}]", standard_stack));
        }

        return result;
    }

    fn to_string_fast_position_stack(&self) -> Vec<String> {
        let fast_codepos = &self.fast_codepos;

        let mut result = vec![];
        for code in fast_codepos {
            let fast_stack = code.position_stack
                                        .iter()
                                        .map(|e| e.to_string())
                                        .collect::<Vec<_>>()
                                        .join(", ");
            result.push(format!("[{}]", fast_stack));
        }
        return result;
    }

    fn to_string_fast_type_stack(&self) -> Vec<String> {
        let fast_codepos = &self.fast_codepos;

        let mut result = vec![];
        for code in fast_codepos {
            let fast_stack = code.type_stack
                                        .iter()
                                        .map(|e| e.to_string())
                                        .collect::<Vec<_>>()
                                        .join(", ");
            result.push(format!("[{}]", fast_stack));
        }
        return result;
    }
}


pub fn main(path: Utf8PathBuf) -> Result<()> {
    // pathからwasmコードを取得
    let buf: Vec<u8> = std::fs::read(&path).unwrap();
    let m = module::new_module(&buf)?;
    log::debug!("function size is {}", m.funcs.len());

    // wasmコードの型スタックを計算する
    let funcs = m.parse()?;

    // TODO: import_func_lenを使うの実装が微妙なのでリファクタする
    let mut import_func_len = 0;
    let mut merged_funcs: Vec<Vec<AbsCodePos>> = Vec::new();
    // 高速バイトコードを配列に詰める。このとき、高速バイトコードの各命令は、wasmコードの命令と等価な位置(index)へ格納する
    for func in funcs {
        match func {
            Function::ImportFunction(_) => import_func_len += 1,
            Function::BytecodeFunction(b)=> {
                // wasmコードから高速バイトコードを生成する
                let standard_func = CompiledBytecodeFunction::new_standard_code(&m, &b);
                let fast_func = CompiledBytecodeFunction::new_fast_code(&m, &b);

                // 型スタックを計算
                let ext_standard_code = ext_compiled_bytecode(standard_func);
                let ext_fast_code = ext_compiled_bytecode(fast_func);

                // funcとcompiled_funcで同じオフセット同士を紐付ける
                // NOTE: 一つのオフセットに対して複数の命令が入る場合がある
                let all_codes: Vec<AbsCodePos> = merge_codes(&b.codes, &ext_standard_code);
                merged_funcs.push(all_codes);
            },
        }
    }

    // それぞれのバイトコードと型スタックをhuman-friendryな形式で表示する
    print_csv(import_func_len, merged_funcs);

    return Ok(());
}

fn ext_compiled_bytecode(func: CompiledBytecodeFunction<'_>) -> Vec<ExtCompiledCodePos> {
    let mut codes = Vec::new();

    let mut type_stack = Vec::new();
    let mut position_stack = Vec::new();
    for codepos in func.codes {
        let optype = &codepos.optype;
        // pop
        for _ in 0..optype.input.len() {
            position_stack.pop();
            type_stack.pop();
        }
        // push
        for i in 0..optype.output.len() {
            position_stack.push(codepos.offset);
            type_stack.push(optype.output[i]);
        }

        codes.push(
            ExtCompiledCodePos{
                codepos: codepos, 
                position_stack: position_stack.clone(),
                type_stack: type_stack.clone(),
        });
    }
    return codes;
}

// TODO: clone多用しているが、そのcloneが適切か考え直す
fn merge_codes<'a>(codes: &Vec<CodePos<'a>>, compiled_codes: &Vec<ExtCompiledCodePos<'a>>) -> Vec<AbsCodePos<'a>> {
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
    let mut m2: HashMap<u32, Vec<ExtCompiledCodePos>> = HashMap::new();
    let mut v2: Vec<ExtCompiledCodePos> = Vec::new();
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

        // codepos取得
        let codepos;
        match m1.get(&offset) {
            None => codepos = Vec::new(),
            Some(content) => codepos = content.clone(),
        }

        // fast_codepos取得
        let fast_codepos;
        match m2.get(&offset) {
            None => fast_codepos = Vec::new(),
            Some(content) => fast_codepos = content.clone(),
        }

        let e = AbsCodePos{
            offset: offset,
            codepos: codepos,
            fast_codepos: fast_codepos,
        };
        all_codes.push(e);
    }

   return all_codes; 
}

fn print_csv(import_func_len: u32, funcs: Vec<Vec<AbsCodePos>>) {
    // create file
    let file = File::create("wasm.csv").expect("Failed to create file");
    let mut w = BufWriter::new(file);
    let _ = w.write_all(BOM);

    let mut w= csv::WriterBuilder::new()
                                .terminator(Terminator::CRLF)
                                .from_writer(w);
    
    // print header
    // TODO:　ハードコードはバグの要因なのでリファクタする
    let _ = w.write_record(["fidx", "offset", "standard code", "standard stack", "fast code", "fast stack"]);

    // print content
    for (fidx, func) in funcs.iter().enumerate() {
        for code in func {
            // TODO: vecの管理をどうにかする
            let mut contents = vec![["", "", "", "", "", ""]; std::cmp::max(code.codepos.len(), code.fast_codepos.len())];
            if contents.len() == 0 {
                continue;
            }
            let offset_str = &(code.offset.to_string());
            let fidx = import_func_len + fidx as u32;
            let fidx_str = fidx.to_string();
            contents[0][0] = &fidx_str;
            contents[0][1] = &offset_str;

            // standard
            let standard_codes = code.to_string_standard_code();
            let standard_stacks = code.to_string_standard_stack();
            for (i, code) in code.codepos.iter().enumerate() {
                contents[i][2] = standard_codes[i].as_str();
                contents[i][3] = standard_stacks[i].as_str();
            }

            // fast
            let fast_codes = code.to_string_fast_code();
            let fast_stacks = code.to_string_fast_position_stack();
            for (i, code) in code.fast_codepos.iter().enumerate() {
                contents[i][4] = fast_codes[i].as_str();
                contents[i][5] = fast_stacks[i].as_str();
            }

            // offsetを出力

            // 
            for c in contents {
                let _ = w.write_record(c).expect("Failed to write csv");
            }
        }
    }
}