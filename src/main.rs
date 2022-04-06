#![feature(exit_status_error)]

use std::{
    env,
    fs::{copy, create_dir_all},
    path::Path,
    process::Command,
};

#[cfg(debug_assertions)]
const MODE: Mode = Mode::Debug;
#[cfg(not(debug_assertions))]
const MODE: Mode = Mode::Release;

const OS_PKG: &str = "ingram-kernel";
const OS_TARGET: &str = "x86_64-unknown-none";

const UEFI_PKG: &str = "ingram-uefi";
const UEFI_TARGET: &str = "x86_64-unknown-uefi";

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    build_os(dir);
    build_uefi(dir);
    copy_file(dir);
    boot_qemu(dir);
}

fn build_os(dir: &Path) {
    Command::new("cargo")
        .current_dir(dir.join("kernel"))
        .args(&["build", MODE.to_flag()])
        .args(&["--package", OS_PKG])
        .status()
        .expect("Failed to build os")
        .exit_ok()
        .expect("Error occurred while building os")
}

fn build_uefi(dir: &Path) {
    Command::new("cargo")
        .current_dir(dir.join("uefi"))
        .args(&["build", MODE.to_flag()])
        .args(&["--package", UEFI_PKG])
        .status()
        .expect("Failed to build UEFI")
        .exit_ok()
        .expect("Error occurred while building UEFI")
}

fn copy_file(dir: &Path) {
    let target_dir = dir.join("target");
    let os_dir = target_dir.join(OS_TARGET).join(MODE.to_str());
    let uefi_dir = target_dir.join(UEFI_TARGET).join(MODE.to_str());

    let os_file = os_dir.join("ingram-kernel");
    let uefi_file = uefi_dir.join("ingram-uefi.efi");

    let build_dir = dir.join("build");
    let boot_dir = build_dir.join("EFI").join("BOOT");
    create_dir_all(&boot_dir).expect("Failed to create boot directory");

    let os_dest = boot_dir.join("KERNEL");
    let uefi_dest = boot_dir.join("BOOT.EFI");
    copy(os_file, os_dest).unwrap();
    copy(uefi_file, uefi_dest).unwrap();
}

fn boot_qemu(dir: &Path) {
    let build_dir = dir.join("build");

    Command::new("qemu-system-x86_64")
        .arg("-enable-kvm")
        // Disable default devices
        .arg("-nodefaults")
        // Prevent attempting a PXE (network) boot when no boot disk is found
        .args(&["-net", "none"])
        // Use a standard VGA for graphics
        .args(&["-vga", "std"])
        // Setup monitor
        .args(&["-monitor", "vc:1024x768"])
        // Use a modern machine, with acceleration if possible.
        .args(&["-machine", "q35,accel=kvm:tcg"])
        // Allocate some memory
        .args(&["-m", "4G"])
        // Enable serial
        .args(&["-serial", "stdio"])
        // Add the special ISA debug exit device
        // https://github.com/andre-richter/qemu-exit#x86_64
        .args(&["-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"])
        // Set up OVMF
        .args(&["-bios", "/usr/share/edk2-ovmf/x64/OVMF.fd"])
        // Mount a local directory as a FAT partition
        .args(&[
            "-drive",
            format!("format=raw,file=fat:rw:{}", build_dir.to_str().unwrap()).as_str(),
        ])
        .status()
        .expect("Failed to start qemu")
        .exit_ok()
        .expect("Error occurred while starting qemu");
}

enum Mode {
    #[allow(dead_code)]
    Debug,
    #[allow(dead_code)]
    Release,
}

impl Mode {
    fn to_str(&self) -> &str {
        match self {
            Mode::Debug => "debug",
            Mode::Release => "release",
        }
    }

    fn to_flag(&self) -> &str {
        match self {
            Mode::Debug => "--locked",
            Mode::Release => "--release",
        }
    }
}
