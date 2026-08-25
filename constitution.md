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
| cann 安全封装 | Rust 命名惯例 (CamelCase 类型, snake_case 函数) | `Version::str()`, `Context::new()` |
| cann 模块组织 | 按 CANN 概念组织 | `device`, `context`, `stream`, `memory` |

## 4. 错误处理

- **cann-sys**: 直接返回 C 的 `aclError`（`int`），不做转换
- **cann**: 定义 `cann::error::Error` 结构体，带 `std::error::Error` 实现，包含原始 `aclError` 码

## 5. 依赖管理

- **cann-sys**: 零外部依赖（`[dependencies]` 为空，仅 `[build-dependencies]` 可用）
- **cann**: 可依赖 `cann-sys`、`thiserror`、`tracing` 等，审计后加入
- **cann-sys** 严格遵守零依赖原则，不引入任何外部 crate
- 其他 crate 引入外部依赖时需审计其依赖树的 `unsafe` 使用情况，未经审核不得引入

## 6. 测试要求

- 每个 FFI 函数至少有一个**链接/调用测试**（验证符号可链接）
- 需要 Ascend 硬件的测试标记为 `#[ignore]` 或 feature-gated（`#[cfg(feature = "hw_tests")]`）
- 所有 safe 封装层需要纯逻辑单元测试（不依赖硬件）
- **依赖硬件的功能需要有 mock 策略**：使用 `#[cfg(feature = "mock")]` 标记 mock 实现；纯 API 调用（不依赖 NPU 设备）的 feature 不做此要求

## 7. 文档要求

- 所有 `pub` 的 FFI 声明必须有 `///` doc，标注 C 函数原名
- 所有 `pub` 的 cann 封装必须有 `///` doc，包含 usage example

## 8. 预提交检查

提交前必须通过：
- [ ] 编译无 error/warning（Rust edition 2024）
- [ ] 非硬件依赖的测试全部通过
- [ ] 文档生成完整，无 broken link
- [ ] 无未标注 `// SAFETY:` 的 `unsafe` 代码

## 9. Git 版本控制规则

### 9.1 提交信息

- 使用 Conventional Commits（`<type>(<scope>): <description>`；type 见 AGENT.md §7.4）
- **禁止任何 AI 签名 trailer**：提交信息（含 body/footer）不得包含 `Co-Authored-By: Claude`、`Generated with Claude Code` 等自动化署名，也不得虚构人工署名
- 提交粒度：一次提交只做一件事（一个功能/一个修复/一份文档），禁止多主题混提

### 9.2 提交范围

- 仅提交与本次改动直接相关的文件；禁止 `git add -A` 混入无关文件
- 提交前检查 `git status`/`git diff --stat` 确认范围
- `Cargo.lock`、`target/`、临时抓取文件（`/tmp/*` 等）不入库
- 文档类提交类型用 `docs`；SDD 文档与代码分开提交

### 9.3 分支与历史

- 主力开发分支 `main`；功能/修复先建分支再合入
- 禁止改写已推送的历史（rebase 仅限未推送的本地提交）
- 版本 tag 语义化（SemVer），与 `Cargo.toml` 版本号一致后再打 tag

## 10. Rust 工程门槛（强制）

以下为所有成员 crate 的硬性门槛，与 AGENT.md 保持一致，但以本节为强制底线：

- `cargo fmt --all -- --check` 必须通过；`cargo clippy --workspace -- -D warnings` 必须通过
- 库代码禁止 `unwrap()`/`expect()`/`panic!()`/`todo!()`/`unimplemented!()`（仅测试与二进制入口允许）
- 所有 `pub` 项必须有文档注释；所有 `unsafe` 调用必须伴随 `// SAFETY:` 注释
- 不导出 `pub unsafe fn`；公开 API 不暴露裸指针与 `&'static str` 作 C 字符串返回
- CANN 资源（Context/Stream/Event/设备内存）必须 RAII（`Drop` 释放），禁止手动泄漏
- `cann-sys` 保持零第三方依赖（`[dependencies]` 为空）；新增依赖须审计许可证与 `unsafe` 使用
- **契约先行**：公开 API 签名变更先更新 `reinfer/specs/002` 锚点再实现本地代码（R3）

## 11. 参考资源与事实来源

- `docs/cann-850-catalog.md` —— CANN 8.5.0 官方文档目录树与 API 签名核实表（2026-08-25 抓取），符号/签名裁定以此与官方文档为准；与计划中的 verify-list 对照使用
- 昇腾文档站：`https://www.hiascend.com/document/detail/zh/CANNCommunityEdition/850/`，正文为 SSR，目录由 `/ascendgateway/ascendservice/doc/node/tree/...` API 供数据（需 Referer）
- 安装/环境变量说明：`docs/version-detection.md`；版本号编码 `MAJOR×10^7 + MINOR×10^5 + PATCH×10^3`
- 官方接口签名以文档站 `API/appdevgapi/aclcppdevg_03_*.html` 页为准；核实时优先引用本文档 §2 的核定结果
