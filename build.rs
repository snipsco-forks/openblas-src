extern crate cc;

use std::env;
use std::path::{ Path, PathBuf };
use std::process::Command;

macro_rules! binary(() => (if cfg!(target_pointer_width = "32") { "32" } else { "64" }));
macro_rules! feature(($name:expr) => (env::var(concat!("CARGO_FEATURE_", $name)).is_ok()));
macro_rules! switch(($condition:expr) => (if $condition { "YES" } else { "NO" }));
macro_rules! variable(($name:expr) => (env::var($name).unwrap()));

fn main() {
    for v in env::vars() {
        println!("{:?}", v);
    }
    let kind = if feature!("STATIC") {
        "static"
    } else {
        "dylib"
    };
    if !feature!("SYSTEM") {
        let output = PathBuf::from(variable!("OUT_DIR").replace(r"\", "/"));
        if !output.join("libopenblas.a").exists() {
            build(&output);
        }
        println!(
            "cargo:rustc-link-search={}",
            output.join("opt/OpenBLAS/lib").display(),
        );
    }
    println!("cargo:rustc-link-lib={}=openblas", kind);
//    println!("cargo:rustc-link-lib=dylib=gfortran");
}

fn build(output: &Path) {
    let cblas = feature!("CBLAS");
    let lapacke = feature!("LAPACKE");
    let source = PathBuf::from("source");
    let target = variable!("TARGET");
    let host = variable!("HOST");
    let cross_args = if target != host {
        let cc = cc::Build::new().get_compiler();
        let toolchain_bin_dir = cc.path().parent().expect("toolchain is a dir");
        let cc_exe = cc.path().file_name().expect("cc was not a file")
            .to_string_lossy();
        let splitted_exe = cc_exe.split("-").collect::<Vec<_>>();
        let prefix = splitted_exe[0..splitted_exe.len()-1].join("-");
        let fc = toolchain_bin_dir.join(format!("{}-gfortran", prefix));
        let ar = toolchain_bin_dir.join(format!("{}-ar", prefix));
        vec!(
            format!("CC={}", cc.path().to_string_lossy()),
//            format!("FC={}", fc.to_string_lossy()),
            format!("AR={}", ar.to_string_lossy()),
            format!("NOFORTRAN=1"),
            "HOSTCC=cc".to_string(),
            "TARGET=ARMV6".to_string(),
        )
    } else {
        vec!()
    };
    env::remove_var("TARGET");
    run(
        Command::new("make")
            .args(&["libs", "netlib", "shared"])
            .arg(format!("BINARY={}", variable!("CARGO_CFG_TARGET_POINTER_WIDTH")))
            .arg(format!("{}_CBLAS=1", switch!(cblas)))
            .arg(format!("{}_LAPACKE=1", switch!(lapacke)))
            // .arg(format!("-j{}", variable!("NUM_JOBS")))
            .args(cross_args)
            .current_dir(&source),
    );
    run(
        Command::new("make")
            .arg("install")
            .arg(format!("DESTDIR={}", output.display()))
            .current_dir(&source),
    );
}

fn run(command: &mut Command) {
    println!("Running: `{:?}`", command);
    match command.status() {
        Ok(status) => if !status.success() {
            panic!("Failed: `{:?}` ({})", command, status);
        },
        Err(error) => {
            panic!("Failed: `{:?}` ({})", command, error);
        }
    }
}
