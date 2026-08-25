// Huawei Ascend CANN NPU SDK 的安全 Rust 封装。
//!
//! 本 crate 基于 `cann-sys` 提供的 FFI 绑定，提供类型安全、内存安全的 CANN API。
//!
//! ## 模块
//! - [`error`] —— CANN 错误类型。
//! - [`version`] —— CANN 版本查询。
//! - [`context`] —— CANN 上下文（RAII）。
//! - [`device`] —— 设备管理。
//! - [`stream`] —— Stream 流。
//! - [`event`] —— Event 事件。
//! - [`buffer`] —— 设备/主机内存缓冲区。
//!
//! ## 线程亲和性
//!
//! ACL 的当前设备按调用线程绑定（`aclrtSetDevice`），跨线程使用设备资源
//! 前必须在目标线程显式 `set_device`。详见各模块文档。

/// 设备/主机内存缓冲区。
pub mod buffer;
/// CANN 上下文（RAII）。
pub mod context;
/// 设备管理。
pub mod device;
/// CANN 错误类型。
pub mod error;
/// Event 事件。
pub mod event;
/// GE 计算图（ONNX 解析与 .om 编译）。
pub mod graph;
/// aclnn 算子（Matmul/Softmax/RmsNorm）。
pub mod op;
/// Stream 流。
pub mod stream;
/// 张量/张量列表/标量等 aclnn 基础类型。
pub mod tensor;
/// CANN 版本查询。
pub mod version;

pub use crate::error::Error;

/// 真机 smoke 共享上下文：CANN 的 `aclInit` 是进程级单次（7.x 重复调用返回
/// `ACL_ERROR_REPEAT_INITIALIZE`），并行测试必须共享同一个实例（不 Drop）。
#[cfg(all(test, feature = "ffi"))]
pub(crate) fn test_shared_ctx() -> &'static context::Context {
    use std::sync::OnceLock;
    static CTX: OnceLock<context::Context> = OnceLock::new();
    CTX.get_or_init(|| context::Context::new().expect("test shared aclInit"))
}

/// 进程级单次 `aclInit`（幂等）：CANN 初始化全局唯一——7.x 重复调用返回
/// `ACL_ERROR_REPEAT_INITIALIZE`。`Context::new` 与版本探测共用本入口。
#[cfg(feature = "ffi")]
pub(crate) fn ensure_acl_init() -> Result<(), Error> {
    use std::sync::OnceLock;
    static RESULT: OnceLock<i32> = OnceLock::new();
    let code = *RESULT.get_or_init(|| {
        // SAFETY: configPath 传 NULL 使用默认配置；进程级初始化，幂等。
        unsafe { cann_sys::aclInit(std::ptr::null()) }
    });
    if code == cann_sys::ACL_SUCCESS {
        Ok(())
    } else {
        Err(Error::from(code))
    }
}

/// 进程级单次 `aclnnInit`（幂等）：aclnn 算子 API 需在 `aclInit` 之后另行初始化，
/// 重复调用返回失败。`OpExecutor` 首次创建时懒加载。
#[cfg(feature = "ffi")]
pub(crate) fn ensure_aclnn_init() -> Result<(), Error> {
    use std::sync::OnceLock;
    static RESULT: OnceLock<i32> = OnceLock::new();
    let code = *RESULT.get_or_init(|| {
        // SAFETY: configPath 传 NULL 使用默认配置；进程级初始化，幂等。
        unsafe { cann_sys::aclnn_ops::aclnnInit(std::ptr::null()) }
    });
    if code == cann_sys::aclnn_ops::ACLNN_SUCCESS {
        Ok(())
    } else {
        Err(Error::from(code))
    }
}

/// 顶层便捷重导出（常用类型）。
pub use context::Context;
pub use version::Version;
