//! CANN 版本查询。
//!
//! 通过 FFI 调用 `aclsysGetVersionStr` 和 `aclsysGetVersionNum` 获取 CANN 版本信息。
//! 这些调用需要 NPU 驱动支持；无驱动时返回错误。

use crate::error::Error;
#[cfg(feature = "ffi")]
use std::ffi::CStr;
#[cfg(feature = "ffi")]
use std::sync::OnceLock;

/// CANN 版本查询接口。
pub struct Version;

#[cfg(feature = "ffi")]
impl Version {
    /// 查询 CANN 版本字符串。
    ///
    /// 存在 `aclsysGetVersionStr`（CANN 8.x+）时使用之；7.x 回退 `aclrtGetVersion`。
    /// 需要 NPU 驱动；驱动不可用时返回 `Err(Error)`。
    #[cfg(cann_sdk_has_aclsys_get_version_str)]
    pub fn str() -> Result<String, Error> {
        ensure_init_once()?;
        let pkg_name = c"CANN".as_ptr();
        let mut buf = [0u8; cann_sys::ACL_PKG_VERSION_MAX_SIZE];
        // SAFETY: pkgName 是有效的 NUL 结尾 C 字符串。
        // versionStr 缓冲区长度为 128 字节，足够容纳任何 CANN 版本号。
        let ret = unsafe {
            cann_sys::aclsysGetVersionStr(pkg_name, buf.as_mut_ptr() as *mut std::ffi::c_char)
        };
        if ret != cann_sys::ACL_SUCCESS {
            return Err(Error::from(ret));
        }
        // SAFETY: FFI 调用成功，缓冲区包含有效的 NUL 结尾 C 字符串。
        let c_str = unsafe { CStr::from_ptr(buf.as_ptr() as *const std::ffi::c_char) };
        Ok(c_str.to_str().unwrap_or_default().to_string())
    }

    /// 版本查询回退路径（CANN 7.x：无 aclsys* 符号，用 `aclrtGetVersion`）。
    #[cfg(not(cann_sdk_has_aclsys_get_version_str))]
    pub fn str() -> Result<String, Error> {
        ensure_init_once()?;
        let (major, minor, patch) = rt_version()?;
        Ok(format!("{major}.{minor}.{patch}"))
    }

    /// 查询 CANN 版本号（整数形式）。
    ///
    /// 存在 `aclsysGetVersionNum`（CANN 8.x+）时使用之；7.x 由 `aclrtGetVersion` 推算。
    /// 需要 NPU 驱动；驱动不可用时返回 `Err(Error)`。
    #[cfg(cann_sdk_has_aclsys_get_version_str)]
    pub fn num() -> Result<i32, Error> {
        ensure_init_once()?;
        let pkg_name = c"CANN".as_ptr();
        let mut num = 0i32;
        // SAFETY: pkgName 是有效的 NUL 结尾 C 字符串。
        // versionNum 指向栈上有效的 i32 变量。
        let ret = unsafe { cann_sys::aclsysGetVersionNum(pkg_name, &mut num) };
        if ret != cann_sys::ACL_SUCCESS {
            return Err(Error::from(ret));
        }
        Ok(num)
    }

    /// 版本号回退路径（CANN 7.x）：`major*10_000_000 + minor*100_000 + patch*1000`。
    #[cfg(not(cann_sdk_has_aclsys_get_version_str))]
    pub fn num() -> Result<i32, Error> {
        ensure_init_once()?;
        let (major, minor, patch) = rt_version()?;
        Ok(major * 10_000_000 + minor * 100_000 + patch * 1000)
    }
}

/// 进程级单次 `aclInit`：CANN 的初始化是进程全局且不可重复（7.x 上
/// `aclInit` 二次调用返回 `ACL_ERROR_REPEAT_INITIALIZE`），版本探测共用一次。
#[cfg(feature = "ffi")]
fn ensure_init_once() -> Result<(), Error> {
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
#[cfg(all(feature = "ffi", not(cann_sdk_has_aclsys_get_version_str)))]
fn rt_version() -> Result<(i32, i32, i32), Error> {
    let mut major = 0i32;
    let mut minor = 0i32;
    let mut patch = 0i32;
    // SAFETY: 三个参数均指向栈上有效的 i32 变量；需在 aclInit 之后调用。
    let ret = unsafe { cann_sys::aclrtGetVersion(&mut major, &mut minor, &mut patch) };
    if ret != cann_sys::ACL_SUCCESS {
        return Err(Error::from(ret));
    }
    Ok((major, minor, patch))
}

/// 无 `ffi` 特性时的降级实现：不链接 `libascendcl`，统一返回"未启用"错误。
#[cfg(not(feature = "ffi"))]
impl Version {
    /// `ffi` 未启用时的错误（code 为 -1，非 ACL 码；message 为中文说明）。
    fn unavailable() -> Error {
        Error {
            code: -1,
            message: "cann ffi 特性未启用，请以 --features ffi 构建".to_string(),
        }
    }

    /// 查询 CANN 版本字符串（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    pub fn str() -> Result<String, Error> {
        Err(Self::unavailable())
    }

    /// 查询 CANN 版本号（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    pub fn num() -> Result<i32, Error> {
        Err(Self::unavailable())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_version_str_9_0_0() {
        let v = Version::str().unwrap();
        assert_eq!(v, "9.0.0");
    }

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_version_num_9_0_0() {
        let n = Version::num().unwrap();
        assert_eq!(n, 90_000_000);
    }

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_str_and_num_consistent() {
        let v = Version::str().unwrap();
        let n = Version::num().unwrap();
        let parts: Vec<&str> = v.split('.').collect();
        assert_eq!(parts.len(), 3);
        let major: i64 = parts[0].parse().unwrap();
        let minor: i64 = parts[1].parse().unwrap();
        let patch: i64 = parts[2].parse().unwrap();
        let expected = major * 10_000_000 + minor * 100_000 + patch * 1000;
        assert_eq!(n as i64, expected);
    }
}
