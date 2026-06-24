//! CANN 版本查询。
//!
//! 通过 FFI 调用 `aclsysGetVersionStr` 和 `aclsysGetVersionNum` 获取 CANN 版本信息。
//! 这些调用需要 NPU 驱动支持；无驱动时返回错误。

use crate::error::Error;
use std::ffi::CStr;

/// CANN 版本查询接口。
pub struct Version;

impl Version {
    /// 查询 CANN 版本字符串。
    ///
    /// 调用 `aclInit(NULL)` 初始化后，通过 `aclsysGetVersionStr("CANN", ...)` 获取版本号（如 `"9.0.0"`）。
    /// 需要 NPU 驱动；驱动不可用时返回 `Err(Error)`。
    pub fn str() -> Result<String, Error> {
        // SAFETY: configPath 传 NULL 使用默认配置。
        let ret = unsafe { cann_sys::aclInit(std::ptr::null()) };
        if ret != cann_sys::ACL_SUCCESS {
            return Err(Error::from(ret));
        }
        let pkg_name = c"CANN".as_ptr();
        let mut buf = [0u8; cann_sys::ACL_PKG_VERSION_MAX_SIZE];
        // SAFETY: pkgName 是有效的 NUL 结尾 C 字符串。
        // versionStr 缓冲区长度为 128 字节，足够容纳任何 CANN 版本号。
        let ret = unsafe {
            cann_sys::aclsysGetVersionStr(pkg_name, buf.as_mut_ptr() as *mut std::ffi::c_char)
        };
        // SAFETY: 无论版本查询成功与否，都应释放运行环境资源。
        unsafe { cann_sys::aclFinalize(0) };
        if ret != cann_sys::ACL_SUCCESS {
            return Err(Error::from(ret));
        }
        // SAFETY: FFI 调用成功，缓冲区包含有效的 NUL 结尾 C 字符串。
        let c_str = unsafe { CStr::from_ptr(buf.as_ptr() as *const std::ffi::c_char) };
        Ok(c_str.to_str().unwrap_or_default().to_string())
    }

    /// 查询 CANN 版本号（整数形式）。
    ///
    /// 调用 `aclInit(NULL)` 初始化后，通过 `aclsysGetVersionNum("CANN", ...)` 获取版本号（如 `90_000_000`）。
    /// 需要 NPU 驱动；驱动不可用时返回 `Err(Error)`。
    pub fn num() -> Result<i32, Error> {
        // SAFETY: configPath 传 NULL 使用默认配置。
        let ret = unsafe { cann_sys::aclInit(std::ptr::null()) };
        if ret != cann_sys::ACL_SUCCESS {
            return Err(Error::from(ret));
        }
        let pkg_name = c"CANN".as_ptr();
        let mut num = 0i32;
        // SAFETY: pkgName 是有效的 NUL 结尾 C 字符串。
        // versionNum 指向栈上有效的 i32 变量。
        let ret = unsafe { cann_sys::aclsysGetVersionNum(pkg_name, &mut num) };
        // SAFETY: 无论版本查询成功与否，都应释放运行环境资源。
        unsafe { cann_sys::aclFinalize(0) };
        if ret != cann_sys::ACL_SUCCESS {
            return Err(Error::from(ret));
        }
        Ok(num)
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
