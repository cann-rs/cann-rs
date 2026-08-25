# Tasks：ACL L0 运行时绑定（Phase 1）

> Derived from plan.md · 每条可独立验证；验证 = 对应验收项

## Task 1: build.rs 降级与探测骨架（P0，先行）

- 非 ffi 构建：跳过 SDK 硬检测（不再 `exit(1)`），打印探测信息；ffi 构建保持现有链接/rpath 逻辑
- 新增 `cann_sys_has_*` 存在性探测（grep include/acl/*.h → rustc-cfg）
- `rerun-if-changed` 增加 `acl_error_code.h`、`acl_memory.h` 等
- 验证：无 SDK 环境 `cargo check --workspace` 通过；有 SDK + `--features ffi` 链接通过

## Task 2: RT 错误码常量（acl_error_code.rs）

- 从 `acl_error_code.h` 抄录 `ACL_ERROR_RT_*`（内存/驱动/参数类，含数值出处注释）
- 验证：单元测试逐个断言数值 == 头文件 grep 结果（脚本化 diff，防漂移）

## Task 3: 设备与 SOC（acl_rt.rs + acl_device.rs）

- 绑定 `aclrtGetDeviceNum`(⚠️) / `aclrtSetDevice` / `aclrtResetDevice`(⚠️) / `aclrtGetSocName`(⚠️)
- 验证：ffi 编译通过；声明带 `# SAFETY`；cfg 探测生效（改名时 fallback 路径可用）

## Task 4: 内存原语（acl_memory.rs）

- `aclrtMalloc/Free/MallocHost/FreeHost/Memcpy` + `ACL_MEM_MALLOC_*` / `ACL_MEMCPY_*` 常量
- 验证：smoke 测试（真机）分配/释放/拷贝往返；签名核对 verify-list

## Task 5: Stream/Event 绑定（acl_rt.rs）

- Create/Destroy Stream；Create/Record/Synchronize/Destroy Event
- 验证：真机 smoke：`stream 上 record` → `synchronize` 返回成功

## Task 6: cann 安全封装（context/device/stream/event/buffer）

- `Context::init`（RAII）、`device_count`、`set_device`、`Stream`、`Event`、`DeviceBuffer`（**impl Send** + `as_ptr`）、`HostBuffer`
- `cann/Cargo.toml`：`[features]` `ffi = ["cann-sys/ffi"]`，**默认关闭**
- 验证：`cargo test` 单测（构造/析构/Debug）；真机 smoke 全绿；`cargo doc` 通过

## Task 7: Error 分类（error.rs）

- `is_oom()` / `is_recoverable()` 按**码段白名单**实现；未知码 fail-closed（两个方法都返回 false）
- `Error` 增加 `From<aclError>`、`Display` 完善；与 reinfer `LaunchError` 映射对齐（引用 002 plan.md 映射表）
- 验证：白名单/非白名单码的单测表齐全；新增码默认 Fatal 的行为有专门测试

## Task 8: 线程亲和性文档与可选 debug_assert（Chore）

- 文档标注：ACL per-thread device 语义，跨线程使用必须显式 `set_device`
- 可选：`debug_assert` 校验 device 一致性
- 验证：注释/文档评审通过

## Task 9: CI 与文档收尾

- 三档 CI：无 SDK（cargo check/clippy/test）/ 有 SDK（--features ffi 编译）/ NPU（smoke，自托管）
- README API 表追加 L0 类型行；spec changelog 留档
- 验证：CI 三档各自绿；README 与实现一致

---

Completion gate：Tasks 1–9 accepted；verify-list 全部划线（⚠️ 清除）；与 reinfer specs/002 契约表逐项核对一致；两仓库 CI 绿。
