# cann-rs Project Constitution

本文件定义 cann-rs 项目所有成员 crate 必须遵守的全局技术约束。任何 spec.md / plan.md / 代码 均不得违反。是 AI 开发时的"潜意识"边界。

## 1. 目录结构与模块边界

- `cann-sys/` — 提供**裸 FFI 绑定**，零抽象，只做 `extern "C"` 声明 + 类型/常量/结构体翻译
- `cann/` — 提供**安全 Rust 封装**，依赖 `cann-sys`，可添加第三方依赖
- 新增子 crate (如 `cann-ops`, `cann-nn`) 直接放置于项目根目录
- `docs/specs/<feature-id>/` — 每个功能的 SDD 文档 (spec / plan / tasks)

**依赖方向**：`cann → cann-sys → (C 链接库)`，不允许反向或跨层依赖。

## 2. 安全规则

- **所有 FFI 函数声明**必须标记 `unsafe`，必须附带 `// SAFETY:` 注释解释调用方前提条件
- **cann-sys** 中禁止任何 safe 封装逻辑（不自动 Drop、不转换错误、不包装 struct）
- **cann** 中禁止导出 `unsafe` 函数（内部可调用 unsafe，但必须封装到 safe 接口中）
- **禁止在库代码中 panic**（应返回 `Result`；仅测试或二进制入口可用 `unwrap`）
- **禁止在 FFI 边界传递未初始化内存**（除非 C 端明确要求）

## 3. 命名规则

| 层 | 规则 | 示例 |
|----|------|------|
| cann-sys 类型/常量 | 与 C 头文件完全一致 | `aclsysGetVersionStr`, `aclError`, `ACL_SUCCESS` |
| cann-sys 模块组织 | 按 C 头文件分组 | `acl_rt`, `acl_base_rt` |
| cann 安全封装 | Rust 命名惯例 (CamelCase 类型, snake_case 函数) | `Version::get_str()`, `Context::new()` |
| cann 模块组织 | 按 CANN 概念组织 | `device`, `context`, `stream`, `memory` |

## 4. 错误处理

- **cann-sys**: 直接返回 C 的 `aclError`（`int`），不做转换
- **cann**: 定义 `cann::error::Error` 枚举，带 `std::error::Error` 实现，包含原始 `aclError` 码

## 5. 依赖管理

- **cann-sys**: 零外部依赖（`[dependencies]` 为空，仅 `[build-dependencies]` 可用）
- **cann**: 可依赖 `cann-sys`、`thiserror`、`tracing` 等，审计后加入
- 所有 crate 不得引入 `unsafe` 操作的外部 crate（如 `libc` 除外）

## 6. 测试要求

- 每个 FFI 函数至少有一个**链接/调用测试**（验证符号可链接）
- 需要 Ascend 硬件的测试标记为 `#[ignore]` 或 feature-gated（`#[cfg(feature = "hw_tests")]`）
- 所有 safe 封装层需要纯逻辑单元测试（不依赖硬件）
- **不可用硬件时应有 mock 策略**：使用 `cfg!(feature = "mock")` 替代真实调用

## 7. 文档要求

- 所有 `pub` 的 FFI 声明必须有 `///` doc，标注 C 函数原名
- 所有 `pub` 的 cann 封装必须有 `///` doc，包含 usage example
- 文档中须注明对应 CANN 版本（当前目标：≥ 9.0.0）

## 8. 预提交检查

提交前必须通过：
- [ ] `cargo check` 无 error/warning
- [ ] `cargo test` 所有非硬件测试通过
- [ ] `cargo doc` 无 broken link
- [ ] 无未标注 `// SAFETY:` 的 `unsafe` 代码
