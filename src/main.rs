use std::{
    fs::File,
    path::{Path, PathBuf},
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input_path = Path::new(&args[1]);

    // Infer the extension to allow drag and drop support.
    let new_extension = match input_path.extension().unwrap().to_str().unwrap() {
        "nutexb" => "dds",
        _ => "nutexb",
    };
    let output_path = args
        .get(2)
        .map(|a| PathBuf::from(a))
        .unwrap_or(input_path.with_extension(new_extension));

    let output_name = output_path.file_name().unwrap().to_str().unwrap();
    let output_file = File::create(&output_path).unwrap();
    let mut output_file = std::io::BufWriter::new(output_file);

    let start = std::time::Instant::now();
    match input_path.extension().unwrap().to_str().unwrap() {
        "dds" => {
            let mut reader = File::open(input_path).unwrap();
            let dds = nutexb::ddsfile::Dds::read(&mut reader).unwrap();
            nutexb::write_nutexb(output_name, &dds, &mut output_file).unwrap();
        }
        "nutexb" => {
            let nutexb = nutexb::NutexbFile::read_from_file(input_path).unwrap();
            let dds = nutexb::create_dds(&nutexb).unwrap();
            dds.write(&mut output_file).unwrap();
        }
        _ => {
            let image = image::open(input_path).unwrap();
            nutexb::write_nutexb(output_name, &image, &mut output_file).unwrap();
        }
    }
    println!("Completed operation in {:?}", start.elapsed());
}
