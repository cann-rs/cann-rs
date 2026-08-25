//! CANN Event 事件安全封装。
//!
//! `Event` 为 `aclrtCreateEvent`/`RecordEvent`/`SynchronizeEvent`/`StreamWaitEvent`/
//! `DestroyEvent` 的 RAII 封装，用于流间同步：在流上记录（`record`）、等待事件
//! （`synchronize`）、让其它流等待本事件（`stream_wait`）。
//!
//! 线程亲和性：事件句柄与创建线程绑定的设备上下文相关；跨线程使用前必须在目标线程
//! `set_device`（见 [`crate::device`]）。`Event` 不实现 `Send`/`Sync`——同一事件的
//! 并发等待需要外部同步，保守起见禁止跨线程移动。
//! 设备复位（`reset_device`）前必须先析构本事件。

use crate::error::Error;
use crate::stream::Stream;
#[cfg(feature = "ffi")]
use std::ffi::c_void;

/// Event 事件对象（RAII）。
///
/// 构造时调用 `aclrtCreateEvent` 创建事件，析构时调用 `aclrtDestroyEvent` 销毁。
/// 用法：`Context::new()` 与 `set_device` 之后创建；在流上 `record` 打点，等待侧
/// `synchronize`（本线程阻塞）或 `stream_wait`（另一流等待）。
///
/// 线程亲和性：不实现 `Send`/`Sync`；事件归属创建线程绑定的设备，跨线程使用前必须
/// 在目标线程 `set_device`，并由调用方保证并发安全。
#[derive(Debug)]
pub struct Event {
    #[cfg(feature = "ffi")]
    handle: *mut c_void,
}

#[cfg(feature = "ffi")]
impl Event {
    /// 创建新 Event 事件（对应 `aclrtCreateEvent`）。
    ///
    /// 用法：需已完成 `Context::new()` 且当前线程已 `set_device`；失败时返回 `Err(Error)`
    /// （如事件数量超限 207007 类错误码）。
    ///
    /// 线程亲和性：事件属于创建线程当前绑定的设备上下文。
    pub fn new() -> Result<Self, Error> {
        let mut handle: *mut c_void = std::ptr::null_mut();
        // SAFETY: `handle` 指向有效的 `*mut c_void` 输出槽位；调用前需 `aclInit` +
        // `set_device`（文档已注明）。
        let ret = unsafe { cann_sys::aclrtCreateEvent(&mut handle) };
        if ret != cann_sys::ACL_SUCCESS {
            return Err(Error::from(ret));
        }
        Ok(Event { handle })
    }

    /// 在指定流上记录本事件（对应 `aclrtRecordEvent`）。
    ///
    /// `stream` 传 `None` 表示当前线程的默认流（C 侧传 NULL）；`Some(&stream)` 记录在
    /// 显式流上。记录后的事件可通过 `synchronize`/`stream_wait` 等待。
    ///
    /// 线程亲和性：须在绑定事件归属设备的线程上调用；`stream` 必须属于同一设备上下文。
    pub fn record(&self, stream: Option<&Stream>) -> Result<(), Error> {
        let target = record_stream_ptr(stream);
        // SAFETY: `self.handle` 为合法事件句柄；`target` 为合法流句柄或 NULL（默认流）。
        let ret = unsafe { cann_sys::aclrtRecordEvent(self.handle, target) };
        if ret != cann_sys::ACL_SUCCESS {
            return Err(Error::from(ret));
        }
        Ok(())
    }

    /// 阻塞等待本事件被记录（对应 `aclrtSynchronizeEvent`）。
    ///
    /// 用法：等待此前 `record` 打点的事件发生；失败时返回 `Err(Error)`
    /// （如事件同步超时 507047 类错误码）。
    ///
    /// 线程亲和性：须在绑定事件归属设备的线程上调用。
    pub fn synchronize(&self) -> Result<(), Error> {
        // SAFETY: `self.handle` 为 `aclrtCreateEvent` 成功返回的有效句柄，
        // 且事件已通过 `record` 记录。
        let ret = unsafe { cann_sys::aclrtSynchronizeEvent(self.handle) };
        if ret != cann_sys::ACL_SUCCESS {
            return Err(Error::from(ret));
        }
        Ok(())
    }

