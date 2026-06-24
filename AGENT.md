# cann-rs Agent 开发规范

本文件定义 AI Agent 执行 cann-rs 项目开发任务时必须遵守的软件工程标准。所有规范基于 Rust 生态主流实践，覆盖从编码到发布的全生命周期。

---

## 1. 代码格式化

所有 Rust 代码必须通过以下工具链检查：

| 工具 | 命令 | 用途 |
|------|------|------|
| rustfmt | `cargo fmt --all -- --check` | 代码格式化 |
| clippy | `cargo clippy --workspace -- -D warnings` | 静态分析 |
| rustdoc | `cargo doc --workspace --no-deps` | 文档生成 |

**格式化规则**：
- 使用默认 `rustfmt.toml` 配置（4 空格缩进，行宽 100）
- 导入按 `std` → `external crate` → `crate` 分组排列
- 使用 `rustfmt` 的 `imports_granularity = "Crate"` 合并导入

---

## 2. 命名规范

### 2.1 基础规则

| 类型 | 风格 | 示例 |
|------|------|------|
| 模块 | snake_case | `acl_rt`, `error_codes` |
| 结构体 | CamelCase | `Version`, `Context`, `Stream` |
| 枚举 | CamelCase | `Error`, `DataType`, `Format` |
| 枚举变体 | CamelCase（非 SCREAMING_CASE） | `Error::InvalidFile`, `DataType::Float` |
| 函数 | snake_case | `get_version`, `create_context` |
| 常量 | SCREAMING_SNAKE_CASE | `ACL_SUCCESS`, `MAX_VERSION_SIZE` |
| 类型别名 | CamelCase（遵循 C 原名） | `aclError`, `aclFloat16` |
| 生命周期 | 小写单字母 | `'a`, `'ctx` |
| Feature flags | kebab-case | `hw_tests`, `mock` |

### 2.2 FFI 特殊规则（cann-sys 层）

| 规则 | 说明 |
|------|------|
| C 函数名 | 保留原始名称，不做任何转换 |
| C 类型名 | 使用 `pub type` 保持一致（如 `aclError = c_int`） |
| C 常量名 | 保留原始名称（如 `ACL_SUCCESS`） |
| C 枚举名 | 使用 `#[repr(C)]`，保持原始值 |

---

## 3. 错误处理

### 3.1 分层策略

| 层 | 策略 | 示例 |
|----|------|------|
| cann-sys | 直接返回 C 错误码 `aclError` | `pub fn get_version() -> aclError` |
| cann | 封装为 `Result<T, Error>` | `pub fn get_version() -> Result<String, Error>` |

### 3.2 Error 类型设计

```rust
// cann/src/error.rs
#[derive(Debug)]
pub struct Error {
    pub code: aclError,
    pub message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CANN error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for Error {}

impl From<aclError> for Error {
    fn from(code: aclError) -> Self {
        let message = match code {
            0 => "success".to_string(),
            100003 => "invalid file".to_string(),
            _ => format!("unknown error {}", code),
        };
        Self { code, message }
    }
}
```

### 3.3 禁止事项

- **禁止** 在库代码中使用 `unwrap()` / `expect()`（仅测试和 main 入口允许）
- **禁止** 在库代码中使用 `panic!()`（使用 `return Err(...)` 替代）
- **禁止** 使用 `assert!()` / `assert_eq!()` 在生产代码中（仅测试使用）
- **禁止** 使用 `todo!()` / `unimplemented!()`（需实现或返回错误）

### 3.4 错误传播

使用 `?` 运算符传播错误，禁止手动 match 后丢弃上下文：

```rust
// 正确
fn get_version() -> Result<String, Error> {
    let version = unsafe { aclsysGetVersionStr(...) }?;
    Ok(version)
}

// 错误
fn get_version() -> Result<String, Error> {
    match unsafe { aclsysGetVersionStr(...) } {
        0 => Ok(...),
        e => Err(e.into()),
    }
}
```

---

## 4. API 设计原则

### 4.1 分层职责

| 层 | 职责 | 示例 |
|----|------|------|
| cann-sys | 裸 FFI 绑定，不做抽象 | `unsafe fn aclsysGetVersionStr(...)` |
| cann | 安全封装，RAII，错误处理 | `Version::str() -> Result<String, Error>` |

### 4.2 安全 FFI 封装模式

