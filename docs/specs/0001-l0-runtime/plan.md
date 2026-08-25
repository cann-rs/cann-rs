# Plan：ACL L0 运行时绑定（Phase 1）

> Derived from docs/specs/0001-l0-runtime/spec.md

## Architecture Decision

1. **保持手写 extern**：~16 个符号固定可控、零构建依赖、逐条 `# SAFETY` 可审计——不引入 bindgen。
2. **存在性探测 cfg**：build.rs `grep include/acl/*.h`，对风险符号（跨版本可能漂移者）生成 `cann_sys_has_<sym>`；cann 层按 cfg 提供一致接口或标记不可用。8.5 锚点符号已核实，见 verify-list。
3. **模块拆分（cann-sys）**：沿用 `acl_rt.rs`（函数声明），新增 `acl_memory.rs`（内存原语 + 常量）、`acl_error_code.rs`（RT 错误码常量，全部带出处注释）、`acl_device.rs`（设备 + soc）。
4. **cann-sys 向后兼容**：不改现有声明；新符号同样挂在 `#[cfg(cann_sys_ffi)]` 内。
5. **build.rs 降级（P0）**：`CARGO_FEATURE_FFI` 未开启时跳过 SDK 硬检测（不再 `exit(1)`），只打印探测结果；`ffi` 开启时维持现有检测与链接逻辑。
6. **cann 层**：`context/device/stream/event/buffer/error` 模块化封装；所有指针语义收敛为 RAII 类型；线程亲和性（ACL per-thread device）在文档与 `debug_assert`（可选）中约束。

## Module Breakdown

| 模块 | 内容 |
|---|---|
| `cann-sys/src/acl_rt.rs` | 追加 device/stream/event 函数声明 |
| `cann-sys/src/acl_memory.rs` | aclrtMalloc/Free/Host/Memcpy + `ACL_MEM_*` 常量 |
| `cann-sys/src/acl_error_code.rs` | `ACL_ERROR_RT_*` 码表（来源: acl_error_code.h） |
| `cann-sys/build.rs` | 非 ffi 降级 + `cann_sys_has_*` 探测 + rerun-if-changed 扩列 |
| `cann/src/context.rs / device.rs / stream.rs / event.rs / buffer.rs / error.rs` | 安全封装 |

## Interface Contracts（与 reinfer 002 plan.md 锚点一致；`⚠️`=待头文件核实）

```rust
// ============ cann-sys（ffi 门禁内，签名以 AACL 头文件为准）============
pub fn aclrtGetDeviceCount(count: *mut u32) -> aclError;    // ✅ 官方: aclError aclrtGetDeviceCount(uint32_t *count) (aclcppdevg_03_0045)
pub fn aclrtSetDevice(deviceId: i32) -> aclError;            // ✅ (aclcppdevg_03_0039)
pub fn aclrtResetDevice(deviceId: i32) -> aclError;          // ✅ (aclcppdevg_03_0040)
pub fn aclrtCreateStream(stream: *mut *mut c_void) -> aclError;      // ✅ (aclcppdevg_03_0066)
pub fn aclrtDestroyStream(stream: *mut c_void) -> aclError; // ✅
pub fn aclrtCreateEvent(evt: *mut *mut c_void) -> aclError; // ✅ (aclcppdevg_03_0079)
pub fn aclrtRecordEvent(evt: *mut c_void, stream: *mut c_void) -> aclError; // ✅ (aclcppdevg_03_0083; stream=NULL 表示默认流)
pub fn aclrtSynchronizeEvent(evt: *mut c_void) -> aclError; // ✅ (aclcppdevg_03_0088)
pub fn aclrtDestroyEvent(evt: *mut c_void) -> aclError; // ✅
pub fn aclrtMalloc(ptr: *mut *mut c_void, size: usize, policy: aclrtMemMallocPolicy) -> aclError; // ✅ 第三参数为枚举 (aclcppdevg_03_0095)
pub fn aclrtFree(ptr: *mut c_void) -> aclError;
pub fn aclrtMallocHost(ptr: *mut *mut c_void, size: usize) -> aclError;
pub fn aclrtFreeHost(ptr: *mut c_void) -> aclError;
pub fn aclrtMemcpy(dst: *mut c_void, dst_max: usize, src: *const c_void, count: usize, kind: aclrtMemcpyKind) -> aclError; // ✅ 与官方完全一致 (aclcppdevg_03_0105)
pub fn aclrtGetSocName() -> *const c_char;                  // ✅ 官方: const char *aclrtGetSocName(void) (aclcppdevg_03_0048)
pub const ACL_MEM_MALLOC_HUGE_FIRST: u32;   // ... HUGE_ONLY / NORMAL_ONLY / NORMAL_FIRST (aclrtMemMallocPolicy)
pub const ACL_MEMCPY_HOST_TO_DEVICE / DEVICE_TO_HOST / DEVICE_TO_DEVICE / HOST_TO_HOST: aclrtMemcpyKind;

// ============ cann（安全层）============
pub struct Context;                                  // RAII: aclInit/Drop->aclFinalize
impl Context { pub fn init() -> Result<Self, Error>; }
pub fn device_count() -> Result<u32, Error>;
pub fn set_device(dev: u32) -> Result<(), Error>;    // per-thread 语义：调用所在线程生效
pub struct Stream;                                    // aclrtCreateStream/Drop
pub struct Event;                                     // Record(stream)/Synchronize/Drop
pub struct DeviceBuffer;                              // → impl Send（决策已定）
impl DeviceBuffer {
    pub fn alloc(size: usize, flags: MemFlags) -> Result<Self, Error>; // MemFlags 映射 ACL_MEM_MALLOC_*
    pub fn as_ptr(&self) -> *const u8;
    // SAFETY(impl Send)：ACL runtime 保证设备指针可跨线程使用；仅限归属 device
}
pub struct HostBuffer;                                // pinned host 内存
pub enum Error { code: aclError, message: String }
impl Error {
    pub fn is_oom(&self) -> bool;                     // 码段白名单（见表）
    pub fn is_recoverable(&self) -> bool;             // 码段白名单（fail-closed）
}
```

