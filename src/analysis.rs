use camino::Utf8PathBuf;
use byteorder::{LittleEndian, ByteOrder};

pub fn analysis_pc(path: Utf8PathBuf) {
    let buf: Vec<u8> = std::fs::read(&path).unwrap();
    println!("size: {}", buf.len());
    println!("fidx: {}", LittleEndian::read_u32(&buf));
    println!("offset: {}", LittleEndian::read_u32(&buf[4..]));
}