```rust
// 正确：safe 函数封装 unsafe 调用
pub fn get_version() -> Result<String, Error> {
    let mut buf = [0u8; 128];
    let pkg_name = b"CANN\0".as_ptr() as *const c_char;
    let ret = unsafe {
        aclsysGetVersionStr(pkg_name, buf.as_mut_ptr() as *mut c_char)
    };
    if ret != 0 {
        return Err(Error::from(ret));
    }
    let cstr = CStr::from_bytes_with_nul(&buf)?;
    Ok(cstr.to_str()?.to_string())
}
```

### 4.3 Builder 模式

复杂结构使用 Builder 模式：

```rust
pub struct StreamBuilder {
    device_id: i32,
    priority: i32,
}

impl StreamBuilder {
    pub fn new(device_id: i32) -> Self {
        Self { device_id, priority: 0 }
    }

    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn build(self) -> Result<Stream, Error> {
        // ...
    }
}
```

### 4.4 Newtype 模式

使用 Newtype 保护类型安全：

```rust
pub struct DeviceId(i32);

impl DeviceId {
    pub fn new(id: i32) -> Result<Self, Error> {
        if id < 0 {
            Err(Error::new(ErrorKind::InvalidDevice))
        } else {
            Ok(Self(id))
        }
    }

    pub fn raw(&self) -> i32 {
        self.0
    }
}
```

### 4.5 RAII 资源管理

所有 CANN 资源必须实现 `Drop`：

```rust
pub struct Context {
    inner: aclrtContext,
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            aclrtDestroyContext(self.inner);
        }
    }
}
```

### 4.6 禁止事项

- **禁止** 导出 `pub unsafe fn`（内部可调用，但必须封装到 safe 接口）
- **禁止** 在公开 API 中暴露裸指针（使用 `*const T` / `*mut T` 只在内部 FFI 层）
- **禁止** 使用 `&'static str` 作为 C 字符串返回值（使用 `String` 或 `CStr<'static>`）

---

## 5. 测试策略

### 5.1 测试分类

| 类型 | 位置 | 命名 | 触发条件 |
|------|------|------|----------|
| 单元测试 | 同文件 `#[cfg(test)]` | `test_` 前缀 | `cargo test` |
| 集成测试 | `tests/` 目录 | `test_` 前缀 | `cargo test --tests` |
| 文档测试 | doc comment | ` ```rust ` 代码块 | `cargo test --doc` |
| 硬件测试 | `#[ignore]` 或 `#[cfg(feature = "hw_tests")]` | `test_` 前缀 | `cargo test -- --ignored` |

### 5.2 测试组织

```rust
// lib.rs 中
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_str_returns_ok() {
        let result = crate::Version::str();
        assert!(result.is_ok());
    }

    #[test]
    fn test_version_num_returns_positive() {
        let num = crate::Version::num().unwrap();
        assert!(num > 0);
    }
}
```

### 5.3 测试命名

```rust
// 格式：test_<功能>_<场景>_<期望结果>
#[test]
fn test_version_str_when_sdk_installed_returns_ok() { ... }

#[test]
fn test_version_num_when_no_sdk_returns_error() { ... }

#[test]
fn test_context_create_with_invalid_device_returns_error() { ... }
```

### 5.4 Mock 策略

对于需要硬件的功能，使用 mock 替代：

```rust
#[cfg(feature = "mock")]
mod mock {
    pub fn aclsysGetVersionStr(pkg: *const c_char, buf: *mut c_char) -> i32 {
        let version = b"9.0.0\0";
        unsafe {
            std::ptr::copy_nonoverlapping(version.as_ptr(), buf as *mut u8, version.len());
        }
        0
    }
}
```

### 5.5 测试数据管理

- 使用 `#[cfg(test)]` 块中的常量定义测试数据
- 测试文件存放在 `tests/` 目录，按功能命名（如 `tests/version.rs`）
- 测试数据文件存放在 `testdata/` 目录

---

## 6. 文档标准

### 6.1 文档注释

所有 `pub` 项必须有文档注释：

