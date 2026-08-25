//! `cann-sys` 构建脚本。
//!
//! 负责：
//! - 自动发现 CANN SDK 安装路径
//! - 启用 `ffi` 特性时链接 `libascendcl.so`
//! - 非 `ffi` 构建时降级：仅探测并打印，不阻断编译（0001 Task 1）
//! - 符号存在性探测：生成 `cann_sys_has_*` cfg 供跨版本门控

use std::path::{Path, PathBuf};
use std::{env, fs};

/// 需要探测存在性的 L0 函数符号（跨版本可能漂移）。
const SYMBOLS: &[&str] = &[
    // 设备
    "aclrtGetDeviceCount",
    "aclrtSetDevice",
    "aclrtResetDevice",
    "aclrtResetDeviceForce",
    "aclrtGetSocName",
    "aclrtSynchronizeDevice",
    "aclrtSynchronizeDeviceWithTimeout",
    // 内存
    "aclrtMalloc",
    "aclrtMallocAlign32",
    "aclrtFree",
    "aclrtMallocHost",
    "aclrtFreeHost",
    "aclrtMemcpy",
    "aclrtMemcpyAsync",
    "aclrtMemset",
    "aclrtMemsetAsync",
    // 流 / 事件
    "aclrtCreateStream",
    "aclrtDestroyStream",
    "aclrtSynchronizeStream",
    "aclrtStreamQuery",
    "aclrtCreateEvent",
    "aclrtDestroyEvent",
    "aclrtRecordEvent",
    "aclrtSynchronizeEvent",
    "aclrtStreamWaitEvent",
    "aclrtEventElapsedTime",
];

/// 读取并规范化环境变量路径（去空字符串与首尾空白）。
fn env_path(var: &str) -> Option<PathBuf> {
    env::var(var)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
}

/// 查找 CANN SDK 安装目录。
///
/// 按优先级依次检测以下路径（均来自 `set_env.sh` 导出的环境变量，不写死版本路径）：
/// 1. `ASCEND_TOOLKIT_HOME`（`set_env.sh` 主变量，= 安装根）
/// 2. `ASCEND_HOME_PATH`（= 安装根）
/// 3. `ASCEND_AICPU_PATH`（= 安装根）
/// 4. `ASCEND_HOME`（旧版本兼容）
/// 5. `ASCEND_OPP_PATH`（= 根/opp，取其父目录）
/// 6. `$HOME/Ascend/cann`（默认安装位置）
/// 7. `/usr/local/Ascend`（官方标准安装根兜底）
///
/// 需要目录下存在 `include/acl/acl_rt.h` 和 `lib64/libascendcl.so` 才确认有效。
fn sdk_candidates() -> Vec<PathBuf> {
    let mut list: Vec<PathBuf> = [
        env_path("ASCEND_TOOLKIT_HOME"),
        env_path("ASCEND_HOME_PATH"),
        env_path("ASCEND_AICPU_PATH"),
        env_path("ASCEND_HOME"),
        env_path("ASCEND_OPP_PATH").and_then(|p| p.parent().map(PathBuf::from)),
    ]
    .into_iter()
    .flatten()
    .collect();
    if let Some(home) = env_path("HOME") {
        list.push(home.join("Ascend").join("cann"));
    }
    list.push(PathBuf::from("/usr/local/Ascend"));
    list
}

fn find_cann_sdk() -> Option<(PathBuf, PathBuf, PathBuf)> {
    let mut checked: Vec<PathBuf> = Vec::new();
    for candidate in sdk_candidates() {
        let base = fs::canonicalize(&candidate).unwrap_or_else(|_| candidate.clone());
        if checked.contains(&base) {
            continue;
        }
        checked.push(base.clone());
        let include = base.join("include");
        let lib = base.join("lib64");
        if include.join("acl").join("acl_rt.h").exists() && lib.join("libascendcl.so").exists() {
            return Some((base, include, lib));
        }
    }
    None
}

