//! `cann-sys` 构建脚本。
//!
//! 负责：
//! - 自动发现 CANN SDK 安装路径
//! - 启用 `ffi` 特性时链接 `libascendcl.so`

use std::path::PathBuf;
use std::{env, fs};

/// 查找 CANN SDK 安装目录。
///
/// 按优先级依次检测以下路径：
/// 1. `ASCEND_TOOLKIT_HOME` 环境变量
/// 2. `ASCEND_HOME_PATH` 环境变量
/// 3. `ASCEND_HOME` 环境变量
/// 4. `$HOME/Ascend/cann`
/// 5. `/usr/local/Ascend`
///
/// 需要目录下存在 `include/acl/acl_rt.h` 和 `lib64/libascendcl.so` 才确认有效。
///
/// 返回 `(base_dir, include_dir, lib_dir)` 三元组。
fn find_cann_sdk() -> (PathBuf, PathBuf, PathBuf) {
    let candidates = [
        env::var("ASCEND_TOOLKIT_HOME").ok().map(PathBuf::from),
        env::var("ASCEND_HOME_PATH").ok().map(PathBuf::from),
        env::var("ASCEND_HOME").ok().map(PathBuf::from),
        env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join("Ascend").join("cann")),
        Some(PathBuf::from("/usr/local/Ascend")),
    ];

    for candidate in candidates.iter().flatten() {
        let base = fs::canonicalize(candidate).unwrap_or_else(|_| candidate.clone());
        let include = base.join("include");
        let lib = base.join("lib64");
        if include.join("acl").join("acl_rt.h").exists() && lib.join("libascendcl.so").exists() {
            return (base, include, lib);
        }
    }

    eprintln!();
    eprintln!("错误: 未找到 CANN SDK。");
    eprintln!();
    eprintln!("提示: 设置 ASCEND_TOOLKIT_HOME 环境变量，或将 CANN 安装到默认路径。");
    eprintln!("已搜索路径:");
    for candidate in candidates.iter().flatten() {
        eprintln!("  - {}", candidate.display());
    }
    eprintln!();
    std::process::exit(1);
}

/// 构建入口。
fn main() {
    println!("cargo::rustc-check-cfg=cfg(cann_sys_ffi)");

    let (sdk_base, include_dir, lib_dir) = find_cann_sdk();

    let ffi_enabled = env::var("CARGO_FEATURE_FFI").is_ok();
    if ffi_enabled {
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=ascendcl");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
        let devlib = sdk_base.join("aarch64-linux").join("devlib");
        if devlib.exists() {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", devlib.display());
        }
        println!("cargo:rustc-link-arg=-Wl,--allow-shlib-undefined");
    }
    if ffi_enabled {
        println!("cargo:rustc-cfg=cann_sys_ffi");
    }

    println!("cargo:rerun-if-env-changed=ASCEND_TOOLKIT_HOME");
    println!("cargo:rerun-if-env-changed=ASCEND_HOME_PATH");
    println!("cargo:rerun-if-env-changed=ASCEND_HOME");
    println!(
        "cargo:rerun-if-changed={}",
        include_dir.join("acl").join("acl_rt.h").display()
    );
}