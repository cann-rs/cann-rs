# Plan：L1 算子树（aclTensor + 首批 aclnn 算子 + GE 图引擎）

> Derived from docs/specs/0002-l1-aclnn/spec.md

## Architecture Decision

1. **延续手写外 extern + 零依赖**（L0 已建立模式）。
2. **aclnn 两段式封装统一模式**：`<Op>::new(args...)`（调 GetWorkspaceSize 拿 workspace 与 executor）→
   `launch(&Stream)`（调 `<Op>(workspace, wsSize, executor, stream)`）。executor 单独 RAII 不持有（acl 语义：
   workspace 指针由调用方管理；executor 生命周期 = workspace 生命周期）。cann 层 `Operator` 抽象：
   `trait Op { fn executor(&self) -> &OpExecutor; }` + 每算子一个 struct + Builder。
3. **GE 图引擎绑定范围收敛**：只绑 C API `aclgrph*`（parser 头文件）+ 必要的 graphStatus 常量；
   图对象句柄类型为不透明指针；`Session` 在 cann 层做生命周期（build/save）。
   与 GE 的 C++ 图构造库（graph/）划界：reinfer 构图用 C API parser 或 C++ GE 由其自决，本仓库只提供
   parser→build→save 链路（若有需要再加）。
4. **错误码族三轨**：`aclError`（L0 白名单）→ `aclnnStatus` → `graphStatus`；cann 层 `Error` 以
   `code: i32` 承载全部，分类白名单**按族前缀**（aclnnStatus：0=成功，其余 fail-closed Fatal；
   graphStatus 值域 verify 后定白名单）。
5. **GE 链接风险前置（P0 验证）**：先确定 GE API 所在的 .so（objdump/grep 检查 libge_common/
   libge_compiler/libexe_graph 中 aclgrphParseONNX 归属），再定链接指令。
6. **build.rs 扩展**：SYMBOLS 追加 GE/aclnn 符号；非 ffi 不受影响。

## Module Breakdown

| 模块 | 内容 |
|---|---|
| `cann-sys/src/acl_meta.rs` | aclTensor/aclTensorList/aclScalar 类型 + acl_meta.h 生命周期/访问器 |
| `cann-sys/src/acl_datatype.rs` | aclDataType/aclFormat/aclTranspose 枚举（出处注释 + 单测；⚠️ 头文件位置待验证） |
| `cann-sys/src/aclnn_ops.rs` | aclnnStatus + Matmul/Softmax/RMSNorm 两段式声明（头文件 aclnnop/aclnn_*.h） |
| `cann-sys/src/acl_grph.rs` | aclgrphParseONNX/FromMem/BuildModel/SaveModel + graphStatus（parser/onnx_parser.h） |
| `cann-sys/build.rs` | SYMBOLS 扩列 + GE 链接库（ffi 时） |
| `cann/src/tensor.rs` | Tensor/TensorList/Scalar RAII 封装（非 ffi 降级） |
| `cann/src/op/mod.rs` | `Operator` trait + `OpExecutor` + workspace 管理（RAII Vec<u8> or 设备内存？—— workspace 为 host 内存：GetWorkspaceSize 返回的是 device 侧所需 workspace（aclnn 下 workspace 为 host+malloc？verify） |
| `cann/src/op/matmul.rs` `softmax.rs` `rms_norm.rs` | 三算子封装 |
| `cann/src/graph.rs` | Session/Graph ONNX→builder→save |
| `cann/src/error.rs` | 三错误族映射（L0 白名单扩展） |
| `cann/src/lib.rs` | 模块声明 + 文档 |

## Interface Contracts — L1（拟定；同步 reinfer 002 后生效）

> ⚠️=待头文件核对（verify-list）；`aclnnStatus`/`graphStatus` 为 C 返回值类型（typedef int 或 enum）。

