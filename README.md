# AVIF image serializer (muxer)

Minimal writer for AVIF header structure. This is a tiny alternative to [libavif](//lib.rs/libavif).
It creates the jungle of MPEG/HEIF/MIAF/ISO-BMFF "boxes" as appropriate for AVIF files. Supports alpha channel embedding.

Compatible with decoders in Chrome 85, libavif v0.8.1, and Firefox 81a.

Together with [rav1e](//lib.rs/rav1e) it allows pure-Rust AVIF image encoding.

## Requirements

* Rust 1.45

## Usage

1. Compress pixels using an AV1 encoder, such as [rav1e](//lib.rs/rav1e). [libaom](//lib.rs/libaom-sys) works too.

2. Call `avif_serialize::serialize_to_vec(av1_data, None, width, height, 8)`

