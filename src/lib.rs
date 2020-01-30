mod writer;
mod parser;

use std::io::prelude::*;
pub use ddsfile;

pub trait DdsExt {
    fn from_nutexb(data: &[u8]) -> ddsfile::Dds;
    fn write_nutexb<W: Write>(&self, writer: &mut W);
    fn read_nutexb<R: Read>(reader: &mut R) -> ddsfile::Dds {
        let mut buffer = vec![];
        reader.read_to_end(&mut buffer).unwrap();
        Self::from_nutexb(&buffer)
    }
}

impl DdsExt for ddsfile::Dds {
    fn from_nutexb(data: &[u8]) -> ddsfile::Dds {
        todo!()
    }
    
    fn write_nutexb<W: Write>(&self, writer: &mut W) {
        writer::write_nutexb(self, writer).unwrap()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
