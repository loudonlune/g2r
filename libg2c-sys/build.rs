
extern crate bindgen;
use std::{fs::{self, File}, io::Write, path::PathBuf, process::Command};

fn main() {
    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    if fs::metadata(out_path.join("NCEPLIBS-g2c")).is_err() {
        if Command::new("git")
        .current_dir(out_path.clone())
        .arg("clone")
        .arg("https://github.com/NOAA-EMC/NCEPLIBS-g2c.git")
        .spawn()
        .expect("Clone the NCEPLIBS-g2c repository")
        .wait()
        .expect("An exit status from git")
        .code()
        .expect("An exit status from git") != 0 {
            panic!("Cloning NCEPLIBS-g2c failed!")
        }
    } else {
        println!("Using existing NCEPLIBS-g2c repo...");

        Command::new("rm")
        .arg("-rf")
        .current_dir(out_path.clone().join("NCEPLIBS-g2c"))
        .arg("build")
        .spawn()
        .expect("Clean of build to start executing")
        .wait()
        .expect("Clean of g2c build dir to complete");
    }

    Command::new("mkdir")
    .current_dir(out_path.clone().join("NCEPLIBS-g2c"))
    .arg("build")
    .spawn()
    .expect("mkdir to run")
    .wait()
    .expect("mkdir to complete");

    Command::new("cmake")
    .current_dir(out_path.clone().join("NCEPLIBS-g2c/build/"))
    .arg("-DUSE_Jasper=False")
    .arg("-DUSE_PNG=False")
    .arg("..")
    .spawn()
    .expect("CMake to succeed")
    .wait()
    .expect("Cmake to finish");

    let bindings = bindgen::Builder::default()
    .header(out_path.clone().join("NCEPLIBS-g2c/build/src/grib2.h").as_os_str().to_str().unwrap())
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    .generate()
    .expect("Failed to generate bindgen bindings");

    bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("Failed to write generated g2c binding");

    // make absolutely sure buffer is flushed & file is written
    std::mem::drop(
        File::create(
            out_path.join("lib.rs")
        ).unwrap()
        .write("include!(\"bindings.rs\")".as_bytes())
    );
}