/// 收集链接/加载搜索目录（全部存在性校验 + 去重）。
///
/// 与 `set_env.sh` 的 `LD_LIBRARY_PATH` 结构对应（由安装根派生，不写死版本路径）：
/// 主 `lib64`、插件目录（opskernel/nnengine）、op_tiling 库、aml 工具库、driver 库；
/// 再并入用户 `LD_LIBRARY_PATH` 中实际存在的目录（source 过 `set_env.sh` 时驱动等路径自动覆盖）。
fn lib_search_dirs(root: &Path, lib_dir: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let push_if = |dirs: &mut Vec<PathBuf>, p: PathBuf| {
        if p.is_dir() && !dirs.contains(&p) {
            dirs.push(p);
        }
    };
    push_if(&mut dirs, lib_dir.to_path_buf());
    push_if(
        &mut dirs,
        root.join("lib64").join("plugin").join("opskernel"),
    );
    push_if(
        &mut dirs,
        root.join("lib64").join("plugin").join("nnengine"),
    );
    push_if(
        &mut dirs,
        root.join("opp")
            .join("built-in")
            .join("op_impl")
            .join("ai_core")
            .join("tbe")
            .join("op_tiling")
            .join("lib")
            .join("linux"),
    );
    push_if(&mut dirs, root.join("tools").join("aml").join("lib64"));
    push_if(
        &mut dirs,
        root.join("tools").join("aml").join("lib64").join("plugin"),
    );
    push_if(&mut dirs, PathBuf::from("/usr/local/Ascend/driver/lib64"));
    if let Ok(ld) = env::var("LD_LIBRARY_PATH") {
        for part in ld.split(':').map(PathBuf::from) {
            push_if(&mut dirs, part);
        }
    }
    dirs
}

/// 从 `include/acl`（含 `error_codes/` 子目录）探测符号存在性，返回存在的符号名列表（函数 + `ACL_ERROR_RT_*` 宏）。
fn probe_symbols(include_dir: &Path) -> Vec<String> {
    let mut found = Vec::new();
    let mut all = String::new();
    let acl_dir = include_dir.join("acl");
    let Ok(entries) = fs::read_dir(&acl_dir) else {
        return found;
    };
    // 主目录 + error_codes 子目录（错误码头文件所在）
    let mut targets: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    if let Ok(sub) = fs::read_dir(acl_dir.join("error_codes")) {
        targets.extend(sub.flatten().map(|e| e.path()));
    }
    for path in targets {
        if path.extension().is_some_and(|e| e == "h")
            && let Ok(content) = fs::read_to_string(&path)
        {
            all.push_str(&content);
            all.push('\n');
        }
    }
    // 函数符号：直接包含匹配
    for sym in SYMBOLS {
        if all.contains(sym) {
            found.push((*sym).to_string());
        }
    }
    // RT 错误码宏：提取 `ACL_ERROR_RT_*` 定义名
    for line in all.lines() {
        if let Some(stripped) = line.trim_start().strip_prefix("#define ")
            && let Some(name) = stripped.split_whitespace().next()
            && name.starts_with("ACL_ERROR_RT_")
            && !found.iter().any(|f| f == name)
        {
            found.push(name.to_string());
        }
    }
    found
}

/// 符号名 → cfg 名：`cann_sys_has_<原样>`（保留大小写，宏名也保留）。
fn cfg_name(sym: &str) -> String {
    format!("cann_sys_has_{sym}")
}

/// 判断某个 `.so` 的 ELF 字符串表（.dynstr 纯文本）中是否包含目标符号名。
///
/// 零依赖实现（不依赖 `nm`），通过字节窗口匹配符号名字符串。
fn lib_contains_symbol(lib_dir: &Path, lib_file: &str, sym: &str) -> bool {
    let Ok(bytes) = fs::read(lib_dir.join(lib_file)) else {
        return false;
    };
    let needle = sym.as_bytes();
    bytes.windows(needle.len()).any(|w| w == needle)
}

/// 确保 `libascendcl` 基础库之外的多余符号有库可链。
///
/// 不同 SDK 版本/架构会把同一符号放在不同库（如 `aclsysGetVersionStr` 在部分版本位于
/// `libacl_rt.so`/`libascend_common.so`）。流程：基础库已含 → 免链；否则按候选优先级
/// 找到第一个含该符号的库并链接，全无则兜底扫描 lib64 目录。
fn link_symbol_provenance(lib_dir: &Path, sym: &str) {
    // 基础库已含目标符号：无需额外链接
    if lib_contains_symbol(lib_dir, "libascendcl.so", sym) {
        return;
    }
    // 候选库（按出现频率排序），找第一个导出的
    let candidates = ["libacl_rt.so", "libacl_rt_impl.so", "libascend_common.so"];
    for lib in candidates {
        if lib_contains_symbol(lib_dir, lib, sym) {
            let link_name = lib
                .strip_prefix("lib")
                .unwrap_or(lib)
                .strip_suffix(".so")
                .unwrap_or(lib);
            println!("cargo:rustc-link-lib={link_name}");
            eprintln!("cann-sys: 符号 {sym} 未在 libascendcl.so 中，已追加链接 {lib}");
            return;
        }
    }
    // 兜底：扫描全部 .so。找到任意一个导出的即链接（避免链接器缺符号）。
    if let Ok(entries) = fs::read_dir(lib_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("lib")
                && name.ends_with(".so")
                && lib_contains_symbol(lib_dir, &name, sym)
            {
                let link_name = name
                    .strip_prefix("lib")
                    .unwrap_or(&name)
                    .strip_suffix(".so")
                    .unwrap_or(&name);
                println!("cargo:rustc-link-lib={link_name}");
                eprintln!("cann-sys: 符号 {sym} 未在基础库中，已在 {name} 找到并追加链接");
                return;
            }
        }
    }
    eprintln!("cann-sys: 警告: 未在任何库中找到符号 {sym}，链接可能失败（请检查 lib64 目录）");
}

