extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn generate_vk_binding() {
    let builder = bindgen::Builder::default()
        .header("vulkan/vulkan.h")
        .whitelist_type("^Vk.*");

    let builder = if cfg!(target_os = "ios") {
        builder.clang_arg("-DVK_USE_PLATFORM_IOS_MVK")
    } else if cfg!(target_os = "macos") {
        builder.clang_arg("-DVK_USE_PLATFORM_MACOS_MVK")
    } else if cfg!(target_os = "windows") {
        builder
            .clang_arg("-DVK_USE_PLATFORM_WIN32_KHR")
            .clang_arg("-IC:/Program Files (x86)/Windows Kits/10/Include/10.0.17134.0/cppwinrt")
            .clang_arg("-IC:/Program Files (x86)/Windows Kits/10/Include/10.0.17134.0/shared")
            .clang_arg("-IC:/Program Files (x86)/Windows Kits/10/Include/10.0.17134.0/ucrt")
            .clang_arg("-IC:/Program Files (x86)/Windows Kits/10/Include/10.0.17134.0/um")
            .clang_arg("-IC:/Program Files (x86)/Windows Kits/10/Include/10.0.17134.0/winrt")
            .clang_arg("-IC:/Program Files (x86)/Microsoft Visual Studio 14.0/VC/include")
    } else if cfg!(target_os = "android") {
        builder.clang_arg("-DVK_USE_PLATFORM_ANDROID_KHR")
    } else {
        builder
    };

    if let Ok(binding) = builder.generate() {
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        binding
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Unable to write the binding to a file");
    } else {
        panic!("Unable to generate the vulkan binding");
    }
}

fn main() {
    // Tell the rust compiler how to link against vulkan
    if cfg!(target_os = "macos") {
        let macos_err: &'static str = "Unable to find the Vulkan SDK, which is required on MacOS";
        println!("cargo:rustc-link-search={}/lib", env::var("VULKAN_SDK").expect(macos_err));
        println!("cargo:rustc-link-lib=vulkan");
    } else if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=vulkan-1");
    } else {
        println!("cargo:rustc-link-lib=vulkan");
    }

    // Generate the rust bindings for vulkan.
    generate_vk_binding();
}
