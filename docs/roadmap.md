# cann-rs 功能实现规划（Roadmap）

> 创建：2026-08-25 · 依据：`docs/boundary-with-reinfer.md` R5 版本节奏 + `docs/specs/0001-l0-runtime/` SDD + `docs/cann-850-catalog.md`（CANN 8.5.0 官方符号核定表）
> 状态图例：✅ 已实现 · 📋 SDD 已就绪待实现 · 🗓 未 SDD · 🔒 按需（未被消费方引用）

## 总体时间线

```
0.1.x = L0  绑定层地基（进行中）
  ├─ ✅ 0.1.1 版本探测（已发布 crates.io）
  └─ 📋 0.1.2 L0 运行时绑定（0001 SDD 已定稿，未实现）
0.2.x = L1  算子树（reinfer 真正消费层）
0.3.x = L2  分布式与自定义算子
0.4.x = L3  [建议] 模型管理与媒体处理（当前条约未规划）
```

阶段对应边界条约 R5：0.1.x=L0、0.2.x=L1、0.3.x=L2；reinfer 发布期锁 crates.io 版本，开发期 `[patch]` 本地路径。**优先级以 reinfer 消费顺序为准：先能跑 → 再高效 → 再分布式。**

## 功能规划主表

| 阶段 | 功能组 | 官方锚点（cann-850-catalog.md 章节） | cann-rs 交付形态 | 状态 |
|---|---|---|---|---|
| **L0** 0.1.x | 版本/环境探测 | `aclsysGetVersionStr/Num`、`aclrtGetVersion` | `Version::str/num` | ✅ 0.1.1 已发布 |
| **L0** | 初始化生命周期 | `aclInit`/`aclFinalize` | `Context` RAII（0001 Task 6） | 📋 0001 待实现 |
| **L0** | 设备管理 | `aclrtGetDeviceCount`/`aclrtSetDevice`/`aclrtResetDevice`/`aclrtGetSocName` | `device_count()`/`set_device()` | 📋 0001 Task 3 |
| **L0** | 流/事件/同步 | `aclrtCreateStream`/`aclrtSynchronizeStream`/`aclrtCreateEvent`/`aclrtRecordEvent`/`aclrtSynchronizeEvent` | `Stream`/`Event` RAII | 📋 0001 Task 5-6 |
| **L0** | 设备/主机内存 | `aclrtMalloc/Free/MallocHost/FreeHost/Memcpy` | `DeviceBuffer`/`HostBuffer`（impl Send） | 📋 0001 Task 4-6 |
| **L0** | 错误分类 | `ACL_ERROR_RT_*` 码段 | `Error::is_oom/is_recoverable`（码段白名单，fail-closed） | 📋 0001 Task 2/7 |
| **L0** | 工程基线 | build.rs 降级、存在性探测 cfg、CI 三档 | — | 📋 0001 Task 1/9（P0 先行） |
| **L1** 0.2.x | 基础数据类型 | aclTensor / aclDataType / aclFormat / aclDataBuffer（API/basicdataapi） | 类型翻译（`aclTensor` 等） | 🗓 未 SDD |
| **L1** | 图引擎 GE | `aclgrphParseONNX/BuildModel/SaveModel` + Session（API/ascendgraphapi + graph/graphdevg） | `Graph`/`Session` 绑定（哪些算子进图、桶化、内存复用由 reinfer 决策） | 🗓 未 SDD |
| **L1** | aclnn 算子库 | Matmul / Softmax / RMSNorm / TopK…（API/aolapi 算子表，量级大） | `cann-ops` 风格 crate，按算子清单逐批封装 | 🗓 未 SDD |
| **L1** | 设备属性 | `aclrtGetDeviceInfo`/`aclrtGetDeviceCapability`/`aclrtGetDevicesTopo`（8.5 符号） | `DeviceProps`（SoC/显存/L2/拓扑，供能力分级与 TuneDb 键） | 🗓 未 SDD |
| **L2** 0.3.x | HCCL 集合通信 | `hcclInit`/`HcclCommCreate`/`AllReduce`/`AllGather`/`ReduceScatter` + 点对点（API/hcclapiref + commlib/hcclug） | `comm` 原语绑定（算法/拓扑/回退属 reinfer，与 CUDA `crates/comm` 共享设计） | 🗓 未 SDD |
| **L2** | 自定义算子装载执行 | `aclrtBinaryLoadFromFile` / Kernel 加载执行 / aclnnCustomOp 族 | 装载/执行 API 绑定（编译流水线属 reinfer `crates/jit`） | 🗓 未 SDD |
| **L2** | 高级同步/多设备 | PeerAccess、Notify/CntNotify/Label（8.5 新增族） | 按需补绑 | 🔒 按需 |
| **L3** 0.4.x | 模型管理（om 推理） | `aclmdlLoad/Execute/Unload` + 动态 Shape/AIPP 全套（appdevg/acldevg 模型管理节） | `Model` RAII + 推理接口 | 🔒 建议项，reinfer 直接路线可能跳过 |
| **L3** | 媒体处理 DVPP | VPC/VDEC/VENC/JPGD/JPEGD/PNGD（V1/V2，appdevg/acldevg 媒体数据处理节） | `acldvpp` 独立大块 | 🔒 与推理引擎基本正交 |
| **L3** | 运行时 Profiling / Tensor 传输 | `aclrtProf*` / `acltdt*` | 按需绑 | 🔒 按需 |
| — | 不做（reinfer 拥有） | AscendC 编译链 / autotune / TuneDb / benchmark / 内存策略 | ← reinfer（边界条约 R1/R2：同符号只绑一次、单向依赖） | ✅ 已约定 |

## 优先级排布逻辑（reinfer 消费视角）

1. **L0 是硬前置**：没有 `Context`/`Stream`/`DeviceBuffer`，reinfer 后端一行代码都跑不了 → 0001 的 9 个 Task 是唯一当前工作包。
2. **L1 决定"跑得快不快"**：aclTensor 等基础数据类型是 aclnn 算子的地基；aclnn 算子库按 reinfer 算子清单**首批只封装子集**（Matmul/Softmax/RMSNorm 等），不必全量。
3. **L2 是"多卡才能跑大模型"**：HCCL 与 CUDA 侧 `crates/comm` 共享算法设计，何时启用由 reinfer 决定。
4. **L3 建议项**：om 模型推理属于"模型已离线编译好"的旧式路径；reinfer 走图引擎 + 算子双体系，大概率不做——**等 reinfer 明确引用再开 SDD**，避免白做。

## 风险与决策点

| 点 | 说明 |
|---|---|
| aclnn 算子库规模 | 上百算子，需要"算子清单 → 分批封装"机制（每批一个 SDD；优先清单由 reinfer 提供） |
| DVPP/模型管理归属 | 边界条约职责矩阵未覆盖这两块；按"换 CUDA 不成立 → cann-rs"规则归 cann-rs，建议等 reinfer 提出需求再纳入 |
| `ACL_ERROR_RT_*` 数值 | 唯一未核实的 verify 项（0001 verify-list 末两项），卡在 `acl_error_code.h` 真机/头文件核对 |
| 发版节奏 | 0001 全部落地后，L1 首版为 0.2.0；reinfer 开发期用 `[patch]` 不受发版影响 |

## 里程碑入口

- 当前里程碑：`docs/specs/0001-l0-runtime/tasks.md`（Task 1 build.rs 降级 → … → Task 9 CI 收尾）
- 下一个 SDD 候选：`docs/specs/0002-l1-*`（基础数据类型 + 首批 aclnn 算子 + GE 图引擎）
