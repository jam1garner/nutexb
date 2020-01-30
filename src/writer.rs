use std::io::{self, prelude::*};
use binwrite::BinWrite;

#[derive(BinWrite)]
struct NutexbFile {
    data: Vec<u8>,
}

pub fn write_nutexb<W: Write>(dds: &ddsfile::Dds, writer: &mut W) -> io::Result<()> {
    todo!()
}
