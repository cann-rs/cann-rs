# cann-sys-core: Plan（技术方案）

## 1. 文件变更清单（最终版）

```
cann-sys/
├── build.rs                        [NEW]  SDK 发现 + 版本提取 + 链接指令(FFI 可选)
├── Cargo.toml                      [EDIT] 声明 build = "build.rs", [features] ffi = []
├── src/
│   ├── lib.rs                      [EDIT] 模块入口 + include!(version_info.rs)
│   ├── acl_rt.rs                   [NEW]  常量/枚举 + #[cfg(cann_sys_ffi)] extern "C" 块
│   ├── acl_base_rt.rs              [NEW]  aclError 类型 + 错误码常量

cann/
├── Cargo.toml                      [EDIT] 依赖 cann-sys (features = ["ffi"])
├── src/
│   ├── lib.rs                      [EDIT] mod error; mod version;
│   ├── main.rs                     [NEW]  demo: 打印 CANN 版本
│   ├── error.rs                    [NEW]  Error { code, message }
│   └── version.rs                  [NEW]  Version { str(), num() } — FFI 优先
```

## 2. build.rs SDK 发现策略

### 关键发现（来自 set_env.sh 分析）

`set_env.sh` 明确设置 `ASCEND_TOOLKIT_HOME=$version_dirpath`，且 CANN 安装目录在根级提供 symlink：
```
$ASCEND_TOOLKIT_HOME/
├── include/  → aarch64-linux/include/  (symlink)
├── lib64/    → aarch64-linux/lib64/    (symlink)
├── include/version/cann_version.h       ← 版本宏定义
```
因此 **build.rs 不需要架构检测来构造路径**，直接用 `$ASCEND_TOOLKIT_HOME/include` 和 `$ASCEND_TOOLKIT_HOME/lib64` 即可。

### 优先级顺序

```
1. $ASCEND_TOOLKIT_HOME     // set_env.sh 设置的主变量
2. $ASCEND_HOME_PATH        // 兼容变量
3. $ASCEND_HOME             // 旧版本兼容
4. ~/Ascend/cann            // 默认安装位置（symlink: cann → cann-X.Y.Z）
5. /usr/local/Ascend        // 系统级安装
```

### 版本号来源

CANN SDK 在 `include/version/cann_version.h` 中定义版本宏：

```c
#define CANN_VERSION_STR "9.0.0"
#define CANN_MAJOR 9
#define CANN_MINOR 0
#define CANN_PATCH 0
#define CANN_VERSION_NUM ((9 * 10000000) + (0 * 100000) + (0 * 1000))
```

build.rs 解析该文件，生成 Rust 常量到 `$OUT_DIR/version_info.rs`：

```rust
pub const CANN_VERSION_STR: &str = "9.0.0";
pub const CANN_VERSION_NUM: i64 = 90000000;
```

cann-sys 通过 `include!(concat!(env!("OUT_DIR"), "/version_info.rs"))` 引入。

### 链接指令

FFI 链接仅在 `features = ["ffi"]` 启用时生效（通过 `CARGO_FEATURE_FFI` 环境变量检测）：

```rust
fn main() {
    let (sdk_base, include_dir, lib_dir) = find_cann_sdk();
    let ffi_enabled = env::var("CARGO_FEATURE_FFI").is_ok();

    if ffi_enabled {
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=ascendcl");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
        // 无 NPU 驱动时通过 devlib 提供 libascend_hal.so
        let devlib = sdk_base.join("aarch64-linux").join("devlib");
        if devlib.exists() {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", devlib.display());
        }
        println!("cargo:rustc-link-arg=-Wl,--allow-shlib-undefined");
        println!("cargo:rustc-cfg=cann_sys_ffi");  // 条件编译标记
    }
}
```

## 3. FFI 声明策略

### 方案：手动声明 + feature-gated

本次最小功能（版本查询）只涉及 3 个函数 + 少量类型，手动 `extern "C"` 声明。

**FFI 声明仅在 `cann_sys_ffi` cfg 下编译**，默认不启用。这使得 `cann-sys` 的类型/常量测试无需链接 libascendcl 即可运行。

### 声明的 C API

| C 函数 / 类型 | 所在头文件 | Rust 位置 | 条件 |
|---|---|---|---|
| `typedef int aclError` | `acl_base_rt.h` | `acl_base_rt.rs` | 始终 |
| `aclError aclsysGetVersionStr(char*, char*)` | `acl_rt.h` | `acl_rt.rs` extern | `#[cfg(cann_sys_ffi)]` |
| `aclError aclsysGetVersionNum(char*, int32_t*)` | `acl_rt.h` | `acl_rt.rs` extern | `#[cfg(cann_sys_ffi)]` |
| `aclError aclrtGetVersion(int32_t*, int32_t*, int32_t*)` | `acl_rt.h` | `acl_rt.rs` extern | `#[cfg(cann_sys_ffi)]` |
| `ACL_PKG_VERSION_MAX_SIZE` (= 128) | `acl_rt.h` | `acl_rt.rs` 常量 | 始终 |
| `ACL_PKG_VERSION_PARTS_MAX_SIZE` (= 64) | `acl_rt.h` | `acl_rt.rs` 常量 | 始终 |
| `ACL_SUCCESS, ACL_ERROR_INVALID_FILE` 等 | `acl_base_rt.h` | `acl_base_rt.rs` | 始终 |

### FFI 声明代码

