use crate::NutexbFile;
use binrw::prelude::*;
use std::io::{Read, Seek, SeekFrom};

// TODO: This whole file can be merged with lib.rs?

/// Reads the nutexb from the specified `path`. The entire file is buffered to improve performance.
pub fn read_nutexb<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<NutexbFile, Box<dyn std::error::Error>> {
    let mut file = std::io::Cursor::new(std::fs::read(path)?);
    let nutexb = file.read_le::<NutexbFile>()?;
    Ok(nutexb)
}

impl NutexbFile {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        reader.read_le::<NutexbFile>()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        let mut file = std::fs::File::open("/home/jam/Downloads/alp_ike_002_col.nutexb").unwrap();

        let x: NutexbFile = file.read_le().unwrap();

        dbg!(x.footer);
    }
}
