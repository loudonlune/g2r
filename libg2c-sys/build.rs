
use std::{fs, process::Command};

fn main() {
    if fs::metadata("libg2c.a").is_err() {
        if fs::metadata("NCEPLIBS-g2c").is_err() {
            if Command::new("git")
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
            .current_dir("NCEPLIBS-g2c")
            .arg("build")
            .spawn()
            .expect("Clean of build to start executing")
            .wait()
            .expect("Clean of g2c build dir to complete");
        }
    
        Command::new("mkdir")
        .current_dir("NCEPLIBS-g2c")
        .arg("build")
        .spawn()
        .expect("mkdir to run")
        .wait()
        .expect("mkdir to complete");
    
        Command::new("cmake")
        .current_dir("NCEPLIBS-g2c/build/")
        .arg("-DUSE_Jasper=False")
        .arg("-DUSE_PNG=False")
        .arg("..")
        .spawn()
        .expect("CMake to succeed")
        .wait()
        .expect("Cmake to finish");
    
        if Command::new("bash")
        .arg("./make.sh")
        .spawn()
        .expect("Make to run")
        .wait()
        .expect("Make to run")
        .code()
        .expect("An exit code from Make") != 0 {
            panic!("g2c build failed")
        }
    
        Command::new("mv")
        .arg("NCEPLIBS-g2c/build/libg2c.a")
        .arg("./")
        .spawn()
        .expect("mv to run")
        .wait().expect("mv to finish");
    }

    println!("cargo:rustc-link-search=./");
    println!("cargo:rustc-link-lib=g2c");
}