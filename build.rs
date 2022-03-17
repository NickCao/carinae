fn main() {
    cxx_build::bridge("src/main.rs")
        .file("src/nix.cc")
        .flag_if_supported("-std=c++17")
        .compile("carinae");
    println!("cargo:rerun-if-changed=include/nix.hh");
    println!("cargo:rerun-if-changed=src/nix.cc");
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rustc-link-lib=nixstore");
}