```rust
// ============ cann-sys ============
// acl_meta.h
pub type aclnnStatus = c_int;                          // aclnn 错误码族（0=成功）
pub type aclTensor = c_void;                           // 不透明
pub type aclScalar = c_void;
pub type aclTensorList = c_void;
pub fn aclCreateTensor(viewDims: *const i64, viewDimsNum: u64, dataType: aclDataType,
                       stride: *const i64, offset: i64, format: aclFormat, <...>) -> *mut aclTensor; // ⚠️ verify 完整签名
pub fn aclDestroyTensor(t: *const aclTensor) -> aclnnStatus;
pub fn aclGetViewShape(t: *const aclTensor, dims: *mut *mut i64, num: *mut u64) -> aclnnStatus;
pub fn aclGetViewStrides(...) / aclGetViewOffset(...) / aclGetFormat(...) / aclGetDataType(...) -> aclnnStatus;
// aclnnop/aclnn_matmul.h
pub fn aclnnMatmulGetWorkspaceSize(self_: *const aclTensor, mat2: *const aclTensor, out: *mut aclTensor,
    cubeMathType: i8, workspaceSize: *mut u64, executor: *mut *mut c_void) -> aclnnStatus;
pub fn aclnnMatmul(workspace: *mut c_void, workspaceSize: u64, executor: *mut c_void, stream: *mut c_void) -> aclnnStatus;
// aclnn_softmax.h / aclnn_rms_norm.h：形态同 Matmul（⚠️ verify 参数：softmax 有 dim/dtype）
// parser/onnx_parser.h
pub fn aclgrphParseONNX(modelFile: *const c_char, graph: *mut aclgrphGraph) -> graphStatus; // ⚠️ verify 完整签名+graphStatus 类型
pub fn aclgrphParseONNXFromMem(buffer: *const c_char, size: usize, graph: *mut aclgrphGraph) -> graphStatus;
pub fn aclgrphBuildModel(...) -> graphStatus;  // ⚠️ verify
pub fn aclgrphSaveModel(...) -> graphStatus;   // ⚠️ verify

// ============ cann（安全层）============
pub struct Tensor;                                // RAII: aclCreateTensor/Drop->aclDestroyTensor
impl Tensor { pub fn new(dims: &[i64], dt: DataType, fmt: Format, offset: i64, strides: Option<&[i64]>) -> Result<Self, Error>;
              pub fn shape(&self) -> Result<Vec<i64>, Error>; pub fn data_type(&self) -> Result<DataType, Error>; ... }
pub enum DataType { Fp16, Fp32, Int32, ... }      // aclDataType 映射
pub enum Format { Nd, Nz, Nchw, ... }
pub trait Operator {                               // 统一两段式执行
    fn executor(&self) -> &OpExecutor;
    fn launch(&self, stream: &Stream) -> Result<(), Error>;
}
pub struct OpExecutor;                             // workspace + executor 句柄 RAII（host 或 device 内存由 verify 决定）
pub struct Matmul { pub fn new(a: &Tensor, b: &Tensor, out: &Tensor, cube_math_type: i8) -> Result<Self, Error>; }
pub struct Softmax { pub fn new(x: &Tensor, dim: i64, ...) -> Result<Self, Error>; }   // ⚠️ verify
pub struct RmsNorm { ... }
pub struct Graph { pub fn from_onnx(path: &Path) -> Result<Self, Error>; }   // RAII: aclgrphParseONNX
pub struct Session;                                // build/save
```

### Error 族映射（fail-closed）

| 错误族 | 白名单 | 默认 |
|---|---|---|
| aclnnStatus（0 成功） | `ACLNN_SUCCESS` | 非 0 = Fatal（当前未识别的 aclnn 码全部 Fatal） |
| graphStatus | verify 后定（GE 专用码表在 include/graph/ge_error_codes.h 或相关头文件） | 未知 Fatal |
| aclError | 沿用 L0 白名单 | Fatal |

## Risk Assessment

| 风险 | 缓解 |
|---|---|
| `aclCreateTensor` 完整签名与 `aclDataType`/`aclFormat` 枚举头文件位置 | verify-list P0 项，先用头文件 grep 圈定；枚举按源码抄录+单测 |
| aclnn workspace 语义（host/device、生命周期） | 查 aclnn 文档/头文件注释；封进 OpExecutor 文档中；真机验证 |
| GE 链接库集合未知（libge_common/compiler/x 哪个真被需） | P0：nm/grep 定位 aclgrph* 所在 .so，再改 build.rs lib 列表 |
| graphStatus 值域/GE 错误码族 | include/graph/ge_error_codes.h + parser 头文件 grep |
| Softmax/RMSNorm 参数形态 | verify-list：aclnn_softmax.h/aclnn_rms_norm.h 逐个签名 |
| 801 个算子后续批量 | 封装模式（Operator trait + macro 或 codegen）先跑通 3 个，后续批次套用 |

## Verify-list（写代码前逐项核实）

- [ ] `aclCreateTensor` 完整签名（acl_meta.h，12+ 参数）
- [ ] `aclDataType`/`aclFormat`/`aclTranspose` 定义的准确头文件与成员值（acl_base.h vs acl_meta.h）
- [ ] `aclnn_softmax.h`/`aclnn_rms_norm.h` 算子签名（softmax/rms 的 dim/dtype/scale 等参数形态）
- [ ] `aclgrph*` 完整签名 + `graphStatus` 类型与错误码族
- [ ] GE 链接库归属：`nm -D libge_common.so | grep aclgrph` 等
- [ ] `aclnnMatmul` 的 executor/workspace 生命周期语义（头文件注释）
