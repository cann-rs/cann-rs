# cann-rs

[![crates.io](https://img.shields.io/crates/v/cann.svg)](https://crates.io/crates/cann)
[![docs.rs](https://img.shields.io/docsrs/cann)](https://docs.rs/cann)
[![license](https://img.shields.io/badge/license-MIT--OR--Apache--2.0-blue)](LICENSE)

Huawei Ascend CANN NPU SDK 的 **Rust 绑定与安全封装**。
`cann-rs` 将昇腾 CANN C API（`libascendcl` / `libopapi` / `libnnopbase` 等）以 idiomatic Rust 提供：
裸 FFI 绑定（`cann-sys`）+ 类型安全、RAII、错误分类的安全层（`cann`），
供推理引擎等上层项目直接消费（本仓库与 [`reinfer`](https://github.com/cann-rs/reinfer) 引擎按契约协作）。

## 特性

- **双层架构**：`cann-sys`（零第三方依赖的裸 FFI 声明）→ `cann`（安全封装，RAII + `Result` 错误）
- **L0 运行时**：`Context` / 设备（数量、绑定、SOC 型号）/ `Stream` / `Event` /
  设备内存（`DeviceBuffer`、`HostBuffer`）/ 错误分类（OOM/可恢复白名单，fail-closed）
- **L1 算子树**：`Tensor` / `TensorList` / `Scalar` / `DataType` / `Format`；
  aclnn 两段式算子（`Matmul` / `Softmax` / `RmsNorm`）；GE 图引擎（`Graph` / `Session`，
  ONNX → .om 编译，C++ shim 桥接）
- **多 SDK 版本兼容**：CANN 7.x / 8.x 自动检测与降级
  （版本查询回退 `aclrtGetVersion`、GE 图引擎缺失降级、库族符号归属自动探测链接）
- **零依赖约束**：`cann-sys` 不引入任何第三方 crate；FFI 手写、逐条 `# SAFETY` 可审计
- **工程规范**：SDD 文档驱动（spec/plan/tasks）、Conventional Commits、双档 CI 设计

## 环境要求

| 项 | 要求 |
|---|---|
| 平台 | Linux（aarch64 / x86_64） |
| CANN SDK | 7.0+（8.x 为绑定锚点版本；7.x 自动降级） |
| NPU 驱动 | 运行 acl API 必需（版本查询/设备操作） |

## 安装

```toml
[dependencies]
cann = { version = "0.1", features = ["ffi"] }   # ffi 默认关闭
```

仅需裸绑定层：

```toml
cann-sys = "0.1"
```

## 快速开始

```rust
use cann::{device, Context, Version};

fn main() -> Result<(), cann::Error> {
    let _ctx = Context::new()?;              // aclInit（进程级单次，幂等）
    println!("CANN 版本: {:?}", Version::str()?);
    for dev in 0..device::device_count()? {
        device::set_device(dev)?;            // per-thread 设备绑定
        println!("设备 {dev}: SOC = {}", device::soc_name()?);
        device::reset_device(dev)?;          // 引用计数配对释放
    }
    Ok(())
}
```

完整可运行示例（含输出展示）：

```bash
cargo run -p cann --example device_info --features ffi
```

```
CANN 版本: 8.5.0
CANN 版本号: 85000000
设备数量: 1
设备 0: SOC = Ascend910B4
```

## API 概览

### L0（运行时基础）— `cann`

| 类型 | 说明 | 底层 |
|---|---|---|
| `Context`（RAII） | 运行环境生命周期 | `aclInit` / `aclFinalize` |
| `device_count()` / `set_device()` / `reset_device()` | 设备数量与 per-thread 绑定 | `aclrtGetDeviceCount` / `SetDevice` / `ResetDevice` |
| `soc_name()` | 芯片型号（如 `Ascend910B4`） | `aclrtGetSocName` |
| `Stream`（RAII） | 流创建/销毁/同步/查询 | `aclrtCreateStream` / `SynchronizeStream` / `StreamQuery` |
| `Event`（RAII） | 事件记录/同步/流间等待 | `aclrtCreateEvent` / `RecordEvent` / `SynchronizeEvent` / `StreamWaitEvent` |
| `DeviceBuffer` / `HostBuffer`（RAII） | 设备/锁页主机内存 | `aclrtMalloc` / `MallocHost` / `Free*` / `Memcpy` |
| `Error::is_oom()` / `is_recoverable()` | 错误分类（码段白名单，未知码 fail-closed） | `ACL_ERROR_RT_*` |

### L1（aclnn 算子树）— `cann`

| 类型 | 说明 |
|---|---|
| `Tensor` / `TensorList` / `Scalar` | aclTensor 基础类型（RAII + 元数据访问） |
| `DataType` / `Format` | aclDataType / aclFormat 安全枚举（双向映射） |
| `Matmul` / `Softmax` / `RmsNorm` | aclnn 两段式算子：`new()` 计算 workspace → `launch(&Stream)` |
| `Graph` / `Session` | GE 图引擎：`from_onnx` → `build_and_save(.om)` |

### `cann-sys`（裸 FFI）

模块按 C 头文件组织：`acl_base_rt` / `acl_rt` / `acl_device` / `acl_memory` /
`acl_error_code`（120 个 `ACL_ERROR_RT_*`，出处注释）/ `acl_meta` / `acl_datatype` /
`aclnn_ops` / `acl_grph`（GE C++ shim）。

## 特性（Feature Flags）

| Feature | 说明 | 默认 |
|---|---|---|
| `ffi` | 链接 `libascendcl.so` 等并编译 FFI 声明（依赖 `cann-sys/ffi`） | 关 |
| `default` | 无 ffi：类型/常量/错误分类可编译运行（CI 无 SDK 档） | 开 |

> 关闭 `ffi` 时安全层 API 保持签名一致，调用返回"未启用"错误——无 SDK 环境也能编译与单测。

## SDK 检测与构建

构建脚本自动发现 CANN SDK（`source set_env.sh` 后无需配置），优先级：

1. `ASCEND_TOOLKIT_HOME`（`set_env.sh` 主变量）
2. `ASCEND_HOME_PATH`
3. `ASCEND_AICPU_PATH`
4. `ASCEND_HOME`（旧版本兼容）
5. `ASCEND_OPP_PATH`（取父目录）
6. `$HOME/Ascend/cann`
7. `/usr/local/Ascend`

链接/运行时搜索路径：主 `lib64` + 插件库目录（`plugin/opskernel`、`plugin/nnengine`、
`opp/.../op_tiling`、`tools/aml/lib64`、driver 库）+ 用户 `LD_LIBRARY_PATH`。
跨 SDK 版本的**符号归属漂移**（如 `aclsys*` 在 8.x 位于 `libascendcl.so`/`libacl_rt.so` 视版本而定）
由 build.rs 的 ELF `.dynsym` 解析自动补链（零依赖，无需 `nm`）。

## 测试矩阵

| 档位 | 命令 | 环境 |
|---|---|---|
| 无 SDK（lint + 单测） | `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` | 任意 |
| 有 SDK（编译/链接） | `cargo build --workspace --features ffi` | CANN 7.x/8.x |
| 真机（smoke） | `cargo test --workspace --features ffi -- --ignored` | NPU 驱动 |

真机测试覆盖 Context/设备/流/事件/内存/张量/算子的完整往返
（开发板 CANN 7.0 已验 13/16；算子档受 7.x SDK aclnn 运行时环境所限——见
[`docs/specs/0002-l1-aclnn/spec.md`](docs/specs/0002-l1-aclnn/spec.md) 板子验证记录）。

## 版本兼容矩阵

| CANN SDK | 版本查询 | GE 图引擎 | aclnn 算子 |
|---|---|---|---|
| 8.x（绑定锚点） | `aclsys*`（候选包名遍历）→ 枚举 API → runtime 兜底 | ✅ `aclgrph*` | ✅ |
| 7.x | 回退 `aclrtGetVersion`（runtime 版本号） | 降级（无 `aclgrph*`，包络返回不支持） | 取决于算子运行环境 |

## 文档

- [功能规划 Roadmap](docs/roadmap.md) — L0/L1/L2 阶段与 API 总览
- [CANN 8.5 官方符号核定表](docs/cann-850-catalog.md) — 签名/错误码/库归属事实底稿
- [规格文档](docs/specs/) — SDD 三件套（0001-L0 / 0002-L1）
- [与 reinfer 的边界条约](docs/boundary-with-reinfer.md) — 能力归属（R1–R6）
- [项目宪法](constitution.md) — 全局技术约束与 Git/Rust 规则

## License

MIT OR Apache-2.0
