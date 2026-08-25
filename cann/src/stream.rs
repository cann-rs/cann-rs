//! CANN Stream 流安全封装。
//!
//! `Stream` 为 `aclrtCreateStream`/`aclrtDestroyStream` 的 RAII 封装，提供同步等待
//! （`synchronize`）与任务状态查询（`query`）。
//!
//! 线程亲和性：流句柄与创建线程绑定的设备上下文相关；跨线程使用前必须在目标线程
//! `set_device`（见 [`crate::device`]）。`Stream` 不实现 `Send`/`Sync`——同一流的并发
//! 操作需要外部同步，保守起见禁止跨线程移动。
//! 设备复位（`reset_device`）前必须先析构本流。

use crate::error::Error;
#[cfg(feature = "ffi")]
use std::ffi::c_void;

/// Stream 流对象（RAII）。
///
/// 构造时调用 `aclrtCreateStream` 创建流，析构时调用 `aclrtDestroyStream` 销毁。
/// 用法：`Context::new()` 与 `set_device` 之后创建；流上提交的异步任务由后续
/// Task 提供，本类型提供 `synchronize`/`query` 两种等待/查询原语。
///
/// 线程亲和性：不实现 `Send`/`Sync`；如需跨线程使用流，必须在目标线程 `set_device`，
/// 并由调用方保证并发安全（流句柄可跨线程传递的语义由 CANN 提供，本封装不额外放宽）。
#[derive(Debug)]
pub struct Stream {
    #[cfg(feature = "ffi")]
    handle: *mut c_void,
}

#[cfg(feature = "ffi")]
impl Stream {
    /// 创建新 Stream 流（对应 `aclrtCreateStream`）。
    ///
    /// 用法：需已完成 `Context::new()` 且当前线程已 `set_device`；失败时返回 `Err(Error)`
    /// （如流数量超限 207008 类错误码）。
    ///
    /// 线程亲和性：流属于创建线程当前绑定的设备上下文；跨线程使用前必须在目标线程
    /// `set_device`。
    pub fn new() -> Result<Self, Error> {
        let mut handle: *mut c_void = std::ptr::null_mut();
        // SAFETY: `handle` 指向有效的 `*mut c_void` 输出槽位；调用前需 `aclInit` +
        // `set_device`（文档已注明）。
        let ret = unsafe { cann_sys::aclrtCreateStream(&mut handle) };
        if ret != cann_sys::ACL_SUCCESS {
            return Err(Error::from(ret));
        }
        Ok(Stream { handle })
    }

    /// 同步等待本流上的所有任务执行完成（对应 `aclrtSynchronizeStream`）。
    ///
    /// 用法：阻塞调用线程直到本流上已提交的任务全部完成；失败时返回 `Err(Error)`
    /// （如任务超时 507046 类错误码）。
    ///
    /// 线程亲和性：须在绑定本流归属设备的线程上调用。
    pub fn synchronize(&self) -> Result<(), Error> {
        // SAFETY: `self.handle` 为 `aclrtCreateStream` 成功返回的有效句柄；
        // 不得在流自身的回调函数中调用。
        let ret = unsafe { cann_sys::aclrtSynchronizeStream(self.handle) };
        if ret != cann_sys::ACL_SUCCESS {
            return Err(Error::from(ret));
        }
        Ok(())
    }

    /// 查询本流上任务是否全部完成（对应 `aclrtStreamQuery`）。
    ///
    /// 返回 `Ok(true)` 表示流空闲（C 状态码为 0）；`Ok(false)` 表示仍有任务在执行或失败
    /// （失败原因需另行诊断）。
    ///
    /// 线程亲和性：须在绑定本流归属设备的线程上调用。
    pub fn query(&self) -> Result<bool, Error> {
        let mut status: u32 = 0;
        // SAFETY: `self.handle` 为有效流句柄；`status` 指向有效的 `u32` 输出槽位。
        let ret = unsafe { cann_sys::aclrtStreamQuery(self.handle, &mut status) };
        if ret != cann_sys::ACL_SUCCESS {
            return Err(Error::from(ret));
        }
        Ok(status == 0)
    }

    /// 原始句柄（仅供 crate 内部跨模块使用，如 [`crate::event`] 的等待操作）。
    pub(crate) fn raw_handle(&self) -> *mut c_void {
        self.handle
    }
}

#[cfg(feature = "ffi")]
impl Drop for Stream {
    fn drop(&mut self) {
        // SAFETY: `self.handle` 来自 `aclrtCreateStream` 且未被析构；本类型持有唯一所有权。
        // 注意：设备复位（`reset_device`）前必须先析构本流。
        let _ = unsafe { cann_sys::aclrtDestroyStream(self.handle) };
    }
}

/// 无 `ffi` 特性时的降级实现：不链接 `libascendcl`，统一返回"未启用"错误。
#[cfg(not(feature = "ffi"))]
impl Stream {
    /// 创建新 Stream 流（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`（code 为 -1，非 ACL 码）。
    #[allow(clippy::new_without_default)]
    pub fn new() -> Result<Self, Error> {
        Err(Self::unavailable())
    }

    /// 同步等待本流任务完成（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    #[allow(clippy::unused_self)]
    pub fn synchronize(&self) -> Result<(), Error> {
        Err(Self::unavailable())
    }

    /// 查询本流任务状态（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    #[allow(clippy::unused_self)]
    pub fn query(&self) -> Result<bool, Error> {
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
        assert!(Stream::new().is_err());
    }
}

#[cfg(all(feature = "ffi", test))]
mod ffi_smoke {
    use super::*;
    use crate::device::{reset_device, set_device};

    #[test]
    #[ignore = "requires NPU driver"]
    fn stream_create_sync_query_drop() {
        let _ctx = crate::test_shared_ctx();
        set_device(0).unwrap();
        let stream = Stream::new().unwrap();
        stream.synchronize().unwrap();
        assert!(stream.query().unwrap(), "stream should be idle");
        drop(stream); // 析构触发 aclrtDestroyStream
        reset_device(0).unwrap();
    }
}
