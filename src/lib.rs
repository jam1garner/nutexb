pub mod writer;
pub mod parser;
pub mod tegra_swizzle;

use std::io::{prelude::*, self};
use std::path::Path;
pub use ddsfile;

pub trait DdsExt {
    fn from_nutexb(data: &[u8]) -> ddsfile::Dds;
    fn write_nutexb<W: Write>(&self, writer: &mut W, name: &str) -> io::Result<()> ;
    fn read_nutexb<R: Read>(reader: &mut R) -> ddsfile::Dds {
        let mut buffer = vec![];
        reader.read_to_end(&mut buffer).unwrap();
        Self::from_nutexb(&buffer)
    }

    fn write_nutexb_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let path = path.as_ref();
        self.write_nutexb(
            &mut std::fs::File::create(path)?,
            path.file_stem().unwrap().to_str().unwrap()
        )
    }
}

impl DdsExt for ddsfile::Dds {
    fn from_nutexb(data: &[u8]) -> ddsfile::Dds {
        todo!()
    }
    
    fn write_nutexb<W: Write>(&self, writer: &mut W, name: &str) -> io::Result<()> {
        writer::write_nutexb(name, self, writer)
    }
}
