extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let macos_err: &'static str = "Unable to find the Vulkan SDK, which is required on MacOS";

    // Adjust the library search path for MacOs.
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-search={}/lib", env::var("VULKAN_SDK").expect(macos_err));

    // Tell rustc to link against the vulkan library.
    println!("cargo:rustc-link-lib=vulkan");

    // Generate the rust bindings for vulkan.
    let builder = bindgen::Builder::default()
        .header("vulkan/vulkan.h");

    #[cfg(target_os = "ios")]
    let builder = builder.clang_arg("-DVK_USE_PLATFORM_IOS_MVK");

    #[cfg(target_os = "macos")]
    let builder = builder.clang_arg("-DVK_USE_PLATFORM_MACOS_MVK");

    #[cfg(target_os = "windows")]
    let builder = builder.clang_arg("-DVK_USE_PLATFORM_WIN32_KHR");

    let bindings = builder
        .generate()
        .expect("Unable to generate the bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Unable to write the bindings to a file");
}
