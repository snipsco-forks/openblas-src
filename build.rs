extern crate cc;
extern crate copy_dir;

use std::path::PathBuf;
use std::process::Command;

fn yes_no(b:bool) -> &'static str {
    if b { "YES" } else { "NO" }
}

fn cargo_env<S: AsRef<str>>(key: S) -> Option<String> {
    println!("cargo:rerun-if-env-changed={}", key.as_ref());
    match std::env::var(key.as_ref()) {
        Err(std::env::VarError::NotPresent) => None,
        Err(e) => panic!(e),
        Ok(v) => Some(v)
    }
}

fn feature(name: &str) -> bool {
    cargo_env(format!("CARGO_FEATURE_{}", name)).is_some()
}

fn main() {
    let kind = if feature("STATIC") {
        "static"
    } else {
        "dylib"
    };
    if !feature("SYSTEM") {
        build();
    }
    println!("cargo:rustc-link-lib={}=openblas", kind);
    println!("cargo:rerun-if-changed=build.rs");
}

fn build() {
    let cblas = feature("CBLAS");
    let lapacke = feature("LAPACKE");
    let output = PathBuf::from(cargo_env("OUT_DIR").unwrap().replace(r"\", "/"));
    let source = output.join("source");
    if source.exists() {
        std::fs::remove_dir_all(&source).unwrap();
    }
    copy_dir::copy_dir("source", &source).unwrap();
    let tool = cc::Build::new().get_compiler();
    let mut cc = tool.path().to_str().unwrap().to_string();
    let mut args:Vec<String> = vec!();

    if let Some(t) = cargo_env("OPENBLAS_TARGET") {
        args.push(format!("TARGET={}", t));
    } else if cargo_env("TARGET").unwrap() == cargo_env("HOST").unwrap() {
    } else if cargo_env("TARGET").unwrap().contains("ios") {
        let out = Command::new("xcrun")
            .args(&["--sdk", "iphoneos", "-f", "clang" ]).output().unwrap();
        let clang = String::from_utf8(out.stdout).unwrap();
        let out = Command::new("xcrun")
            .args(&["--sdk", "iphoneos", "--show-sdk-path" ]).output().unwrap();
        let sysroot = String::from_utf8(out.stdout).unwrap();
        cc = format!("{} -isysroot {} -arch arm64", clang.trim(), sysroot.trim());
    } else if cargo_env("CARGO_CFG_TARGET_OS").unwrap() == "android" {
        args.push(format!("AR={}", cargo_env("TARGET_AR").unwrap()));
        args.push("ARM_SOFTFP_ABI=1".into());
        match &*cargo_env("CARGO_CFG_TARGET_ARCH").unwrap() {
            "arm" => {
                cc = format!("{} -march=armv6 -mfpu=vfp -funsafe-math-optimizations -ftree-vectorize", cc.replace("-gcc", "-clang"));
                args.push("NUM_THREADS=2".into());
                args.push("TARGET=ARMV6".into());
            },
            "armv7" => {
                cc = format!("{} -march=armv7 -mfpu=vfp -funsafe-math-optimizations -ftree-vectorize", cc.replace("-gcc", "-clang"));
                args.push("NUM_THREADS=2".into());
                args.push("TARGET=ARMV7".into());
            },
            "aarch64" => {
                cc = format!("{} -march=armv8 -funsafe-math-optimizations -ftree-vectorize", cc.replace("-gcc", "-clang"));
                args.push("NUM_THREADS=4".into());
                args.push("TARGET=ARMV8".into());
            },
            _ => panic!("Please configure openblas-src build for {}", cargo_env("TARGET").unwrap())
        }
    } else {
        if let Some(fc) = cargo_env("TARGET_FC") {
            args.push(format!("FC={}", fc));
            println!("cargo:rustc-link-search={}/lib", cargo_env("TARGET_SYSROOT").unwrap());
            println!("cargo:rustc-link-lib=static=gfortran");
        } else {
            args.push(format!("NOFORTRAN=1"));
        }
        let t = match &*cargo_env("CARGO_CFG_TARGET_ARCH").unwrap() {
            "arm" => "ARMV6",
            "armv7" => "ARMV7",
            "armv7s" => "CORTEXA9",
            "aarch64" => "ARMV8",
            _ => panic!()
        };
        args.push(format!("TARGET={}", t));
    }

    if feature("STATIC") {
        args.push("NO_SHARED=1".into());
    }

    let targets = vec!("libs", "netlib", "shared", "install");

    for i in targets {
        let mut build_command = Command::new("make");
        build_command
            .arg(format!("BINARY={}", cargo_env("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap()))
            .arg(format!("{}_CBLAS=1", yes_no(cblas)))
            .arg(format!("{}_LAPACKE=1", yes_no(lapacke)))
            .arg(format!("CC={}", cc))
            .arg(format!("HOSTCC={}", cargo_env("HOST_CC").unwrap_or("cc".into())))
            .arg(format!("DESTDIR={}", output.display()))
            .args(&args)
            .env_remove("TARGET")
            .current_dir(&source)
            .arg(i);

        run(&mut build_command);
    }

    println!(
        "cargo:rustc-link-search={}",
        output.join("opt/OpenBLAS/lib").display(),
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
