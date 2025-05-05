extern crate cc;

fn main() {
    let profile = std::env::var("PROFILE").unwrap();
    let mut build = cc::Build::new();

    if profile == "debug" || profile == "dev" || profile == "test" {
        build.define("DEBUG", None);
    }

    println!("cargo:rerun-if-changed=src/internal/pk_log.c");
    println!("cargo:rerun-if-changed=src/internal/pk_server.c");
    println!("cargo:rerun-if-changed=src/internal/pk_server.h");

    build
        .file("src/internal/pk_server.c")
        .file("src/internal/pk_server.h")
        .file("src/internal/pk_log.c")
        .include("src/internal")
        .define("_GNU_SOURCE", None)
        .compile("pk_server");

    println!("cargo:rustc-link-lib=static=pk_server");
}
