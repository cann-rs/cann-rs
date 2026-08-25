# Spec：L1 算子树（aclTensor + 首批 aclnn 算子 + GE 图引擎）

> 归属：cann-rs · 状态：proposal · 创建：2026-08-25
> 契约锚点：`reinfer/specs/002-ascend-backend/plan.md` §L1（同步后生效）——签名以锚点为准
> 前置：0001-l0-runtime（L0 绑定 ✅ 已提交：Context/Stream/Event/DeviceBuffer/HostBuffer/Error 分类）

## Problem Statement

reinfer 昇腾后端完成"最小闭环"（L0 版本/设备诊断）后，需要真正执行推理：构造张量（`aclTensor`）、
以 aclnn 两段式 API（GetWorkspaceSize + Launch）执行算子（Matmul/Softmax/RMSNorm）并用 GE 图引擎
（`aclgrph*`）构建/编译/保存模型图。本 Spec 定义 cann-rs L1 必须提供的基础数据类型封装、
首批算子封装与图引擎绑定，契约表同步到 reinfer 002（R3）。

## Success Metrics

- **无 SDK 环境**：默认特性 `cargo check/test/clippy` 全绿（新封装带非 ffi 降级）
- **有 SDK**：`--features ffi` 编译链接 libascendcl + libge_common(-compiler) 成功；aclnn 两段式符号可链接
- **真机**：smoke 通过 —— Tensor 创建/销毁；Matmul/Softmax/RMSNorm GetWorkspaceSize+Launch 往返；
  ONNX → aclgrphParseONNX → aclgrphBuildModel → SaveModel 链路
- **契约一致性**：L1 契约表与 reinfer 002 逐项一致（先同步契约再实现）
- 所有新 `extern` 带 `# SAFETY`；`-D warnings` 全绿

## User Stories

1. 作为 reinfer 后端作者，我用 `cann::tensor::Tensor` + `cann::matmul::Matmul` 的 Rust API 完成算子执行，
   不接触 `aclTensor` 裸指针与两段式 executor。
2. 作为 reinfer 端 loader，我用 `cann::graph::Session`（包装 aclgrph*）编译 ONNX/元数据模型并保存 .om。
3. 作为 cann-rs 维护者，我在无 SDL（无驱动）环境也能编译与跑类型测试（L0 之后的 CI 档位复用）。

## Acceptance Criteria

- [ ] **基础数据类型（cann-sys）**：`aclTensor`/`aclTensorList`/`aclScalar` 不透明类型 + 生命周期函数
      （`aclCreateTensor`/`aclDestroyTensor`/`aclDestroyTensorList`/`aclCreateTensorList`）与访问器
      （形状/步长/offset/format/datatype）绑定；`aclDataType`/`aclFormat`/`aclTranspose` 枚举按
      `acl_meta.h`/`acl_base.h` 抄录（数值出处注释 + 单测）。⚠️ verify：`aclDataType` 枚举确切头文件位置。
- [ ] **首批 aclnn 算子（cann-sys）**：Matmul/Softmax/RMSNorm 两段式函数
      （`aclnn<Op>GetWorkspaceSize` + `aclnn<Op>`）绑定，`aclnnStatus` 类型 + 常量（出处注释）。
      ⚠️ verify：`aclnn_softmax.h`/`aclnn_rms_norm.h` 的算子签名参数（softmax 的 dtype/dim 参数形态）。
- [ ] **GE 图引擎（cann-sys）**：`aclgrphParseONNX`/`aclgrphParseONNXFromMem`/`aclgrphBuildModel`/
      `aclgrphSaveModel` 绑定（头文件 `include/parser/onnx_parser.h`；返回类型 `graphStatus`）；
      ⚠️ verify：aclgrphBuildModel 完整签名、graphStatus 错误码族、GE 头文件依赖（parser 目录）与链接库集合
      （libge_common/libge_compiler/libexe_graph）。
- [ ] **cann 层**：`Tensor`（RAII + 构造器 + 元数据访问）、`TensorList`、`Scalar`；`Operator` 封装模式
      （首例 Matmul/Softmax/RMSNorm 实现 `GetWorkspace → Executor::launch(&Stream)`）；
      `Session`/`Graph`（ONNX → 模型编译/保存）。全部非 ffi 降级 + `-D warnings` 绿。
- [ ] **错误族对接**：`aclnnStatus`/`graphStatus` 到 `cann::Error` 的映射（fail-closed：未知码 Fatal），
      并入 L0 白名单机制文档附录。
- [ ] **契约同步**：`reinfer/specs/002` 增加 L1 契约表（与本文一致）；两仓库 CI 引用一致。

## Non-Goals

- 全量 803 个 aclnn 算子（逐批；本 SDD 只绑 3 个首批算子，封装模式可复用）
- 算子自动调优/benchmark/混合精度策略（reinfer KernelProvider + TuneDb）
- GE 图引擎的图构造 C++ API（`include/graph/` 的 tensor.h/operator.h/graph.h）——reinfer 若直接构图属
  引擎侧；cann-rs 只绑 C API 解析/编译/保存链路
- AscendC 内核编译（reinfer `crates/jit`）；内存策略（reinfer memory）
- aclnn 算子执行期间的 buffer pool/stream 池管理（reinfer）

## Constraints

- 手写 `extern "C"`，零新增构建依赖；cann-sys 零第三依赖不变
- 符号漂移：沿用 build.rs `cann_sys_has_*` 探测（新增 GE/aclnn 符号到 SYMBOLS）
- GE 链接：新增 `libge_common`/`libge_compiler` 等链接库时在 build.rs 中 gated（仅 ffi）
- 许可不加；与 L0 同守 constitution（RAII、SAFETY、禁止 panic…）

## Changelog

- 2026-08-25：发布。verify-list 全清（本地 SDK 8.5 头文件；真机项移交开发板）；
  采用 C++ shim 桥接 `aclgrph*`（C++ API）；`aclError`/`aclnnStatus` 同为 `i32` 别名
  （共用 `From<aclError>`，E0119 限制文档化）；workspace 使用 host 对齐缓冲区
  （官方建议 device 侧，真机验证失败则迁移 `DeviceBuffer`——待开发板）。
