# cann-sys-core: Tasks（任务清单）

共 9 个原子任务，按依赖关系排序。每个任务产出可独立验证。

---

### T1. cann-sys build.rs：SDK 发现与链接

**输入**：CANN SDK 安装目录结构（`~/Ascend/cann-9.0.0`）

**任务**：
- 文件：`cann-sys/build.rs`
- 实现 SDK 候选路径遍历（`ASCEND_TOOLKIT_HOME` → `ASCEND_HOME` → 默认路径）
- 实现平台检测（`aarch64-linux` / `x86_64-linux`）
- 验证头文件（`acl/acl_rt.h`）和库文件（`libascendcl.so`）存在
- 输出 `cargo:rustc-link-search` 和 `cargo:rustc-link-lib` 指令
- 设置 `rerun-if-env-changed` 触发器
- 无 SDK 时给出清晰错误信息

**验证**：
- [ ] 有 SDK 时 `print!("{}", include_dir)` 显示正确路径
- [ ] 无 SDK 时编译失败，输出包含 `ASCEND_TOOLKIT_HOME` 指引

**依赖**：无

---

### T2. cann-sys error_codes：错误码常量

**输入**：`acl_base_rt.h` 中 `ACL_SUCCESS`、`ACL_ERROR_INVALID_FILE` 等定义

**任务**：
- 文件：`cann-sys/src/error_codes.rs`
- 声明 `ACL_SUCCESS`（= 0）
- 声明 `ACL_ERROR_INVALID_FILE`（= 100003）
- 其他本次 feature 用到的错误码

**验证**：
- [ ] `cassert_eq!(ACL_SUCCESS, 0)`
- [ ] `cargo doc` 生成文档

**依赖**：无

---

### T3. cann-sys acl_base_rt：基础类型定义

**输入**：`acl_base_rt.h` 中 `aclError` typedef

**任务**：
- 文件：`cann-sys/src/acl_base_rt.rs`
- 声明 `pub type aclError = c_int;`
- 声明 `ACL_PKG_VERSION_MAX_SIZE`（= 128）
- 声明 `ACL_PKG_VERSION_PARTS_MAX_SIZE`（= 64）

**验证**：
- [ ] `aclError` 类型可通过 `cfg(test)` 编译

**依赖**：无

---

### T4. cann-sys acl_rt：版本查询 FFI 函数

**输入**：`acl_rt.h` 中 `aclsysGetVersionStr`、`aclsysGetVersionNum`、`aclrtGetVersion` 的函数签名

**任务**：
- 文件：`cann-sys/src/acl_rt.rs`
- 导入 `acl_base_rt` 和 `error_codes`
- 声明 `extern "C" { ... }` 块
  - `aclsysGetVersionStr(pkgName: *const c_char, versionStr: *mut c_char) -> aclError`
  - `aclsysGetVersionNum(pkgName: *const c_char, versionNum: *mut i32) -> aclError`
  - `aclrtGetVersion(major: *mut i32, minor: *mut i32, patch: *mut i32) -> aclError`
- 每个 unsafe 函数附带 `// SAFETY:` 注释

**验证**：
- [ ] `cargo check` 通过
- [ ] `cargo doc` 显示函数签名和 safety 说明

**依赖**：T2, T3

---

### T5. cann-sys lib.rs：模块入口

**输入**：T2, T3, T4 的模块

**任务**：
- 文件：`cann-sys/src/lib.rs`
- 声明 `pub mod acl_base_rt`
- 声明 `pub mod error_codes`
- 声明 `pub mod acl_rt`
- 重新导出：`pub use acl_base_rt::*;` `pub use error_codes::*;` `pub use acl_rt::*;`

**验证**：
- [ ] `cargo build` 通过
- [ ] 外部 crate 可通过 `use cann_sys::aclsysGetVersionStr;` 调用

**依赖**：T2, T3, T4

---

### T6. cann-sys Cargo.toml：build.rs 声明

**输入**：无（仅配置变更）

**任务**：
- 文件：`cann-sys/Cargo.toml`
- 添加 `build = "build.rs"`
- 确认 `[dependencies]` 为空（零依赖约束）

**验证**：
- [ ] `cargo build` 自动执行 `build.rs`

**依赖**：T1（需 build.rs 就位）

---

### T7. cann error.rs：错误类型

**输入**：`aclsysGetVersionStr` 返回的 `aclError`

**任务**：
- 文件：`cann/src/error.rs`
- 定义 `Error` 结构体（`code: aclError`, `message: String`）
- 实现 `std::error::Error` trait
- 实现 `std::fmt::Display` trait
- 实现 `From<aclError>` 转换（`ACL_ERROR_INVALID_FILE` → 带消息的错误，其他 → Unknown）

**验证**：
- [ ] `let e = Error::from(ACL_ERROR_INVALID_FILE);`
- [ ] `format!("{}", e)` 不 panic

**依赖**：T5（需要 `aclError` 类型）

---

### T8. cann version.rs：安全封装

**输入**：`aclsysGetVersionStr` / `aclsysGetVersionNum` FFI 函数

**任务**：
- 文件：`cann/src/version.rs`
- 定义 `pub struct Version;`
- `Version::str() -> Result<String, Error>`:
  - 分配 128 字节缓冲区（`[0u8; 128]`）
  - 调用 `aclsysGetVersionStr(c_str!("CANN"), buf)`
  - 检查返回值，失败返回 `Err`
  - 成功将缓冲区转为 `String` 返回
- `Version::num() -> Result<i32, Error>`:
  - 调用 `aclsysGetVersionNum(c_str!("CANN"), &mut num)`
  - 检查返回值，失败返回 `Err`
  - 成功返回数值
- 使用 `std::ffi::CStr` 安全处理 C 字符串边界

**验证**：
- [ ] `Version::str()` 在有 SDK 时返回 `Ok("9.0.0")`
- [ ] `Version::num()` 在有 SDK 时返回 `Ok(90000000)`（计算公式：`9*10000000 + 0*100000 + 0*1000`）

**依赖**：T5, T7

---

### T9. cann lib.rs + Demo：集成入口

**输入**：T8 的 Version API

**任务**：
- 文件：`cann/src/lib.rs` — 声明 `pub mod error`、`pub mod version`
- 文件：`cann/src/main.rs` — 调用 `Version::str()` / `Version::num()` 并 `println!`

**验证**：
- [ ] `cargo run -p cann` 输出类似 `CANN version: 9.0.0 (num: 90000000)`

**依赖**：T8

---

## 任务执行顺序

```
T1 ──→ T6 ──→ [T2, T3] ──→ T4 ──→ T5 ──→ T7 ──→ T8 ──→ T9
                  ↑
            (可并行,无依赖)
```

**组间并发**：T2(T3) 与 T1 可同时执行（不同文件，无冲突）
**组内串行**：T4→T5 必须串行（T4 产出被 T5 导入）
