use std::fs;
use std::path::Path;

fn main() {
    let path = std::env::args_os().nth(1).expect("Please specify path to an AVIF file to optimize");

    let avif_file = fs::read(&path).expect("Can't load input image");

    let avif = avif_parse::read_avif(&mut avif_file.as_slice()).unwrap();

    // Chrome won't like the 0 size (https://crbug.com/1120973)
    // - put real size in your code.
    // Firefox doesn't mind it tho.
    let out = avif_serialize::serialize_to_vec(&avif.primary_item, avif.alpha_item.as_deref(), 0, 0, 8);

    let new_path = Path::new(&path).with_extension("rewrite.avif");
    fs::write(&new_path, out).expect("Can't write new file");
    eprintln!("Written {}", new_path.display());
}