/// 构建入口。
fn main() {
    println!("cargo::rustc-check-cfg=cfg(cann_sys_ffi)");

    let ffi_enabled = env::var("CARGO_FEATURE_FFI").is_ok();
    // ffi 特性启用即声明 FFI 符号，与 SDK 探测结果无关：
    // 无 SDK 时声明可编译（check 不链接），链接失败留给用户环境。
    if ffi_enabled {
        println!("cargo::rustc-cfg=cann_sys_ffi");
    }

    println!("cargo:rerun-if-env-changed=ASCEND_TOOLKIT_HOME");
    println!("cargo:rerun-if-env-changed=ASCEND_HOME_PATH");
    println!("cargo:rerun-if-env-changed=ASCEND_AICPU_PATH");
    println!("cargo:rerun-if-env-changed=ASCEND_OPP_PATH");
    println!("cargo:rerun-if-env-changed=ASCEND_HOME");
    println!("cargo:rerun-if-env-changed=LD_LIBRARY_PATH");

    match find_cann_sdk() {
        Some((base, include_dir, lib_dir)) => {
            eprintln!("cann-sys: 已找到 CANN SDK: {}", base.display());

            // 符号存在性探测（跨版本漂移门控）
            let mut found = probe_symbols(&include_dir);
            found.sort();
            for sym in &found {
                println!("cargo::rustc-check-cfg=cfg({})", cfg_name(sym));
                println!("cargo::rustc-cfg={}", cfg_name(sym));
            }
            eprintln!(
                "cann-sys: 探测到 {} 个符号，已生成 cann_sys_has_* cfg",
                found.len()
            );

            // 头文件目录变更时重跑探测
            println!(
                "cargo:rerun-if-changed={}",
                include_dir.join("acl").display()
            );

            if ffi_enabled {
                // 搜索/rpath 目录集：主 lib64 + set_env.sh LD_LIBRARY_PATH 结构（插件/op_tiling/aml/driver）+ 用户 LD_LIBRARY_PATH
                let search_dirs = lib_search_dirs(&base, &lib_dir);
                for d in &search_dirs {
                    println!("cargo:rustc-link-search=native={}", d.display());
                    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", d.display());
                }
                println!("cargo:rustc-link-lib=ascendcl");
                let devlib = base.join("aarch64-linux").join("devlib");
                if devlib.exists() {
                    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", devlib.display());
                }
                println!("cargo:rustc-link-arg=-Wl,--allow-shlib-undefined");
                // 跨 SDK 版本/架构的符号库归属差异：确保被引用的符号有库可链
                // （如 aclsysGetVersionStr 在部分版本位于 libacl_rt.so，而非 libascendcl.so）
                link_symbol_provenance(&lib_dir, "aclsysGetVersionStr");
                link_symbol_provenance(&lib_dir, "aclsysGetVersionNum");
            } else {
                eprintln!("cann-sys: ffi 特性未启用，仅类型/常量编译（无链接）");
            }
        }
        None => {
            eprintln!();
            eprintln!("cann-sys: 警告: 未找到 CANN SDK——以无 SDK 模式构建（默认特性）。");
            if ffi_enabled {
                eprintln!(
                    "cann-sys: ffi 特性已启用；找到 SDK 后请设置 ASCEND_TOOLKIT_HOME 并重新构建以完成链接。"
                );
            } else {
                eprintln!(
                    "cann-sys: 若需链接 libascendcl，请设置 ASCEND_TOOLKIT_HOME 并启用 --features ffi。"
                );
            }
            eprintln!("已搜索路径:");
            for candidate in sdk_candidates() {
                eprintln!("  - {}", candidate.display());
            }
            eprintln!();
        }
    }
}
