# Spec：ACL L0 运行时绑定（Phase 1）

> 归属：cann-rs · 状态：proposal · 创建：2026-08-25
> 契约锚点：`reinfer/specs/002-ascend-backend/plan.md` §Interface Contracts (L0) —— 签名以锚点为准
> 相关：`docs/boundary-with-reinfer.md`（归属条约）

## Problem Statement

cann/cann-sys 0.1.x 目前只提供版本探测（`aclsysGetVersionStr/Num`）与 `aclInit/aclFinalize`。reinfer 的昇腾后端 L0 需要**设备 / 流 / 事件 / 设备内存 / 主机内存** 原语与错误分类。本 Spec 定义 cann 必须提供的安全 API 集合与行为语义，使 `reinfer --features ascend` 能完成"编译 + 版本/设备诊断"最小闭环。

## Success Metrics

- **无 SDK 环境**：`cargo check --workspace`（默认 features）通过，类型/常量/Error 分类单测可运行（build.rs 降级后）
- **有 SDK 无 NPU 驱动**：`cargo check --workspace --features ffi` 通过
- **NPU 真机**：smoke 测试通过（Context 初始化、设备数量、DeviceBuffer 分配/释放、HostBuffer、Stream/Event 往返）
- **契约一致性**：本 spec 全部公开签名与 reinfer 002 plan.md 契约表逐项一致；不一致先行更新契约并记录 changelog
- 所有新 `extern` 带 `# SAFETY` 注释；`cargo clippy --all -- -D warnings` 绿

## User Stories

1. 作为 reinfer 后端作者，我只调用 cann 的安全 API（`Context`/`Stream`/`Event`/`DeviceBuffer`/`HostBuffer`/`Error`），从不接触头文件与指针。
2. 作为 cann-rs 维护者，我在无 NPU 的机器上也能跑类型与错误分类测试（no-fuzz CI 档）。

## Acceptance Criteria

- [ ] 决策已落实：`DeviceBuffer` 实现 `Send`（附 SAFETY 注释：仅限归属 device 使用）；`Error` 分类采用**码段白名单**（未知码 fail-closed → `is_oom()==false, is_recoverable()==false`）；`cann` 的 `[features] ffi` **默认关闭**（转发 `cann-sys/ffi`）
- [ ] `cann-sys`：`aclrtGetDeviceNum`(⚠️verify)、`aclrtSetDevice`、`aclrtCreateStream/DestroyStream`、`aclrtCreateEvent/RecordEvent/SynchronizeEvent/DestroyEvent`、`aclrtMalloc/Free/MallocHost/FreeHost/Memcpy`、`aclrtGetSocName`(⚠️verify) 绑定完成；`ACL_MEM_MALLOC_*`、`ACL_MEMCPY_*`、`ACL_ERROR_RT_*` 常量带出处注释
- [ ] `cann`：上述类型全部可安全构造、使用、析构；文档注明线程亲和性（ACL per-thread device 绑定）
- [ ] build.rs：非 `ffi` 构建时不再 `exit(1)`（改为探测告警），存在性探测生成 `cann_sys_has_*` cfg
- [ ] CI 三档：无 SDK（lint+单测）/ 有 SDK（编译）/ NPU（smoke）

## Non-Goals

- aclnn 算子绑定（L1）；Graph/HCCL（L2）；CustomOp / AscendC 编译链（后者属 reinfer `crates/jit`，见边界条约）
- bindgen 自动化生成；内存池/VMM 语义（那是 reinfer `crates/memory` 的策略层）
- 性能优化与 benchmark（benchmark 属于 reinfer）

## Constraints

- 手写 `extern "C"`，零新增构建依赖（不引 bindgen/libclang）
- 8.x/9.x 符号漂移：可疑符号经 build.rs 存在性探测 + `#[cfg(cann_sys_has_*)]` 门控；符号核实清单见 plan.md §verify-list
- 许可 MIT OR Apache-2.0；SDK 探测链沿用既有 build.rs（`ASCEND_TOOLKIT_HOME` → … → `/usr/local/Ascend`）
- 不修改 `cann-sys` 现有 API（保持向后兼容 0.1.x）
