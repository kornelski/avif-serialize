use std::path::Path;
use std::fs;

// Careful! This throws away alpha channel, until this is resolved:
// https://github.com/mozilla/mp4parse-rust/pull/239
fn main() {
    let path = std::env::args_os().nth(1).expect("Please specify path to an AVIF file to optimize");

    let avif = fs::read(&path).expect("Can't load input image");

    let mut ctx = mp4parse::AvifContext::new();
    mp4parse::read_avif(&mut avif.as_slice(), &mut ctx).unwrap();

    // Chrome won't like the 0 size (https://crbug.com/1120973)
    // - put real size in your code.
    // Firefox doesn't mind it tho.
    let out = avif_serialize::serialize_to_vec(&ctx.primary_item, None, 0, 0, 8);

    let new_path = Path::new(&path).with_extension("rewrite.avif");
    fs::write(&new_path, out).expect("Can't write new file");
    eprintln!("Written {}", new_path.display());
}
