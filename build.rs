fn main() {
    // Îi spune lui cargo: dacă se schimbă linker.ld, recompilează tot.
    println!("cargo:rerun-if-changed=linker.ld");
    // Și îi spune linker-ului unde să caute fișierul.
    println!("cargo:rustc-link-search={}", env!("CARGO_MANIFEST_DIR"));
}
