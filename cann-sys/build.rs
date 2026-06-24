use std::path::PathBuf;
use std::{env, fs};

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
    eprintln!("error: CANN SDK not found.");
    eprintln!();
    eprintln!(
        "help: set ASCEND_TOOLKIT_HOME environment variable, or install CANN to the default location."
    );
    eprintln!("help: searched paths:");
    for candidate in candidates.iter().flatten() {
        eprintln!("  - {}", candidate.display());
    }
    eprintln!();
    std::process::exit(1);
}

fn extract_cann_version(include_dir: &std::path::Path) -> (String, i64) {
    let version_h = include_dir.join("version").join("cann_version.h");
    if version_h.exists()
        && let Ok(content) = fs::read_to_string(&version_h)
    {
        for line in content.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("#define CANN_VERSION_STR ") {
                let ver = val.trim_matches('"').trim().to_string();
                let num = parse_version_num(&ver);
                return (ver, num);
            }
        }
    }
    eprintln!(
        "warning: {} not found or parseable; falling back to path name",
        version_h.display()
    );
    let name = include_dir
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if let Some(ver) = extract_version_from_name(name) {
        let num = parse_version_num(&ver);
        return (ver, num);
    }
    eprintln!("warning: could not determine CANN version");
    ("0.0.0".to_string(), 0)
}

fn extract_version_from_name(name: &str) -> Option<String> {
    let bytes = name.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            let mut dots = 0u8;
            while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                if bytes[i] == b'.' {
                    dots += 1;
                }
                if dots > 2 {
                    break;
                }
                i += 1;
            }
            if dots == 2 {
                let ver = &name[start..i];
                let parts: Vec<&str> = ver.split('.').collect();
                if parts.len() == 3
                    && !parts[0].is_empty()
                    && !parts[1].is_empty()
                    && !parts[2].is_empty()
                {
                    return Some(ver.to_string());
                }
            }
        } else {
            i += 1;
        }
    }
    None
}

fn parse_version_num(ver: &str) -> i64 {
    let parts: Vec<&str> = ver.split('.').collect();
    if parts.len() == 3 {
        let major: i64 = parts[0].parse().unwrap_or(0);
        let minor: i64 = parts[1].parse().unwrap_or(0);
        let patch: i64 = parts[2].parse().unwrap_or(0);
        major * 10_000_000 + minor * 100_000 + patch * 1000
    } else {
        0
    }
}

fn main() {
    // Allow the custom cfg flag used for conditional FFI compilation.
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

    let (ver_str, ver_num) = extract_cann_version(&include_dir);
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(
        out.join("version_info.rs"),
        format!(
            "pub const CANN_VERSION_STR: &str = \"{}\";\npub const CANN_VERSION_NUM: i64 = {};\n",
            ver_str, ver_num
        ),
    )
    .expect("failed to write version_info.rs");

    println!("cargo:rerun-if-env-changed=ASCEND_TOOLKIT_HOME");
    println!("cargo:rerun-if-env-changed=ASCEND_HOME_PATH");
    println!("cargo:rerun-if-env-changed=ASCEND_HOME");
    println!(
        "cargo:rerun-if-changed={}",
        include_dir.join("version").join("cann_version.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        include_dir.join("acl").join("acl_rt.h").display()
    );
}
