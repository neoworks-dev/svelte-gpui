//! Generates the operation codes from `protocol.json`, which the runtime reads too.

use std::fmt::Write;

fn main() {
    let path = "../protocol.json";
    println!("cargo:rerun-if-changed={path}");
    let text = std::fs::read_to_string(path).expect("protocol.json next to host/");
    let protocol: serde_json::Value = serde_json::from_str(&text).expect("protocol.json parses");
    let mut out = String::new();
    for (code, name) in protocol["operations"]
        .as_array()
        .expect("operations")
        .iter()
        .enumerate()
    {
        let name = name.as_str().expect("operation name");
        let mut constant = String::new();
        for character in name.chars() {
            if character.is_uppercase() {
                constant.push('_');
            }
            constant.push(character.to_ascii_uppercase());
        }
        writeln!(out, "pub const {constant}: i64 = {code};").unwrap();
    }
    let target = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("protocol.rs");
    std::fs::write(target, out).unwrap();
}
