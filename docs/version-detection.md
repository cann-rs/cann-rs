# CANN Version Detection

## Design

CANN 版本检测通过 FFI 调用 `aclsysGetVersionStr` / `aclsysGetVersionNum` 实现，**无降级逻辑**。

```
Version::str()
  └─① FFI: aclsysGetVersionStr("CANN", buf)
      ├─ ACL_SUCCESS  → 返回版本字符串
      └─ err          → 返回 Error

Version::num()
  └─① FFI: aclsysGetVersionNum("CANN", &num)
      ├─ ACL_SUCCESS  → 返回版本数值
      └─ err          → 返回 Error
```

## 运行条件

| 环境 | `Version::str()` | `Version::num()` |
|------|-----------------|-----------------|
| SDK + NPU 驱动 | `Ok("8.5.0")` | `Ok(80501000)` |
| SDK 无驱动 | `Err(ACL_ERROR_INVALID_FILE)` | 同上 |
| 无 SDK | 构建失败 | 构建失败 |

## 编译

```bash
cargo build --release
./target/release/cann
# 输出:
#   CANN version: 9.0.0
#   CANN version num: 90000000
# 或（无驱动）:
#   CANN version: not detected (CANN error (100003): Invalid file)
```

## 运行硬件测试

```bash
cargo test -- --ignored
```

## FFI 调用链路

`cann/Cargo.toml` → `cann-sys (features = ["ffi"])` → build.rs 链接 `libascendcl.so`
→ `acl_rt.rs` 的 `extern "C"` 块 → `version.rs` 的安全封装
