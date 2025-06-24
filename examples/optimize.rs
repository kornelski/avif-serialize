use avif_serialize::Aviffy;
use std::fs;
use std::path::Path;

fn main() {
    let path = std::env::args_os().nth(1).expect("Please specify path to an AVIF file to optimize");

    let avif_file = fs::read(&path).expect("Can't load input image");

    let avif = avif_parse::read_avif(&mut avif_file.as_slice()).unwrap();
    let info = avif.primary_item_metadata().unwrap();

    let out = Aviffy::new()
        .set_seq_profile(info.seq_profile)
        .set_chroma_subsampling(info.chroma_subsampling)
        .set_monochrome(info.monochrome)
        .to_vec(
            &avif.primary_item,
            avif.alpha_item.as_deref(),
            info.max_frame_width.get(),
            info.max_frame_height.get(),
            info.bit_depth,
        );

    let new_path = Path::new(&path).with_extension("rewrite.avif");
    fs::write(&new_path, out).expect("Can't write new file");
    eprintln!("Written {}", new_path.display());
}
