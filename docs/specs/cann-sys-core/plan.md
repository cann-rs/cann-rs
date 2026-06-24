# cann-sys-core: Plan（技术方案）

## 1. 文件变更清单

```
cann-sys/
├── build.rs                        [NEW]  SDK 发现 + 链接指令
├── Cargo.toml                      [EDIT] 声明 build = "build.rs"
├── src/
│   ├── lib.rs                      [EDIT] 模块入口，重新导出
│   ├── acl.rs                      [NEW]  aclInit / aclFinalize（预留，暂不实现）
│   ├── acl_rt.rs                   [NEW]  aclrtGetVersion / aclsysGetVersionStr / aclsysGetVersionNum
│   ├── acl_base_rt.rs              [NEW]  类型定义（aclError, aclDataType 等）
│   └── error_codes.rs             [NEW]  错误码常量

cann/
├── Cargo.toml                      [EDIT] 依赖 cann-sys
├── src/
│   ├── lib.rs                      [EDIT] 模块入口
│   ├── error.rs                    [NEW]  Error 类型定义
│   └── version.rs                  [NEW]  Version 安全封装

bin-demo/  (或 cann/src/main.rs)
└── main.rs                         [NEW]  打印当前 CANN 版本
```

## 2. build.rs SDK 发现策略

### 优先级顺序

```
1. $ASCEND_TOOLKIT_HOME     // 工具包根路径
2. $ASCEND_HOME             // 兼容旧环境变量
3. ~/Ascend/ascend-toolkit/latest  // 默认安装位置
4. /usr/local/Ascend/latest        // 系统级安装
```

### 发现流程（伪代码）

```
fn find_cann_sdk() -> PathBuf {
    for candidate in [env("ASCEND_TOOLKIT_HOME"), env("ASCEND_HOME"),
                      "~/Ascend/ascend-toolkit/latest",
                      "/usr/local/Ascend/latest"] {
        let base = resolve_symlink(candidate);
        let platform = detect_platform();  // "aarch64-linux" | "x86_64-linux"
        let include = base.join(platform).join("include");
        let lib = base.join(platform).join("lib64");
        if include.join("acl").join("acl_rt.h").exists() && lib.join("libascendcl.so").exists() {
            return (include, lib);
        }
    }
    panic!("CANN SDK not found. Set ASCEND_TOOLKIT_HOME or install CANN.");
}
```

### 链接指令

```rust
fn main() {
    let (include_dir, lib_dir) = find_cann_sdk();

    // 通过 cargo 指令通知 rustc
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=ascendcl");
    println!("cargo:include={}", include_dir.display());  // bindgen 用

    // 重编译触发（SDK 变更时）
    println!("cargo:rerun-if-env-changed=ASCEND_TOOLKIT_HOME");
    println!("cargo:rerun-if-env-changed=ASCEND_HOME");
}
```

### 平台检测

```rust
fn detect_platform() -> &'static str {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("aarch64", "linux") => "aarch64-linux",
        ("x86_64", "linux")  => "x86_64-linux",
        _ => unimplemented!("only Linux (aarch64/x86_64) supported"),
    }
}
```

## 3. FFI 声明策略

### 选择方案：手动声明

本次最小功能（版本查询）只涉及 3 个函数 + 少量类型，手动 `extern "C"` 声明比 bindgen 更可控、构建更快。

### 需声明的 C API

| C 函数 / 类型 | 所在头文件 | 导入位置 |
|---|---|---|
| `int32_t aclError` | `acl_base_rt.h` | `acl_base_rt.rs` |
| `aclError aclsysGetVersionStr(char *pkgName, char *versionStr)` | `acl_rt.h` | `acl_rt.rs` |
| `aclError aclsysGetVersionNum(char *pkgName, int32_t *versionNum)` | `acl_rt.h` | `acl_rt.rs` |
| `aclError aclrtGetVersion(int32_t *major, int32_t *minor, int32_t *patch)` | `acl_rt.h` | `acl_rt.rs` |
| `ACL_PKG_VERSION_MAX_SIZE` (= 128) | `acl_rt.h` | `acl_rt.rs` |
| `ACL_SUCCESS` (= 0) 及常用错误码 | `acl_base_rt.h` | `error_codes.rs` |
| `aclCANNPackageName` 枚举 | `acl_rt.h` | `acl_rt.rs` |