### Error 分类：码段白名单（fail-closed）

| 类别 | 白名单码段（数值以头文件为准） | 语义 |
|---|---|---|
| OOM | `ACL_ERROR_RT_MEMORY_ALLOCATION` / `ACL_ERROR_RT_MEMORY_FREE` | **reinfer 映射 `LaunchError::Oom`**（驱逐→重试） |
| Recoverable | 驱动/上下文类（`ACL_ERROR_RT_INTERNAL_ERROR`、上下文丢失类码段） | reinfer 映射 `LaunchError::Driver`（重建上下文） |
| 其他/未知 | 未列出的全部 | `false` → reinfer 映射 `LaunchError::Fatal`（fail-closed） |

> **不确定即失败**：新 SDK 出现的新错误码默认视为 Fatal，必须显式加入白名单才能放宽——避免错误分类的静默漂移。

## Risk Assessment

| 风险 | 缓解 |
|---|---|
| 符号在 8.x/9.x 改名 | 已按 CANN 8.5.0 官方文档核定（详见 verify-list 附录）；仍保留 build.rs 存在性探测兜底 |
| RT 错误码数值漂移 | `acl_error_code.h` 逐条核对并写出处注释；白名单机制 fail-closed |
| 无 SDK 时 CI 断裂 | build.rs 降级（P0 任务）先落地再补绑定 |
| 线程亲和性（per-thread set_device）遭误用 | 文档 + 可选 debug_assert（Task 8 可选） |
| ffi 默认关导致链接类错误后置 | 特征矩阵 CI：`--features ffi` 开启的 job 在有 SDK 环境编译 + smoke |

## Verify-list（2026-08-25 已按 CANN 8.5.0 官方文档核定，依据 `docs/cann-850-catalog.md` §2）

- [x] ~~`aclrtGetDeviceNum` vs `aclrtGetDeviceCount`~~ → ✅ 官方为 **`aclrtGetDeviceCount(uint32_t *count)`**（aclcppdevg_03_0045），已改契约
- [x] ~~`aclrtResetDevice` / `aclrtSynchronizeDevice` 存在性~~ → ✅ 存在（aclcppdevg_03_0040），另确认真机同步用 `aclrtSynchronizeDevice`（aclcppdevg_03_0056）
- [x] ~~`aclrtMemcpy` 形参顺序（dst, dstMax, src, count, kind）~~ → ✅ 与官方完全一致（aclcppdevg_03_0105）
- [x] ~~`aclrtGetSocName` 签名~~ → ✅ 官方 8.5 为 **`const char *aclrtGetSocName(void)`**（aclcppdevg_03_0048），无参返回指针，已改契约
- [x] ~~`aclrtMalloc` 第三参数类型~~ → ✅ **`aclrtMemMallocPolicy` 枚举**（aclcppdevg_03_0095），已改契约
- [ ] `ACL_ERROR_RT_*` 数值段（acl_error_code.h）——官方文档无码表页，待头文件实机核对
- [ ] `ACL_MEM_MALLOC_HUGE_FIRST/…` 数值顺序（acl_rt.h）——待头文件核对

> 契约锚点侧同步要求（R3）：`reinfer/specs/002-ascend-backend/plan.md` 的 L0 契约表需同改：
> `aclrtGetDeviceNum`/带缓冲区版 `aclrtGetSocName` 两处签名。此为跨仓库同步事项。