```rust
/// 获取 CANN 版本字符串
///
/// # Arguments
///
/// * `无` - 此函数不需要参数
///
/// # Returns
///
/// 返回 CANN 版本字符串，如 "9.0.0"
///
/// # Errors
///
/// 返回 `Error` 如果：
/// - CANN SDK 未安装
/// - 环境变量未设置
///
/// # Examples
///
/// ```
/// use cann::Version;
///
/// let version = Version::str().unwrap();
/// assert!(version.starts_with("9."));
/// ```
pub fn get_version() -> Result<String, Error> { ... }
```

### 6.2 FFI 函数文档

```rust
/// 获取 CANN 包版本字符串
///
/// 对应 C 函数：`aclsysGetVersionStr`
///
/// # Safety
///
/// - `pkg_name` 必须是有效的 NUL 结尾 C 字符串
/// - `version_str` 必须指向至少 128 字节的可写缓冲区
/// - 调用期间缓冲区内容会被修改
#[no_mangle]
pub unsafe extern "C" fn aclsysGetVersionStr(
    pkg_name: *const c_char,
    version_str: *mut c_char,
) -> aclError { ... }
```

### 6.3 模块文档

```rust
//! # CANN Runtime
//!
//! 提供 CANN 运行时 API 的安全封装。
//!
//! ## 功能模块
//!
//! - [`device`] - 设备管理
//! - [`context`] - 上下文管理
//! - [`stream`] - 流管理
//! - [`version`] - 版本查询
//!
//! ## 快速开始
//!
//! ```rust
//! use cann::Version;
//!
//! let version = Version::str().unwrap();
//! println!("CANN version: {}", version);
//! ```
```

### 6.4 文档测试

所有示例代码必须可编译：

```rust
/// ```
/// use cann::Version;
///
/// let version = Version::str().unwrap();
/// assert!(!version.is_empty());
/// ```
```

---

## 7. 版本管理

### 7.1 语义化版本

遵循 SemVer 2.0.0 规范：

| 版本号 | 含义 | 示例 |
|--------|------|------|
| Major | 不兼容的 API 变更 | 1.0.0 → 2.0.0 |
| Minor | 向后兼容的功能新增 | 1.0.0 → 1.1.0 |
| Patch | 向后兼容的问题修复 | 1.0.0 → 1.0.1 |

### 7.2 版本号格式

```
<major>.<minor>.<patch>[-<prerelease>][+<build>]
```

示例：
- `0.1.0` - 初始开发版本
- `0.1.1-alpha.1` - 预发布版本
- `1.0.0+build.123` - 构建元数据

### 7.3 变更日志

使用 [Keep a Changelog](https://keepachangelog.com/) 格式：

```markdown
# Changelog

## [0.2.0] - 2026-06-24

### Added
- `Version::str()` 获取 CANN 版本字符串
- `Version::num()` 获取 CANN 版本数值

### Changed
- Improved error messages for missing SDK

### Fixed
- Fixed buffer overflow in version string handling

## [0.1.0] - 2026-06-23

### Added
- Initial project structure
- FFI bindings for `aclsysGetVersionStr`
```

### 7.4 Git 提交规范

使用 [Conventional Commits](https://www.conventionalcommits.org/)：

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

类型（Type）：
- `feat`: 新功能
- `fix`: 问题修复
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构（不新增功能/修复问题）
- `perf`: 性能优化
- `test`: 测试相关
- `build`: 构建系统/依赖变更
- `ci`: CI 配置变更
- `chore`: 其他杂项

示例：
```
feat(version): add CANN version query API

- Implement `Version::str()` and `Version::num()`
- Add FFI bindings for `aclsysGetVersionStr`

Closes #12
```

---

## 8. CI/CD 流程

### 8.1 GitHub Actions 工作流

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, master]
  pull_request:

env:
  CARGO_TERM_COLOR: always

jobs:
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - run: cargo fmt --all -- --check
      - run: cargo clippy --workspace -- -D warnings

  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace

  doc:
    name: Documentation
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo doc --workspace --no-deps
        env:
          RUSTDOCFLAGS: -D warnings
```

### 8.2 发布流程

1. 更新版本号（`Cargo.toml`）
2. 更新 CHANGELOG.md
3. 创建 Git tag
4. 推送到 GitHub
5. 创建 GitHub Release
6. 发布到 crates.io（可选）

```bash
# 发布流程
cargo login <token>
cargo publish --allow-dirty
```

---

## 9. 性能指南

### 9.1 基准测试

使用 `criterion` 进行基准测试：

```rust
// benches/version_benchmark.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_version_str(c: &mut Criterion) {
    c.bench_function("version_str", |b| {
        b.iter(|| {
            cann::Version::str().unwrap();
        })
    });
}

criterion_group!(benches, benchmark_version_str);
criterion_main!(benches);
```

