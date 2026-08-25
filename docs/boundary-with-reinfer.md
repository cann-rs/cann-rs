# 与 reinfer 的边界条约（Ascend 能力归属）

> 状态：2026-08-25 双方确认 · 契约锚点：`reinfer/specs/002-ascend-backend/plan.md`（L0 签名表）
> 镜像：`reinfer/specs/002-ascend-backend/boundary.md`（英文版）——出现分歧时以锚点为准。

## 1. 目的

reinfer（引擎）与 cann-rs（SDK 绑定层）并行成长，**互不重叠、互不缺口**。本文件是 Ascend 能力归属的唯一来源——下面每一项恰好属于一边。

## 2. 归属判定规则（唯一规则）

把 SDD 的粒度检验用到硬件栈上：

> **若该工作"换成 CUDA 实现仍成立" → reinfer（引擎）负责。**
> **若该工作只属于 CANN SDK 表面 → cann-rs（绑定层）负责。**

例：KV 页淘汰策略换 CUDA 也要做 → reinfer；`aclrtMalloc` 签名换 CUDA 就不成立 → cann-rs。

## 3. 分层

```
CANN SDK (8.x/9.x) → cann-sys (裸 FFI) → cann (安全 API)   ← cann-rs 拥有
        → reinfer-ascend（后端消费）→ reinfer 引擎            ← reinfer 拥有
```

与本仓库 constitution.md §1 的 `cann → cann-sys`（禁止反向/跨层）一致。

## 4. 职责矩阵

| 能力 | 归属 | 边界说明 |
|---|---|---|
| aclInit/aclFinalize、Context RAII | cann-rs（cann） | reinfer 只管消费 `cann::Context` |
| 设备数量 / set / reset | cann-rs（cann-sys + cann） | reinfer 映射到 `core::DeviceId` |
| Stream / Event（创建、记录、同步） | cann-rs（绑定 + 安全 API） | reinfer 在 `ExecCtx` 中编排使用 |
| 设备/主机内存 分配释放原语 | cann-rs（`DeviceBuffer`、`HostBuffer`） | **策略**（页池、引用计数、offload、VMM 语义）= reinfer `crates/memory` |
| ACL 错误码与 `is_oom/is_recoverable` | cann-rs（SDK 语义） | reinfer 映射到引擎 `LaunchError`（映射表见 002/plan.md） |
| 设备属性（SoC、显存、L2…） | cann-rs（`DeviceProps`） | reinfer 用于能力分级与 TuneDb 键 |
| aclnn 算子封装（Matmul/Softmax/RMSNorm/TopK…） | cann-rs（cann-ops 风格 crate） | reinfer 只做 KernelProvider Vendor 档选择 + 自动调优 |
| 图捕获（GE 图引擎 `aclgrph*`：aclgrphParseONNX/BuildModel/SaveModel + Session） | cann-rs（绑定） | reinfer 决定哪些算子进图、桶化、内存复用 |

> 注（2026-08-25）：CANN 8.5.0 已无 `aclrtGraph*` 符号，图体系为 GE 图引擎（`api/ascendgraphapi`：aclgrph* + Session）。旧表述的 `aclrtGraph*` 仅为 8.x 早期动态图 API；按 8.5 锚点统一为 aclgrph*。
| HCCL 原语（初始化、集合通信、收发） | cann-rs（cann-sys + cann） | reinfer：算法、拓扑、回退（与 CUDA 侧 `crates/comm` 共享设计） |
| AscendC 内核**编译流水线**（bisheng/AOC、缓存、锁） | **reinfer `crates/jit`** | 对齐 FlashInfer JitSpec；内核源码资产属于引擎 | 
| AscendC 自定义算子**装载/执行 API**（aclnnCustomOp…） | cann-rs（绑定） | reinfer 驱动 编译 → 装载 → 执行 |
| 版本/环境探测 | cann-rs（0.1.x 已有） | reinfer `diag` 负责输出格式 |
| autotune、TuneDb、基准、差分测试 | reinfer | 引擎级；cann-rs 不做 benchmark |
| 绑定层 smoke 测试 | cann-rs 自身测试 | reinfer 在 `ascend-gpu` runner 上做集成测试 |
| 契约文档与变更记录 | reinfer `specs/002`（锚点） | cann-rs 镜像并互相引用；改动走 spec changelog |

## 5. 硬规则

- **R1 单向依赖**：cann-rs 绝不引入 reinfer 类型；reinfer 绝不直接消费 cann-sys 类型。
- **R2 不重复绑定**：同一个 SDK 符号只绑定一次（全部在 cann-sys）。
- **R3 契约先行**：签名变更先更新 `reinfer/specs/002`，两仓库再实现（SDD 防腐烂条款，类比本仓库 constitution）。
- **R4 SDK 8.x/9.x 门禁**：cann-rs 只负责探测与上报；reinfer 决定行为（回退或拒绝）。
- **R5 节奏**：cann-rs 版本语义化 —— 0.1.x=L0，0.2.x=L1，0.3.x=L2；reinfer 发布期锁 crates.io 版本，开发期用 `[patch]` 本地路径。
- **R6 治理同源**：两仓库同样 SDD（spec/plan/tasks）+ Conventional Commits + 无 AI 署名 trailer；出现分歧先开 issue 对齐，禁止静默分叉。

## 6. 归属清单（速查）

**cann-rs 实现**（SDK 表面）：Context + 设备/流/事件生命周期、内存原语、aclnn 算子封装、图绑定、HCCL 绑定、CustomOp 执行 API、DeviceProps、错误码与分类、版本探测。

**reinfer 实现**（引擎表面）：`crates/ascend` 后端（消费、能力分级、`diag`）、KernelProvider 选择 + TuneDb/自动调优、内存策略、通信算法、AscendC 流水线、基准与差分测试、契约治理。
