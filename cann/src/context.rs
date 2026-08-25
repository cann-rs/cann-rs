//! CANN 上下文（Context）安全封装。
//!
//! `Context` 为 `aclInit`/`aclFinalize` 的 RAII 封装。
//!
//! 线程亲和性：`aclInit`/`aclFinalize` 为进程级操作，与调用线程无关，`Context`
//! 可在任意线程创建/析构。但析构（`aclFinalize`）前必须先析构全部设备资源
//! （`Stream`/`Event`/`DeviceBuffer`/`HostBuffer`）并复位设备，否则运行时释放可能失败。

use crate::error::Error;

/// CANN 运行环境上下文（RAII 守护）。
///
/// 构造时调用 `aclInit(NULL)`（默认配置）初始化运行环境，析构时调用 `aclFinalize(0)` 释放。
/// 用法：在进程早期创建并保持存活，直到所有设备操作（stream/event/buffer）结束、
/// 设备复位之后。
///
/// 线程亲和性：本类型无内部状态，自动实现 `Send + Sync`；`aclInit`/`aclFinalize` 均为
/// 进程级调用，可在任意线程执行。注意本类型不绑定任何设备（`aclrtSetDevice` 的
/// per-thread 语义见 [`crate::device`]）。
#[derive(Debug)]
pub struct Context;

#[cfg(feature = "ffi")]
impl Context {
    /// 初始化 CANN 运行环境（对应 `aclInit(NULL)`）。
    ///
    /// 用法：必须在调用任何其他 ACL API 之前成功调用；本调用不设置设备，与
    /// `aclrtSetDevice` 无关。
    ///
    /// 失败时返回 `Err(Error)`：无 NPU 驱动时 `aclInit` 返回 100003/107xxx 类错误码。
    pub fn new() -> Result<Self, Error> {
        // SAFETY: `configPath` 传 NULL 使用默认配置；`aclInit` 为进程级初始化，
        // 不依赖线程或设备，可安全从任意线程调用。
        let ret = unsafe { cann_sys::aclInit(std::ptr::null()) };
        if ret != cann_sys::ACL_SUCCESS {
            return Err(Error::from(ret));
        }
        Ok(Context)
    }
}

#[cfg(feature = "ffi")]
impl Drop for Context {
    fn drop(&mut self) {
        // SAFETY: 本类型持有 `aclInit` 的配对权，析构时释放运行环境；`deviceId` 传 0 即可。
        // 前置条件：全部设备资源（Stream/Event/Buffer）已析构、设备已复位（见模块文档）。
        let _ = unsafe { cann_sys::aclFinalize(0) };
    }
}

/// 无 `ffi` 特性时的降级实现：不链接 `libascendcl`，统一返回"未启用"错误。
#[cfg(not(feature = "ffi"))]
impl Context {
    /// 初始化 CANN 运行环境（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`（code 为 -1，非 ACL 码）。
    #[allow(clippy::new_without_default)]
    pub fn new() -> Result<Self, Error> {
        Err(Self::unavailable())
    }

    /// `ffi` 未启用时的错误（code 为 -1，非 ACL 码；message 为中文说明）。
    fn unavailable() -> Error {
        Error {
            code: -1,
            message: "cann ffi 特性未启用，请以 --features ffi 构建".to_string(),
        }
    }
}

#[cfg(all(test, not(feature = "ffi")))]
mod tests {
    use super::*;

    #[test]
    fn new_returns_err_without_ffi() {
        assert!(Context::new().is_err());
    }
}

#[cfg(all(feature = "ffi", test))]
mod ffi_smoke {
    use super::*;

    #[test]
    #[ignore = "requires NPU driver"]
    fn init_and_drop_roundtrip() {
        let ctx = Context::new().unwrap();
        drop(ctx); // 析构触发 aclFinalize(0)
    }
}
