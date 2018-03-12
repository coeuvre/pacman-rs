extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/stb_image.c")
        .compile("stb_image");
    println!("cargo:rustc-link-lib=static=stb_image");
}