```rust
// acl_rt.rs — 仅 FFI 启用时编译
#[cfg(cann_sys_ffi)]
unsafe extern "C" {
    pub fn aclsysGetVersionStr(pkgName: *const c_char, versionStr: *mut c_char) -> aclError;
    pub fn aclsysGetVersionNum(pkgName: *const c_char, versionNum: *mut i32) -> aclError;
    pub fn aclrtGetVersion(major: *mut i32, minor: *mut i32, patch: *mut i32) -> aclError;
}
```

## 4. cann 层安全封装：Version API（双重来源）

### 设计目标

1. **优先通过 FFI** 调用 `aclsysGetVersionStr`/`aclsysGetVersionNum` 获取运行时版本
2. **FFI 失败时**（如 driver 未安装、无硬件、函数返回错误）自动降级到编译期常量
3. **零中断**：在任何环境下 `Version::str()` / `Version::num()` 均返回 `Ok`

### 实现

```rust
impl Version {
    pub fn str() -> Result<String, Error> {
        // 1. FFI 优先
        match Self::try_ffi_str() {
            Ok(v) => return Ok(v),
            Err(_) => { /* 降级到编译期常量 */ }
        }
        // 2. 编译期常量（从 cann_version.h 提取）
        Ok(cann_sys::CANN_VERSION_STR.to_string())
    }

    pub fn num() -> Result<i64, Error> {
        match Self::try_ffi_num() {
            Ok(n) => return Ok(n as i64),
            Err(_) => { /* 降级 */ }
        }
        Ok(cann_sys::CANN_VERSION_NUM)
    }

    fn try_ffi_str() -> Result<String, Error> {
        let pkg_name = b"CANN\0".as_ptr() as *const c_char;
        let mut buf = [0u8; cann_sys::ACL_PKG_VERSION_MAX_SIZE];
        // SAFETY: pkgName 是有效 C 字符串，缓冲区 ≥128 字节
        let ret = unsafe {
            cann_sys::aclsysGetVersionStr(pkg_name, buf.as_mut_ptr() as *mut c_char)
        };
        if ret != cann_sys::ACL_SUCCESS {
            return Err(Error::from(ret));
        }
        // SAFETY: FFI 成功，缓冲区包含有效 C 字符串
        let c_str = unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) };
        Ok(c_str.to_str().unwrap_or_default().to_string())
    }

    fn try_ffi_num() -> Result<i32, Error> {
        let pkg_name = b"CANN\0".as_ptr() as *const c_char;
        let mut num: i32 = 0;
        let ret = unsafe { cann_sys::aclsysGetVersionNum(pkg_name, &mut num) };
        if ret != cann_sys::ACL_SUCCESS {
            return Err(Error::from(ret));
        }
        Ok(num)
    }
}
```

### 降级条件

| 场景 | FFI 是否成功 | 最终来源 |
|------|-------------|----------|
| 有 CANN SDK + NPU 驱动 | ✅ `ACL_SUCCESS` | FFI 运行时值 |
| 有 CANN SDK，无驱动 | ❌ `ACL_ERROR_INVALID_FILE` | 编译期常量 |
| 无 CANN SDK | 构建失败 — `build.rs exit(1)` | — |

## 5. Error 类型

```rust
#[derive(Debug)]
pub struct Error {
    pub code: aclError,    // 原始 ACL 错误码 (i32)
    pub message: String,
}

impl From<aclError> for Error {
    fn from(code: aclError) -> Self {
        let message = match code {
            ACL_ERROR_INVALID_FILE => "Invalid file".to_string(),
            _ => format!("Unknown CANN error code: {}", code),
        };
        Error { code, message }
    }
}
```

## 6. 测试策略

| 测试层级 | 文件 | 测试内容 | 条件 |
|---------|------|---------|------|
| cann-sys unit | `acl_base_rt.rs` | 类型定义、错误码常量 | 始终编译 |
| cann-sys unit | `acl_rt.rs` | FFI 链接测试 | `#[cfg(cann_sys_ffi)]` |
| cann unit | `version.rs` | `Version::str/num` 返回值验证 | 始终编译 |
| cann unit | `error.rs` | Error 类型构造与 Display | 始终编译 |

### SDK 环境检测

- 无 SDK：build.rs 失败，不产生任何测试二进制
- 有 SDK 无驱动：FFI 返回错误，降级到编译期常量，测试验证常量值
- 有 SDK 有驱动：FFI 成功，测试验证运行时值

### 运行测试

```bash
# 默认模式（类型/常量测试，无需 FFI 链接）
cargo test -p cann-sys    # 3 个测试
cargo test -p cann        # 11 个测试

# FFI 模式（启用 extern 声明 + 链接）
cargo test -p cann-sys --features ffi    # 需要完整 CANN 运行时
```

## 7. 版本号编码格式

```
CANN_VERSION_NUM = MAJOR × 10 000 000 + MINOR × 100 000 + PATCH × 1000
```

示例：`CANN 9.0.0` → `9 × 10 000 000 + 0 × 100 000 + 0 × 1000 = 90 000 000`

## 8. 风险与缓解（更新版）

| 风险 | 影响 | 缓解 |
|------|------|------|
| 环境变量未设置 | 构建失败 | build.rs 遍历多个候选路径，最后打印清晰指引 |
| `aclsysGetVersionStr` 无驱动时失败 | FFI 返回 `ACL_ERROR_INVALID_FILE` | 自动降级到编译期常量 |
| 无 NPU 驱动导致库加载失败 | 二进制运行错误 | rpath 添加 devlib 路径+`--allow-shlib-undefined` |
| 缓冲区大小不足 | 缓冲区溢出 | 使用 `ACL_PKG_VERSION_MAX_SIZE`（128）确保安全 |
