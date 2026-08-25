//! CANN 设备管理安全封装。
//!
//! 提供设备数量查询、per-thread 设备绑定与 SOC 型号查询。
//!
//! 线程亲和性（重要）：ACL 的"当前设备"按调用线程绑定（`aclrtSetDevice`）。
//! `set_device`/`reset_device` 只影响调用线程；跨线程使用设备资源前，必须在目标线程
//! 显式调用 `set_device`。设备复位（`reset_device`）前必须先析构该设备上的全部
//! `Stream`/`Event`/`DeviceBuffer`/`HostBuffer`。

use crate::error::Error;
#[cfg(feature = "ffi")]
use std::ffi::CStr;

/// 查询本机可用设备数量（对应 `aclrtGetDeviceCount`）。
///
/// 用法：先 `Context::new()` 初始化，再调用本函数获取设备总数（设备逻辑 ID 范围为
/// `0..count-1`）。无 NPU 驱动或未初始化时返回 `Err(Error)`。
///
/// 线程亲和性：查询为进程级操作，不依赖调用线程的设备绑定。
#[cfg(feature = "ffi")]
pub fn device_count() -> Result<u32, Error> {
    let mut count: u32 = 0;
    // SAFETY: `count` 指向有效的 `u32` 输出槽位；调用前需 `aclInit`（文档已注明）。
    let ret = unsafe { cann_sys::acl_device::aclrtGetDeviceCount(&mut count) };
    if ret != cann_sys::ACL_SUCCESS {
        return Err(Error::from(ret));
    }
    Ok(count)
}

/// 将当前线程绑定到指定设备（对应 `aclrtSetDevice`）。
///
/// 用法：线程内首次使用设备前调用；每次调用使设备引用计数 +1，需与 `reset_device` 配对。
/// `dev` 超出实际设备数时返回 `Err(Error)`（如 107001 类错误码）。
///
/// 线程亲和性：**仅对调用线程生效**（per-thread 语义）。其他线程使用该设备前必须自行
/// 调用本函数；本线程的设备绑定不影响其他线程。
#[cfg(feature = "ffi")]
pub fn set_device(dev: u32) -> Result<(), Error> {
    // SAFETY: `deviceId` 需为合法设备 ID（越界时由运行时返回错误）；
    // 调用绑定调用线程的当前设备，per-thread 语义由文档说明。
    let ret = unsafe { cann_sys::acl_device::aclrtSetDevice(dev as i32) };
    if ret != cann_sys::ACL_SUCCESS {
        return Err(Error::from(ret));
    }
    Ok(())
}

/// 释放当前线程对指定设备的引用（对应 `aclrtResetDevice`）。
///
/// 与 `set_device` 配对使用：每次调用引用计数 −1，归零后才真正释放设备。
/// **复位前必须先析构该设备上显式创建的全部 `Stream`/`Event`/`DeviceBuffer`/
/// `HostBuffer`**，否则释放行为未定义。
///
/// 线程亲和性：仅影响调用线程对设备的引用。
#[cfg(feature = "ffi")]
pub fn reset_device(dev: u32) -> Result<(), Error> {
    // SAFETY: `deviceId` 需为当前线程已绑定的设备 ID；调用前必须已析构该设备上的
    // 全部显式资源（文档已注明）。
    let ret = unsafe { cann_sys::acl_device::aclrtResetDevice(dev as i32) };
    if ret != cann_sys::ACL_SUCCESS {
        return Err(Error::from(ret));
    }
    Ok(())
}

/// 查询当前线程设备的 SOC 型号名（对应 `aclrtGetSocName`）。
///
/// 用法：返回如 `"Ascend910B3"` 的型号字符串。需先 `Context::new()`；C 侧返回的指针由
/// 运行时持有、调用方不得释放，本函数只拷贝为 `String`。返回 NULL 或非 UTF-8 内容时
/// 返回 `Err(Error)`。
///
/// 线程亲和性：返回当前线程绑定设备的型号；未绑定设备时可能失败。
#[cfg(feature = "ffi")]
pub fn soc_name() -> Result<String, Error> {
    // SAFETY: `aclrtGetSocName` 无参，返回运行时持有的静态字符串指针（可能为 NULL）；
    // 须在 `aclInit` 之后调用（前置条件由调用方保证，见模块文档）。
    let ptr = unsafe { cann_sys::acl_device::aclrtGetSocName() };
    if ptr.is_null() {
        return Err(Error::from(cann_sys::ACL_ERROR_UNINITIALIZE));
    }
    // SAFETY: `ptr` 已判空，且 C 端保证指向 NUL 结尾的静态字符串
    // （运行时持有，仅在本函数作用域内读取）。
    let name = unsafe { CStr::from_ptr(ptr) };
    match name.to_str() {
        Ok(s) => Ok(s.to_string()),
        Err(_) => Err(Error {
            code: -1,
            message: "SOC 型号名不是合法 UTF-8 字符串".to_string(),
        }),
    }
}

/// 无 `ffi` 特性时的降级实现：不链接 `libascendcl`，统一返回"未启用"错误。
#[cfg(not(feature = "ffi"))]
pub fn device_count() -> Result<u32, Error> {
    Err(unavailable())
}

/// 将当前线程绑定到指定设备（需要 `ffi` 特性）。
///
/// 未启用 `ffi` 特性时返回 `Err(Error)`。语义同 ffi 实现：per-thread 设备绑定。
#[cfg(not(feature = "ffi"))]
pub fn set_device(_dev: u32) -> Result<(), Error> {
    Err(unavailable())
}

/// 释放当前线程对指定设备的引用（需要 `ffi` 特性）。
///
/// 未启用 `ffi` 特性时返回 `Err(Error)`。
#[cfg(not(feature = "ffi"))]
pub fn reset_device(_dev: u32) -> Result<(), Error> {
    Err(unavailable())
}

/// 查询 SOC 型号名（需要 `ffi` 特性）。
///
/// 未启用 `ffi` 特性时返回 `Err(Error)`。
#[cfg(not(feature = "ffi"))]
pub fn soc_name() -> Result<String, Error> {
    Err(unavailable())
}

/// `ffi` 未启用时的错误（code 为 -1，非 ACL 码；message 为中文说明）。
#[cfg(not(feature = "ffi"))]
fn unavailable() -> Error {
    Error {
        code: -1,
        message: "cann ffi 特性未启用，请以 --features ffi 构建".to_string(),
    }
}

#[cfg(all(test, not(feature = "ffi")))]
mod tests {
    use super::*;

    #[test]
    fn device_count_returns_err() {
        assert!(device_count().is_err());
    }

    #[test]
    fn set_device_returns_err() {
        assert!(set_device(0).is_err());
    }

    #[test]
    fn reset_device_returns_err() {
        assert!(reset_device(0).is_err());
    }

    #[test]
    fn soc_name_returns_err() {
        assert!(soc_name().is_err());
    }
}

#[cfg(all(feature = "ffi", test))]
mod ffi_smoke {
    use super::*;

    #[test]
    #[ignore = "requires NPU driver"]
    fn count_set_soc_name_reset() {
        let _ctx = crate::test_shared_ctx();
        let count = device_count().unwrap();
        assert!(count > 0, "device count should be > 0, got {count}");
        set_device(0).unwrap();
        let name = soc_name().unwrap();
        assert!(name.starts_with("Ascend"), "unexpected soc name: {name}");
        reset_device(0).unwrap();
    }
}
