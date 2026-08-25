//! `cann-sys` 构建脚本。
//!
//! 负责：
//! - 自动发现 CANN SDK 安装路径
//! - 启用 `ffi` 特性时链接 `libascendcl.so`
//! - 非 `ffi` 构建时降级：仅探测并打印，不阻断编译（0001 Task 1）
//! - 符号存在性探测：生成 `cann_sys_has_*` cfg 供跨版本门控

use std::path::{Path, PathBuf};
use std::process::Command;
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
    // L1: aclTensor / aclnn 算子 / GE 图引擎
    "aclCreateTensor",
    "aclCreateScalar",
    "aclCreateTensorList",
    "aclDestroyTensor",
    "aclGetViewShape",
    "aclGetStorageShape",
    "aclGetViewStrides",
    "aclGetViewOffset",
    "aclGetFormat",
    "aclGetDataType",
    "aclGetTensorListSize",
    "aclnnMatmulGetWorkspaceSize",
    "aclnnMatmul",
    "aclnnSoftmaxGetWorkspaceSize",
    "aclnnSoftmax",
    "aclnnRmsNormGetWorkspaceSize",
    "aclnnRmsNorm",
    "aclgrphParseONNX",
    "aclgrphParseONNXFromMem",
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

/// 判断某个 `.so` 是否**导出**目标符号（ELF64 小端 `.dynsym` 解析）。
///
/// 零依赖实现（不依赖 `nm`）。注意：不能用"字符串包含"判断——库的内部字符串/
/// 依赖引用也包含符号名，会误报；必须解析动态符号表确认导出。
fn lib_contains_symbol(lib_dir: &Path, lib_file: &str, sym: &str) -> bool {
    let Ok(bytes) = fs::read(lib_dir.join(lib_file)) else {
        return false;
    };
    if bytes.len() < 64 || &bytes[0..4] != b"\x7fELF" {
        return false;
    }
    // ELF64 header: e_shoff@40(u64) e_shentsize@58(u16) e_shnum@60(u16) e_shstrndx@62(u16)
    let shoff = read_u64(&bytes, 40);
    let shentsize = read_u16(&bytes, 58) as usize;
    let shnum = read_u16(&bytes, 60) as usize;
    let shstrndx = read_u16(&bytes, 62) as usize;
    if shentsize < 64 || shnum == 0 || shstrndx >= shnum {
        return false;
    }
    let section = |i: usize| -> Option<&[u8]> {
        let off = (shoff as usize).checked_add(i * shentsize)?;
        bytes.get(off..off + shentsize)
    };
    // section header fields: sh_name@0(u32) sh_type@4(u32) sh_offset@24(u64) sh_size@32(u64)
    // 注意：section() 返回的是 section header（64B 元数据），字符串表数据
    // 需按 header 中的 sh_offset/sh_size 在文件内容上再取一次。
    let Some(shstr_hdr) = section(shstrndx) else {
        return false;
    };
    let shstr_off = read_u64(shstr_hdr, 24) as usize;
    let shstr_size = read_u64(shstr_hdr, 32) as usize;
    let Some(shstr) = bytes.get(shstr_off..shstr_off + shstr_size) else {
        return false;
    };
    let shstr_name = |off: u32| -> Option<&[u8]> {
        let s = shstr.get(off as usize..)?;
        let end = s.iter().position(|&b| b == 0).unwrap_or(s.len());
        Some(&s[..end])
    };
    let mut dynsym: Option<&[u8]> = None;
    let mut dynstr: Option<&[u8]> = None;
    for i in 0..shnum {
        let Some(sh) = section(i) else {
            continue;
        };
        let Some(name) = shstr_name(read_u32(sh, 0)) else {
            continue;
        };
        let stype = read_u32(sh, 4);
        let off = read_u64(sh, 24) as usize;
        let size = read_u64(sh, 32) as usize;
        if name == b".dynsym" && stype == 11 {
            dynsym = bytes.get(off..off + size);
        }
        if name == b".dynstr" && stype == 3 {
            dynstr = bytes.get(off..off + size);
        }
    }
    let Some(ents) = dynsym else { return false };
    let Some(strs) = dynstr else { return false };
    // Elf64_Sym: st_name@0(u32) st_info@4(u8) st_other@5(u8) st_shndx@6(u16)
    // shndx == 0 (SHN_UNDEF) 表示"未定义引用"——库只引用不提供该符号，不算导出。
    for ent in ents.chunks(24) {
        if ent.len() < 24 {
            continue;
        }
        if read_u16(ent, 6) == 0 {
            continue;
        }
        let st_name = read_u32(ent, 0) as usize;
        if st_name >= strs.len() {
            continue;
        }
        let rest = &strs[st_name..];
        let end = rest.iter().position(|&b| b == 0).unwrap_or(rest.len());
        if &rest[..end] == sym.as_bytes() {
            return true;
        }
    }
    false
}

