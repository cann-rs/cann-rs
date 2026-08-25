# Tasks：L1 算子树（aclTensor + 首批 aclnn 算子 + GE 图引擎）

> Derived from plan.md · 每条可独立验证 = 对应验收项

## Task L1-0: Verify-list 实机核对（P0，先行）

- 在本地 SDK 头文件逐项核对 plan.md Verify-list 6 项（aclCreateTensor 签名、aclDataType/aclFormat 定义位置与值、
  softmax/rms 算子签名、aclgrph* 全签名、GE 链接库归属（nm -D）、executor/workspace 语义）
- 验证：verify-list 全划线 + 记录到 `docs/cann-850-catalog.md` §2 追加 L1 表（如有新钉）

## Task L1-1: 基础数据类型绑定（acl_meta.rs + acl_datatype.rs）

- `aclTensor`/`aclScalar`/`aclTensorList` 不透明类型 + 生命周期/访问器函数（含 `# SAFETY` + 官方锚点）
- `aclDataType`/`aclFormat`/`aclTranspose` 枚举抄录（数值出处注释 + 单测断言）
- 验证：ffi 编译通过；非 ffi clippy/test 绿

## Task L1-2: 首批 aclnn 算子（aclnn_ops.rs）

- `aclnnStatus` 类型/常量；Matmul/Softmax/RMSNorm 两段式声明（签名按 L1-0 核实）
- 验证：ffi 编译 + 链接（libascendcl 已含 aclnn？若 aclnn 在独立 .so 则 build.rs 补链）

## Task L1-3: GE 图引擎绑定（acl_grph.rs）

- `aclgrphParseONNX`/`ParseONNXFromMem`/`BuildModel`/`SaveModel` + `graphStatus` + GE 错误码常量
- build.rs：GE 链接库按 L1-0 归属结果补（ffi 时）
- 验证：ffi 编译链接；parser 头文件依赖；`graphStatus` 失败路径返回码可译

## Task L1-4: cann 安全封装（tensor.rs / op/* / graph.rs）

- `Tensor`/`TensorList`/`Scalar`（RAII + 元数据访问 + 非 ffi 降级）
- `Operator` trait + `OpExecutor`（workspace 语义文档化）+ Matmul/Softmax/RmsNorm 实现（Builder 风格）
- `Graph`（from_onnx/from_mem）+ `Session`（build/save .om）
- 验证：无 ffi 单测（构造/类型）；真机 smoke（GetWorkspaceSize+launch 往返、ONNX→build→save）

## Task L1-5: 错误族扩展（error.rs）

- `Error` 增加 From<aclnnStatus>/From<graphStatus>（fail-closed：非 0 Fatal；graphStatus 白名单按 L1-0 verify 结果）
- 验证：测试表（成功/未知码 fail-closed）+ 文档更新

## Task L1-6: 契约同步与收尾

- `reinfer/specs/002-ascend-backend/plan.md` 增加 L1 契约表（与本文 plan.md §Interface Contracts 逐项一致）；
  reinfer `boundary.md` 若需增补（图引擎边界已明）
- CI：`--features ffi` job 增加 aclnn/GE 链接构建；README 追加 L1 API 行；0002 changelog
- 验证：双仓库 CI 逻辑一致；两仓库文档交叉引用正确

---

Completion gate：L1-0..L1-6 接受；verify-list 全清；与 reinfer 002 L1 契约表逐项核对一致；
真机 smoke（三算子 + GE 链路）通过（延续 L0 待驱动）。
