extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to link this library against
    println!("cargo:rustc-link-lib=vulkan");
    println!("cargo:rustc-link-search={}/lib", env::var("VULKAN_SDK").unwrap());

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}/include", env::var("VULKAN_SDK").unwrap()))
        .generate()
        .expect("Unable to generate the bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Unable to write the bindings to a file");
}
