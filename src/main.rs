use nutexb::ddsfile::*;
use nutexb::DdsExt;
use std::io::Cursor;

fn main() {
    let mut reader = Cursor::new(&include_bytes!("def_ike_001_osan_col.dds")[..]);
    let dds = Dds::read(&mut reader).unwrap();
    dds.write_nutexb_to_file("def_ike_001_osan_col.nutexb").unwrap();
    let mut reader = Cursor::new(&include_bytes!("ike.dds")[..]);
    let dds = Dds::read(&mut reader).unwrap();
    std::fs::write("ike_layer_data.dds", &dds.data).unwrap();
}
