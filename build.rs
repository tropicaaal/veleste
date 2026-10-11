use std::path::PathBuf;

fn main() {
    let dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("link");
    println!("cargo:rustc-link-search=native={}", dir.display());
    println!("cargo:rustc-link-lib=static=celeste");
    println!("cargo:rustc-link-lib=static=c");
    println!("cargo:rustc-link-lib=static=m");
    println!("cargo:rerun-if-changed=link/libceleste.a");
}