use crate::core::function::{BytecodeFunction, Function};
use crate::core::module;

use camino::Utf8PathBuf;
use wasmparser::Operator;

use std::fs::File;
use anyhow::Result;

const BYTE_U8: u32 = 1;
const BYTE_U32: u32 = 4;
const BYTE_U64: u32 = 8;

pub fn create_table(path: Utf8PathBuf) -> Result<()> {
    let buf: Vec<u8> = std::fs::read(&path).unwrap();

    // コードから各セクションの情報を抽出
    let m = module::new_module(&buf)?;
    log::debug!("function size is {}", m.funcs.len());

    // 型スタックを生成
    let funcs = m.parse()?;


    // 型スタックから型スタックテーブルを生成する
    let (tablemap_func, tablemap_offset) = calc_tablemap(&funcs);
    
    // tablemapをもとにファイルに書き込む
    let _ = write_type_stack_table(&funcs, "type_table");
    let _ = write_tablemap_func(&tablemap_func, "tablemap_func");
    let _ = write_tablemap_offset(&tablemap_offset, &funcs, "tablemap_offset");

    Ok(())
}

// TODO: テスト書く
pub fn calc_tablemap(funcs: &Vec<Function>) -> (Vec<u32>, Vec<Vec<u32>>) {
    let mut tablemap_func: Vec<u32> = vec![];
    let mut tablemap_offset: Vec<Vec<u32>> = vec![];

    let mut tablemap_offset_addr = 0;

    for func in funcs {
        match func {
            Function::ImportFunction(_) => {
                tablemap_func.push(0);
            }
            Function::BytecodeFunction(f) => {
                tablemap_func.push(tablemap_offset_addr);
                tablemap_offset_addr += calc_tablefunc(&f);

                tablemap_offset.push(calc_tableoffset(&f));
            }
        }
    }
    return (tablemap_func, tablemap_offset);
}

fn calc_tablefunc(func: &BytecodeFunction) -> u32 {
    // "tablemap_offset format"
    // 関数fについて
    //  - fのローカルの長さ(u32)
    //  - fのローカル (local.len * u8)
    //  - 各コード位置について
    //      - offset  (u32)
    //      - address (u64)
    let local_len = func.locals.len() as u32;
    let codes_len = func.codes.len() as u32 + 1; // codeの一番初め、offset=0のときを考慮するために+1
    log::debug!("local: {}, codes: {}", local_len, codes_len);
    return BYTE_U32 + (local_len * BYTE_U8) + (codes_len * (BYTE_U32 + BYTE_U64));
}

fn calc_tableoffset(func: &BytecodeFunction) -> Vec<u32> {
    let last = func.codes.last().expect("codes last");
    let mut offset_to_codepos: Vec<u32> = vec![0; last.offset as usize + 1];
    // TODO: ここのaddrは、関数を超えて状態が引き継がれるはずなので、このままの実装だとだめ
    let mut addr = 0 as u32;

    for codepos in &func.codes {
        // "type_stack_table format"
        // 各コード位置について
        //  - 型スタックの長さ (u32)
        //  - 型スタックの中身 (stack.len * u8)
        // 但し、OpcodeがCallの場合は、「呼び出し途中」と「呼び出し後」の2パターン書く
        let opcode = &codepos.opcode;
        let len  = codepos.type_stack.len() as u32;

        if let Operator::Call{..} = opcode {
            addr += BYTE_U32 + (len - codepos.callee_return_size) * BYTE_U8;
        }
        addr += BYTE_U32 + len * BYTE_U8;

        offset_to_codepos[codepos.offset as usize] = addr;
    }
    return offset_to_codepos;
}

pub fn write_type_stack_table(funcs: &Vec<Function>, filename: &str) -> Result<()> {
    let f: File = File::create(filename)?;

    // TODO: codepos周りの命名がキモいので整理する
    for function in funcs {
        match function {
            Function::ImportFunction(_) => {
            }
            Function::BytecodeFunction(func) => {
                for codepos in &func.codes {
                    // let _ = write_type_stack(&mut type_stack_table, &codepos)?;
                    let _ = io::write_u32(&f, codepos.type_stack.len() as u32);
                    let _ = io::write_u8s(&f, &codepos.type_stack);
                }
            }
        }
    } 

    return Ok(());
}

pub fn write_tablemap_func(tablemap_func: &Vec<u32>, filename: &str) -> Result<()> {
    let f: File = File::create(filename)?;

    let mut fidx = 0;
    for addr in tablemap_func {
        // TODO: この実装だとimport functionのfidxが考慮されない
        let _ = io::write_u32(&f, fidx)?;
        let _ = io::write_u64(&f, *addr as u64)?;
        fidx += 1;
    } 

    return Ok(());
}

// "tablemap_offset format"
// 関数fについて
//  - fのローカルの長さ(u32)
//  - fのローカル (local.len * u8)
//  - 各コード位置について
//      - offset  (u32)
//      - address (u64)
pub fn write_tablemap_offset(tablemap_offset: &Vec<Vec<u32>>, funcs: &Vec<Function>, filename: &str) -> Result<()> {
    let f: File = File::create(filename)?;

    let mut fidx = 0;
    for function in funcs {
        match function {
            Function::ImportFunction(_) => {

            }
            Function::BytecodeFunction(func) => {
                let locals = &func.locals;
                let _ = io::write_u32(&f, locals.len() as u32)?;
                let _ = io::write_u8s(&f, &locals)?;
                for c in &func.codes {
                    let _ = io::write_u32(&f, c.offset)?;
                    let _ = io::write_u64(&f, tablemap_offset[fidx][c.offset as usize] as u64)?;
                }
                fidx += 1;
            }
        }
    }
    return Ok(());
}

mod io {
    use std::io::{Write, Error};
    use std::fs::File;
    use byteorder::{LittleEndian, ByteOrder};

    // プリミティブ
    pub fn write_u8(mut file: &File, n: u8) -> Result<usize, Error> {
        let buf = [n; 1];
        return file.write(&buf);
    }

    pub fn write_u32(mut file: &File, n: u32) -> Result<usize, Error> {
        let mut buf = [0; 4];
        LittleEndian::write_u32(&mut buf, n);
        // let e = file.write(&buf);
        return file.write(&buf);
    }

    pub fn write_u64(mut file: &File, n: u64) -> std::io::Result<usize> {
        let mut buf = [0; 8];
        LittleEndian::write_u64(&mut buf, n);
        return file.write(&buf);
    }

    // 配列
    pub fn write_u8s(file: &File, v: &[u8]) -> Result<usize, Error> {
        for vi in v {
            let _ = write_u8(file, *vi)?;
        }
        return Ok(0);
    }
}