### 9.2 内存管理

- 使用 `Box::leak` 分配长期存在的内存
- 使用 `ManuallyDrop` 精确控制析构时机
- 避免不必要的内存拷贝（使用零拷贝技术）

### 9.3 并发安全

- 使用 `Arc<Mutex<T>>` 共享可变状态
- 使用 `Arc<T>` 共享不可变状态
- 使用 `crossbeam` 进行无锁编程

---

## 10. 构建配置

### 10.1 Cargo.toml 规范

```toml
[package]
name = "cann"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
description = "Rust bindings for CANN (Ascend Compute Neural Network)"
license = "Apache-2.0"
repository = "https://github.com/yourusername/cann-rs"
documentation = "https://docs.rs/cann"
readme = "README.md"
keywords = ["ascend", "npu", "cann", "ai", "bindings"]
categories = ["external-ffi-bindings", "os", "hardware-support"]

[dependencies]
# 依赖按字母顺序排列
thiserror = "1"
tracing = "0.1"

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
tempfile = "3"

[build-dependencies]
bindgen = "0.69"

[features]
default = []
hw_tests = []
mock = []

[[bench]]
name = "version_benchmark"
harness = false
```

### 10.2 Feature Flags

| Feature | 描述 | 默认 |
|---------|------|------|
| `mock` | 启用 mock 实现，不需要真实 SDK | 否 |
| `hw_tests` | 运行需要硬件的测试 | 否 |

### 10.3 Profile 配置

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true

[profile.dev]
opt-level = 0
debug = true
```

---

## 11. 安全规范

### 11.1 FFI 安全

- 所有 `extern "C"` 函数必须标记为 `unsafe`
- 所有 `unsafe` 块必须有 `// SAFETY:` 注释
- 禁止传递未初始化内存（除非 C 端明确要求）
- 使用 `NonNull<T>` 替代裸指针表示有效指针

### 11.2 输入验证

- 验证所有外部输入（指针、长度、索引）
- 使用 `checked_add` / `checked_mul` 防止整数溢出
- 使用 `slice::from_raw_parts` 时验证长度和对齐

### 11.3 内存安全

- 使用 `ManuallyDrop` 精确控制资源释放
- 使用 `Pin<T>` 防止指针失效
- 避免 `transmute`（除非绝对必要）

---

## 12. 跨平台支持

### 12.1 目标平台

当前支持：
- Linux aarch64（Ascend NPU 主要平台）
- Linux x86_64（开发/测试环境）

### 12.2 条件编译

```rust
#[cfg(target_arch = "aarch64")]
fn get_platform() -> &'static str {
    "aarch64-linux"
}

#[cfg(target_arch = "x86_64")]
fn get_platform() -> &'static str {
    "x86_64-linux"
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
compile_error!("unsupported platform");
```

---

## 13. 依赖管理

### 13.1 依赖审计

引入新依赖前必须：
1. 检查依赖的许可证兼容性（Apache-2.0 或 MIT）
2. 检查依赖的活跃度（最近更新、issue 响应）
3. 检查依赖的 `unsafe` 使用情况
4. 检查依赖的依赖树大小

### 13.2 依赖锁定

- 提交 `Cargo.lock` 文件
- 使用 `cargo update` 更新依赖
- 定期检查依赖安全性（`cargo audit`）

---

## 14. 代码审查清单

提交代码前检查：

- [ ] `cargo fmt --all -- --check` 通过
- [ ] `cargo clippy --workspace -- -D warnings` 通过
- [ ] `cargo test --workspace` 全部通过
- [ ] `cargo doc --workspace --no-deps` 生成成功
- [ ] 所有 `pub` 项有文档注释
- [ ] 所有 `unsafe` 块有 `// SAFETY:` 注释
- [ ] 无 `unwrap()` / `expect()` 在库代码中
- [ ] 无 `todo!()` / `unimplemented!()` 在生产代码中
- [ ] 提交信息符合 Conventional Commits 规范
- [ ] 变更日志已更新

---

## 15. 参考资源

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)
- [C Cookbook - FFI](https://rust-lang.github.io/rust-cookbook/ffi/)
- [Bindgen Documentation](https://rust-lang.github.io/rust-bindgen/)
- [SemVer 2.0.0](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Conventional Commits](https://www.conventionalcommits.org/)
