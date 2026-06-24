# cann-sys-core: CANN FFI 绑定层（版本查询）

## Problem Statement

CANN SDK 提供 C API 用于 Ascend NPU 管理。Rust 生态缺乏对这些 API 的绑定。
`cann-sys` 作为 `-sys` 约定 crate，职责是将 CANN C API 以 unsafe Rust 的形式暴露出来，不添加任何安全抽象。
首个里程碑实现**查询 CANN 版本号**这一最小可行功能，验证从 Rust 经 FFI 调用 CANN 的能力。

## Success Metrics

- 在安装有 CANN 9.0.0 的系统上，`cargo run` 可输出当前 CANN 版本号（字符串和数值）
- `cargo test` 包含一个版本号正确性的验证测试
- 无 CANN SDK 时编译失败，错误信息包含 `"CANN SDK not found"` 和 `"ASCEND_TOOLKIT_HOME"` 字样的显式指引
- `cargo doc` 正常生成

## User Stories

| 角色 | 场景 | 期望结果 |
|------|------|----------|
| Rust 开发者 | 获取 CANN 版本字符串 | 调用 `Version::str()` 返回 `"9.0.0"` |
| Rust 开发者 | 获取 CANN 版本数值 | 调用 `Version::num()` 返回 `90000000` |
| 集成工程师 | 在没有 CANN SDK 的环境构建 | 编译时得到清晰错误和修复指引 |
| 安装维护者 | 自定义 CANN 安装路径 | 通过环境变量指定 SDK 路径 |

## Acceptance Criteria

- [ ] `cargo build` 在有 CANN SDK 的系统上成功
- [ ] `aclsysGetVersionStr` 可从 Rust 调用并返回正确版本字符串
- [ ] `aclsysGetVersionNum` 可从 Rust 调用并返回正确版本数字
- [ ] 所有 `extern "C"` 函数签名与 C 头文件一致（含 `unsafe` 标注）
- [ ] 无 CANN SDK 时编译失败，报错信息包含 `ASCEND_TOOLKIT_HOME` 环境变量的提示
- [ ] `cann-sys` 零外部依赖（`[dependencies]` 为空）

## Non-Goals

- **不做安全封装**：不提供 RAII、错误类型转换或自动资源管理（这是 `cann` crate 的职责）
- **不做 `aclInit` / `aclFinalize`**：版本查询不需要初始化运行时（后续 feature 再引入）
- **不做其他 API**：不包含设备管理、内存、流等 API 声明（后续 feature 逐步添加）
- **不做 mock 层**：版本查询是纯字符串/数值操作，不需要通过硬件测试
- **不跨平台**：仅 Linux (aarch64/x86_64)

## Constraints

- **CANN 兼容版本**：≥ 8.0（9.0.0 已安装验证）
- **Rust edition**：2024
- **目标架构**：aarch64-linux（当前环境）
- **cann-sys 零外部依赖原则**：`[dependencies]` 为空，仅 `[build-dependencies]` 可在审计后引入
