use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let toolchain_dir = Path::new("toolchain");

    if toolchain_dir.exists() {
        println!("SDK already installed");
    } else {
        println!("Installing SDK...");
        let download_url = "https://toolchains.bootlin.com/downloads/releases/toolchains/riscv64-lp64d/tarballs/riscv64-lp64d--musl--stable-2024.05-1.tar.xz";
        let temp_tar_path = "/tmp/toolchain.tar.xz";

        // Download the toolchain tarball
        Command::new("curl")
            .arg(download_url)
            .arg("-o")
            .arg(temp_tar_path)
            .status()
            .expect("Failed to download toolchain");

        // Create the toolchain directory and extract the tarball
        fs::create_dir(toolchain_dir).expect("Failed to create toolchain directory");
        Command::new("tar")
            .arg("-xf")
            .arg(temp_tar_path)
            .arg("-C")
            .arg("toolchain")
            .arg("--strip-components=1")
            .status()
            .expect("Failed to extract toolchain");

        // Remove the temporary tarball
        fs::remove_file(temp_tar_path).expect("Failed to remove temp tarball");
    }

    // Install nightly rust and target
    Command::new("rustup")
        .arg("toolchain")
        .arg("install")
        .arg("nightly")
        .status()
        .expect("Failed to install nightly rust");

    Command::new("rustup")
        .arg("target")
        .arg("add")
        .arg("riscv64gc-unknown-linux-musl")
        .arg("--toolchain")
        .arg("nightly")
        .status()
        .expect("Failed to add target riscv64gc-unknown-linux-musl");

    Command::new("rustup")
        .arg("component")
        .arg("add")
        .arg("rust-src")
        .arg("--toolchain")
        .arg("nightly")
        .status()
        .expect("Failed to add rust-src component");

    Command::new("rustup")
        .arg("component")
        .arg("llvm-tools-preview")
        .arg("--toolchain")
        .arg("nightly")
        .status()
        .expect("Failed to add llvm-tools-preview component");
}