### 缓冲区规范

`aclsysGetVersionStr` 的 `versionStr` 缓冲区由调用者分配：
- `ACL_PKG_VERSION_MAX_SIZE` = 128 字节足够存放任何 CANN 版本字符串（当前 `"9.0.0"` 仅 5 字节）

## 4. cann-sys 模块导出结构

```rust
// lib.rs
pub mod acl_base_rt;
pub mod error_codes;
pub mod acl_rt;

// 重新导出最常用类型
pub use acl_base_rt::*;
pub use error_codes::*;
pub use acl_rt::*;
```

```rust
// acl_base_rt.rs
pub type aclError = i32;
// 仅类型定义，不包含函数
```

```rust
// error_codes.rs
pub const ACL_SUCCESS: aclError = 0;
pub const ACL_ERROR_INVALID_FILE: aclError = 100003;
// ... 仅本次用到的错误码
```

```rust
// acl_rt.rs
extern "C" {
    // // SAFETY: pkgName 必须是 NUL 结尾 C 字符串，versionStr 缓冲区至少 128 字节
    pub fn aclsysGetVersionStr(pkgName: *const c_char, versionStr: *mut c_char) -> aclError;
    // // SAFETY: pkgName 必须是 NUL 结尾 C 字符串，versionNum 指向有效 int32_t
    pub fn aclsysGetVersionNum(pkgName: *const c_char, versionNum: *mut i32) -> aclError;
    // // SAFETY: major、minor、patch 指向有效 int32_t
    pub fn aclrtGetVersion(majorVersion: *mut i32, minorVersion: *mut i32, patchVersion: *mut i32) -> aclError;
}
```

## 5. cann 层安全封装接口设计

```rust
// cann/src/version.rs
pub struct Version;

impl Version {
    /// 获取 CANN 版本字符串，如 "9.0.0"
    pub fn str() -> Result<String, Error>;
    /// 获取 CANN 版本数值
    pub fn num() -> Result<i32, Error>;
}
```

```rust
// cann/src/error.rs
#[derive(Debug)]
pub struct Error {
    pub code: aclError,
    pub message: String,
}
impl std::error::Error for Error {}
impl std::fmt::Display for Error { /* ... */ }
```

封装要点：
- `Version::str()` 内部分配 128 字节缓冲区，调用 `aclsysGetVersionStr("CANN", buf)`
- `Version::num()` 调用 `aclsysGetVersionNum("CANN", &mut num)`
- 错误时将 `aclError` 码映射到 `Error` 类型（当前仅映射 `ACL_SUCCESS` 和 `ACL_ERROR_INVALID_FILE`，其余归入 `Unknown`）

## 6. 测试策略

| 测试层级 | 测试内容 | 需要硬件 | 方式 |
|---|---|---|---|
| cann-sys unit | 编译/link 验证 | 否 | `#[test]` 链接测试（只在有 SDK 时运行） |
| cann-sys unit | FFI 参数正确传递 | 否 | 用 mock 函数测试指针传递（可选） |
| cann integration | 版本字符串格式正确 | 是 | `#[test]` + `#[cfg(any(target_arch = "aarch64", ...))]` |
| cann integration | 版本数值语义正确 | 是 | `assert!(num >= 80000000)` |

## 7. 风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| 环境变量未设置 | 构建失败 | build.rs 遍历多个候选路径，最后打印清晰指引 |
| `aclsysGetVersionStr` 需要 `aclInit` | 函数调用失败 | 需验证（CANN 文档未注明需要 init，nm 确认符号存在） |
| 缓冲区大小不足 | 缓冲区溢出 | 使用 `ACL_PKG_VERSION_MAX_SIZE`（128）确保安全 |
| 无 Ascend NPU 设备 | 部分测试不可用 | 版本查询不依赖设备，纯 API 调用，应在无设备时也工作 |
