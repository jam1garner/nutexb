fn main() {
    // TODO: Print usage and add better argument handling.
    let args: Vec<String> = std::env::args().collect();
    let input_image_path = std::path::Path::new(&args[1]);
    let output_nutex_path = std::path::Path::new(&args[2]);

    // TODO: Add meaningful error messages.
    let output_name = output_nutex_path.file_name().unwrap().to_str().unwrap();
    let output_file = std::fs::File::create(output_nutex_path).unwrap();
    let mut output_file = std::io::BufWriter::new(output_file);

    match input_image_path.extension().unwrap().to_str().unwrap() {
        "dds" => {
            let mut reader = std::fs::File::open(input_image_path).unwrap();
            let dds = nutexb::ddsfile::Dds::read(&mut reader).unwrap();
            nutexb::writer::write_nutexb(output_name, &dds, &mut output_file).unwrap();
        }
        _ => {
            let image = image::open(input_image_path).unwrap();
            nutexb::writer::write_nutexb(output_name, &image, &mut output_file).unwrap();
        }
    }
}
