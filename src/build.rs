use std::{env, fs, io::Read, path::PathBuf};

use xz2::read::XzEncoder;

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let definition = json::stringify(json::parse(include_str!("vial.json")).unwrap());
    let mut compressed = Vec::new();
    XzEncoder::new(definition.as_bytes(), 6)
        .read_to_end(&mut compressed)
        .unwrap();
    fs::write(
        out.join("config_generated.rs"),
        format!(
            "pub const VIAL_KEYBOARD_ID: [u8; 8] = [0x83, 0xD1, 0x49, 0xE5, 0xA7, 0x60, 0x2C, 0xBF];\n\
             pub const VIAL_KEYBOARD_DEF: [u8; {}] = {:?};\n",
            compressed.len(),
            compressed,
        ),
    )
    .unwrap();
    fs::write(out.join("memory.x"), include_bytes!("memory.x")).unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=src/memory.x");
    println!("cargo:rerun-if-changed=src/keyboard.toml");
    println!("cargo:rerun-if-changed=src/vial.json");
    println!("cargo:rustc-link-arg=--nmagic");
    println!("cargo:rustc-link-arg=-Tlink.x");
    println!("cargo:rustc-link-arg=-Tdefmt.x");
}
