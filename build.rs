extern crate cc;

fn main() {
    println!("cargo:rerun-if-changed=src/server/server.c");
    println!("cargo:rerun-if-changed=src/server/server/h");

    cc::Build::new()
        .file("src/server/server.c")
        .include("src/server")
        .define("_GNU_SOURCE", None)
        .compile("server");

    println!("cargo:rustc-link-lib=static=server");
}
