//! `cann` 构建脚本：跨版本 SDK 能力探测。
//!
//! 生成 `cann_sdk_has_<symbol>` cfg（check-cfg 无条件声明），供本 crate
//! 在编译期选择实现路径：
//! - `cann_sdk_has_aclsys_get_version_str`：CANN 8.x+ 有 `aclsysGetVersionStr`
//!   （7.x 无，版本查询回退 `aclrtGetVersion`）
//! - `cann_sdk_has_aclgrph`：CANN 8.x+ 有 GE 图引擎 C API（7.x 无，Graph 降级）

use std::path::{Path, PathBuf};
use std::{env, fs};

/// 查找 SDK include 目录（与 cann-sys/build.rs 一致的探测链，无需 lib64 校验）。
fn find_include_dir() -> Option<PathBuf> {
    let candidates: Vec<PathBuf> = [
        env::var("ASCEND_TOOLKIT_HOME").ok(),
        env::var("ASCEND_HOME_PATH").ok(),
        env::var("ASCEND_AICPU_PATH").ok(),
        env::var("ASCEND_HOME").ok(),
        env::var("ASCEND_OPP_PATH").ok(),
    ]
    .into_iter()
    .flatten()
    .map(PathBuf::from)
    .collect();
    let mut all: Vec<PathBuf> = candidates;
    if let Ok(opp) = env::var("ASCEND_OPP_PATH")
        && let Some(parent) = PathBuf::from(opp).parent()
    {
        all.push(parent.to_path_buf());
    }
    for cand in all {
        let include = cand.join("include");
        if include.join("acl").join("acl_rt.h").exists() {
            return Some(include);
        }
    }
    for fallback in [PathBuf::from("/usr/local/Ascend")] {
        let include = fallback.join("include");
        if include.join("acl").join("acl_rt.h").exists() {
            return Some(include);
        }
        if let Ok(entries) = fs::read_dir(&fallback) {
            for e in entries.flatten() {
                let include = e.path().join("include");
                if include.join("acl").join("acl_rt.h").exists() {
                    return Some(include);
                }
            }
        }
    }
    None
}

/// 扫描 SDK 头文件树中是否出现目标符号（含 L1 头文件家族目录）。
fn symbol_present(include_dir: &Path, sym: &str) -> bool {
    let mut found = false;
    let mut dirs: Vec<PathBuf> = vec![include_dir.join("acl")];
    for extra in ["parser", "graph"] {
        let d = include_dir.join(extra);
        if d.is_dir() {
            dirs.push(d);
        }
    }
    'outer: for dir in dirs {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "h")
                && let Ok(content) = fs::read_to_string(&path)
                && content.contains(sym)
            {
                found = true;
                break 'outer;
            }
        }
    }
    found
}

fn main() {
    // 无条件 check-cfg（跨版本：不存在的 cfg 激活不合法但声明合法）
    println!("cargo::rustc-check-cfg=cfg(cann_sdk_has_aclsys_get_version_str)");
    println!("cargo::rustc-check-cfg=cfg(cann_sdk_has_aclgrph)");

    if let Some(include) = find_include_dir() {
        if symbol_present(&include, "aclsysGetVersionStr") {
            println!("cargo::rustc-cfg=cann_sdk_has_aclsys_get_version_str");
        }
        if symbol_present(&include, "aclgrphParseONNX") {
            println!("cargo::rustc-cfg=cann_sdk_has_aclgrph");
        }
    }

    println!("cargo:rerun-if-env-changed=ASCEND_TOOLKIT_HOME");
    println!("cargo:rerun-if-env-changed=ASCEND_HOME_PATH");
    println!("cargo:rerun-if-env-changed=ASCEND_AICPU_PATH");
    println!("cargo:rerun-if-env-changed=ASCEND_OPP_PATH");
    println!("cargo:rerun-if-env-changed=ASCEND_HOME");
}