/// ELF 小端定长整数读取（切片长度已由调用方保证）。
fn read_u64(bytes: &[u8], off: usize) -> u64 {
    u64::from_le_bytes(bytes[off..off + 8].try_into().unwrap())
}
fn read_u32(bytes: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap())
}
fn read_u16(bytes: &[u8], off: usize) -> u16 {
    u16::from_le_bytes(bytes[off..off + 2].try_into().unwrap())
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
    let candidates = [
        "libacl_rt.so",
        "libacl_rt_impl.so",
        "libascend_common.so",
        "libopapi.so",
        "libnnopbase.so",
        "libaclnn.so",
        "libfmk_onnx_parser.so",
    ];
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

/// 编译 GE 图引擎 C++ shim（`src/ge_shim.cc`）为静态库 `libge_shim.a` 并链接。
///
/// 背景（L1-3）：GE 的 `aclgrph*` 是 **C++ API**（`include/parser/onnx_parser.h` /
/// `include/ge/ge_ir_build.h`），Rust 无法直接 `extern "C"` 声明，需一个 C++ 桥接层。
/// 流程：`cc -std=c++17 -fPIC -c` → `ar crs libge_shim.a` → 静态链接，
/// 并追加 `-lstdc++`（shim 使用 std::map/string/shared_ptr 等 C++ 标准库）。
/// include 需要两级：`<include_dir>`（parser/、graph/、ge/ 头文件）与其父目录。
/// 仅 ffi 档执行（与 libascendcl 同一档位）；失败仅警告不阻断
/// （链接期缺 `cann_grph_*` 符号会以显式错误暴露）。
fn build_ge_shim(include_dir: &Path) {
    let manifest = match env::var("CARGO_MANIFEST_DIR") {
        Ok(d) => PathBuf::from(d),
        Err(_) => {
            eprintln!("cann-sys: 警告: 无 CARGO_MANIFEST_DIR，跳过 GE shim 编译");
            return;
        }
    };
    let shim_src = manifest.join("src").join("ge_shim.cc");
    let out_dir = match env::var("OUT_DIR") {
        Ok(d) => PathBuf::from(d),
        Err(_) => {
            eprintln!("cann-sys: 警告: 无 OUT_DIR，跳过 GE shim 编译");
            return;
        }
    };
    let obj = out_dir.join("ge_shim.o");
    let archive = out_dir.join("libge_shim.a");
    // include 两级：<include_dir>（parser/、graph/、ge/）与其父目录（external/ 等相对包含）
    let include_root = include_dir.parent().unwrap_or(include_dir);
    let cc = env::var("CC").unwrap_or_else(|_| "cc".to_string());
    let ar = env::var("AR").unwrap_or_else(|_| "ar".to_string());

    let compiled = match Command::new(&cc)
        .arg("-std=c++17")
        .arg("-fPIC")
        .arg("-c")
        .arg(&shim_src)
        .arg("-I")
        .arg(include_dir)
        .arg("-I")
        .arg(include_root)
        .arg("-o")
        .arg(&obj)
        .output()
    {
        Ok(out) if out.status.success() => true,
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            eprintln!("cann-sys: 警告: GE shim 编译失败，跳过（链接期将缺 cann_grph_* 符号）");
            eprintln!("cann-sys: cc 输出:\n{stderr}");
            println!("cargo:warning=GE shim 编译失败，链接期将缺 cann_grph_* 符号");
            false
        }
        Err(e) => {
            eprintln!("cann-sys: 警告: 无法启动 C++ 编译器 {cc}（{e}），跳过 GE shim 编译");
            false
        }
    };
    if !compiled {
        return;
    }

    let archived = match Command::new(&ar)
        .arg("crs")
        .arg(&archive)
        .arg(&obj)
        .status()
    {
        Ok(s) => s.success(),
        Err(e) => {
            eprintln!("cann-sys: 警告: 无法启动 ar（{e}），跳过 GE shim 打包");
            false
        }
    };
    if !archived {
        return;
    }

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=ge_shim");
    // shim 使用 C++ 标准库（std::map/std::string/std::shared_ptr），须显式链 libstdc++
    println!("cargo:rustc-link-lib=dylib=stdc++");
    // ge_shim.cc 变更时重跑 build.rs（头文件目录变更已由 rerun-if-changed=include 覆盖）
    println!("cargo:rerun-if-changed={}", shim_src.display());
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

                // ---- GE 图引擎 C++ shim（L1-3）----
                // GE 的 aclgrph* 是 C++ API，经 src/ge_shim.cc 桥接为 extern "C"，
                // 编译为静态库 libge_shim.a 并链接（含 -lstdc++）。
                build_ge_shim(&include_dir);
                // 符号探测：跨 SDK 版本/架构的 GE 符号库归属差异由
                // link_symbol_provenance 兜底（基础库未含时自动补链）。
                link_symbol_provenance(&lib_dir, "aclgrphParseONNX");
                link_symbol_provenance(&lib_dir, "aclgrphParseONNXFromMem");
                link_symbol_provenance(&lib_dir, "aclCreateTensor");
                link_symbol_provenance(&lib_dir, "aclnnMatmul");
                // 显式补链接库名（仅当 .so 存在于 lib64 时，兼容跨版本差异；nm -D 验证）：
                // - libfmk_onnx_parser.so：导出 aclgrphParseONNX / aclgrphParseONNXFromMem
                // - libge_compiler.so：导出 aclgrphBuildModel / aclgrphSaveModel
                // - libge_common.so：ge::Graph 等 GE 公共符号所在库（部分版本兜底）
                for ge_lib in ["fmk_onnx_parser", "ge_compiler", "ge_common"] {
                    if lib_dir.join(format!("lib{ge_lib}.so")).exists() {
                        println!("cargo:rustc-link-lib={ge_lib}");
                    }
                }
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
