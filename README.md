# cann-rs

Huawei Ascend CANN NPU SDK 的 Rust 绑定与安全封装。

## 项目结构

```
cann-rs/
├── cann-sys/    # 原始 FFI 绑定（零外部依赖，链接 libascendcl.so）
└── cann/        # 安全 Rust 封装（类型安全、错误处理）
```

## 环境要求

- Linux（aarch64 / x86_64）
- CANN SDK 8.x 或 9.x（通过 `ASCEND_TOOLKIT_HOME` 环境变量指定安装路径）
- NPU 驱动（版本查询功能需要）

## 安装

```toml
# Cargo.toml
[dependencies]
cann = "0.1"
```

或仅使用 FFI 绑定层：

```toml
cann-sys = { version = "0.1", features = ["ffi"] }
```

## 快速开始

```rust
use cann::version::Version;

fn main() {
    match Version::str() {
        Ok(v) => println!("CANN 版本: {}", v),
        Err(e) => println!("CANN 版本: 未检测到 ({})", e),
    }
    match Version::num() {
        Ok(n) => println!("CANN 版本号: {}", n),
        Err(e) => println!("CANN 版本号: 未检测到 ({})", e),
    }
}
```

## API

### `cann` crate

| 类型 | 说明 |
|------|------|
| `Version::str() -> Result<String, Error>` | 查询 CANN 版本字符串（如 `"9.0.0"`） |
| `Version::num() -> Result<i32, Error>` | 查询 CANN 版本号（如 `90_000_000`） |
| `Context`（RAII） | `aclInit`/`aclFinalize` 运行环境生命周期 |
| `device_count() -> Result<u32, Error>` | 可用设备数量（`aclrtGetDeviceCount`） |
| `set_device(dev)` / `reset_device(dev)` | per-thread 设备绑定与释放（引用计数配对） |
| `soc_name() -> Result<String, Error>` | 当前设备 SOC 型号（如 `"Ascend910B3"`） |
| `Stream`（RAII） | 流创建/销毁 + `synchronize()`/`query()` |
| `Event`（RAII） | 事件创建/销毁；`record(stream)`/`synchronize()`/`stream_wait(stream)` |
| `DeviceBuffer`（RAII, Send） | 设备内存：`alloc(size, MemFlags)`/`as_ptr()`/`len()` |
| `HostBuffer`（RAII） | 锁页主机内存：`alloc(size)`/`as_ptr()`/`len()` |
| `Error::is_oom()` / `is_recoverable()` | 错误分类（码段白名单，未知码 fail-closed） |
| `Error { code, message }` | CANN 操作错误 |

### `cann-sys` crate

| 项目 | 说明 |
|------|------|
| `aclError` | ACL 返回码类型 |
| `ACL_SUCCESS` / `ACL_ERROR_*` | 错误码常量 |
| `aclsysGetVersionStr` | FFI：查询版本字符串 |
| `aclsysGetVersionNum` | FFI：查询版本号（整数） |
| `aclrtGetVersion` | FFI：查询运行时组件版本 |

## 特性

### `ffi` 特性

`cann-sys` 的 `ffi` 特性控制是否链接 `libascendcl.so` 并暴露 FFI 函数声明。

- **启用**：链接原生库，可调用版本查询等 FFI 函数
- **关闭（默认）**：仅提供类型和常量定义，无需 NPU 驱动即可编译

## 注意事项

- 版本查询（`aclsysGetVersionStr` / `aclsysGetVersionNum`）需要 NPU 驱动支持
- 无驱动时返回 `Err(Error)`，`Error.message` 包含中文描述
- 无 CANN SDK 时编译失败，`build.rs` 会打印搜索路径和提示

## SDK 路径检测顺序

1. `ASCEND_TOOLKIT_HOME` 环境变量
2. `ASCEND_HOME_PATH` 环境变量
3. `ASCEND_HOME` 环境变量
4. `$HOME/Ascend/cann`
5. `/usr/local/Ascend`

## License

MIT OR Apache-2.0