    /// 让指定流等待本事件后再继续（对应 `aclrtStreamWaitEvent`）。
    ///
    /// 用法：事件被记录后，`stream` 上后续任务将等待事件发生；用于流间同步。
    ///
    /// 线程亲和性：`stream` 与事件必须属于同一设备上下文；须在绑定该设备的线程上调用。
    pub fn stream_wait(&self, stream: &Stream) -> Result<(), Error> {
        // SAFETY: `stream` 为合法流句柄（借用），`self.handle` 为合法事件句柄。
        let ret = unsafe { cann_sys::aclrtStreamWaitEvent(stream.raw_handle(), self.handle) };
        if ret != cann_sys::ACL_SUCCESS {
            return Err(Error::from(ret));
        }
        Ok(())
    }
}

/// 将 `Option<&Stream>` 整理为 C 侧流句柄：`None`（默认流）映射为 NULL 指针。
#[cfg(feature = "ffi")]
fn record_stream_ptr(stream: Option<&Stream>) -> *mut c_void {
    match stream {
        Some(s) => s.raw_handle(),
        None => std::ptr::null_mut(),
    }
}

#[cfg(feature = "ffi")]
impl Drop for Event {
    fn drop(&mut self) {
        // SAFETY: `self.handle` 来自 `aclrtCreateEvent` 且未被析构；本类型持有唯一所有权。
        // 注意：设备复位（`reset_device`）前必须先析构本事件。
        let _ = unsafe { cann_sys::aclrtDestroyEvent(self.handle) };
    }
}

/// 无 `ffi` 特性时的降级实现：不链接 `libascendcl`，统一返回"未启用"错误。
#[cfg(not(feature = "ffi"))]
impl Event {
    /// 创建新 Event 事件（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`（code 为 -1，非 ACL 码）。
    #[allow(clippy::new_without_default)]
    pub fn new() -> Result<Self, Error> {
        Err(Self::unavailable())
    }

    /// 在流上记录事件（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    #[allow(clippy::unused_self)]
    pub fn record(&self, _stream: Option<&Stream>) -> Result<(), Error> {
        Err(Self::unavailable())
    }

    /// 等待事件（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    #[allow(clippy::unused_self)]
    pub fn synchronize(&self) -> Result<(), Error> {
        Err(Self::unavailable())
    }

    /// 让流等待本事件（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    #[allow(clippy::unused_self)]
    pub fn stream_wait(&self, _stream: &Stream) -> Result<(), Error> {
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
        assert!(Event::new().is_err());
    }
}

/// `ffi` 特性下的纯参数整理测试：不触发任何 FFI 调用，无硬件依赖。
#[cfg(all(feature = "ffi", test))]
mod arg_tests {
    use super::*;

    #[test]
    fn record_none_maps_to_null_stream() {
        // `record(None)` 应将默认流参数整理为 NULL 句柄（C 侧语义：NULL = 默认流）。
        assert_eq!(record_stream_ptr(None), std::ptr::null_mut());
    }
}

#[cfg(all(feature = "ffi", test))]
mod ffi_smoke {
    use super::*;
    use crate::context::Context;
    use crate::device::{reset_device, set_device};
    use crate::stream::Stream;

    #[test]
    #[ignore = "requires NPU driver"]
    fn event_record_sync_and_stream_wait() {
        let _ctx = Context::new().unwrap();
        set_device(0).unwrap();
        let stream = Stream::new().unwrap();
        let event = Event::new().unwrap();
        event.record(Some(&stream)).unwrap();
        event.synchronize().unwrap();
        event.stream_wait(&stream).unwrap();
        event.record(None).unwrap(); // 默认流
        event.synchronize().unwrap();
        // 析构顺序：先事件/流，后复位设备，最后 _ctx 触发 aclFinalize。
        drop(event);
        drop(stream);
        reset_device(0).unwrap();
    }
}
