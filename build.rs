use std::{fs, process::Command};

fn main() {
    println!("cargo:rustc-link-search=./libg2c-sys/");
    println!("cargo:rustc-link-lib=g2c");
}
