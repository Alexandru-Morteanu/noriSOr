use std::{env, path::PathBuf, process::Command};

fn main() {
    // Daca se schimba harta sau vectorii, recompilam.
    println!("cargo:rerun-if-changed=linker.ld");
    println!("cargo:rerun-if-changed=src/boot/vectors.S");
    println!("cargo:rustc-link-search={}", env!("CARGO_MANIFEST_DIR"));

    // Vectorii se asambleaza cu asamblorul GNU din toolchain-ul Espressif,
    // singurul care cunoaste instructiunile de administrare a ferestrelor.
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let obj = out.join("vectors.o");

    let stare = Command::new("xtensa-esp32s3-elf-gcc")
        .arg("-c")
        .arg("-o").arg(&obj)
        .arg("src/boot/vectors.S")
        .status()
        .expect("nu gasesc xtensa-esp32s3-elf-gcc in PATH");

    assert!(stare.success(), "asamblarea vectorilor a esuat");

    // Il dam direct linkerului. KEEP() din linker.ld il apara de --gc-sections.
    println!("cargo:rustc-link-arg={}", obj.display());
}
