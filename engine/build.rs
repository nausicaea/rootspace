use std::process::Command;

fn main() {
    if cfg!(unix) {
        let output = Command::new("uname")
            .arg("-r")
            .output()
            .expect("The command `uname -r` could not be executed");

        let kernel_version = String::from_utf8(output.stdout)
            .expect("Could not parse the command output as valid UTF-8");

        if kernel_version.contains("Microsoft") {
            println!("cargo:rustc-cfg=feature=\"wsl\"");
        }
    }
}
