//! CANN 版本查询。
//!
//! 通过 FFI 调用 `aclsysGetVersionStr` 和 `aclsysGetVersionNum` 获取 CANN 版本信息。
//! 这些调用需要 NPU 驱动支持；无驱动时返回错误。

use crate::error::Error;
#[cfg(all(feature = "ffi", cann_sdk_has_aclsys_get_version_str))]
use std::ffi::{CStr, CString};

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
        crate::ensure_acl_init()?;
        if let Some(ver) = aclsys_version_str() {
            return Ok(ver);
        }
        // ② 旧 API（枚举包名；pyacl 同款，server 有效）
        if let Ok(ver) = enum_version_str() {
            return Ok(ver);
        }
        // ③ 回退：runtime 组件版本（跨 SDK 语义稳定）
        let (major, minor, patch) = rt_version()?;
        Ok(format!("{major}.{minor}.{patch}"))
    }

    /// 版本查询回退路径（CANN 7.x：无 aclsys* 符号，用 `aclrtGetVersion`）。
    #[cfg(not(cann_sdk_has_aclsys_get_version_str))]
    pub fn str() -> Result<String, Error> {
        crate::ensure_acl_init()?;
        let (major, minor, patch) = rt_version()?;
        Ok(format!("{major}.{minor}.{patch}"))
    }

    /// 查询 CANN 版本号（整数形式）。
    ///
    /// 存在 `aclsysGetVersionNum`（CANN 8.x+）时使用之；7.x 由 `aclrtGetVersion` 推算。
    /// 需要 NPU 驱动；驱动不可用时返回 `Err(Error)`。
    #[cfg(cann_sdk_has_aclsys_get_version_str)]
    pub fn num() -> Result<i32, Error> {
        crate::ensure_acl_init()?;
        if let Some(n) = aclsys_version_num() {
            return Ok(n);
        }
        // ② 旧 API（枚举包名）
        if let Ok(s) = enum_version_str() {
            let parts: Vec<&str> = s.split('.').collect();
            if parts.len() >= 3
                && let (Ok(maj), Ok(min), Ok(pat)) = (
                    parts[0].parse::<i32>(),
                    parts[1].parse::<i32>(),
                    parts[2].parse::<i32>(),
                )
            {
                return Ok(maj * 10_000_000 + min * 100_000 + pat * 1000);
            }
        }
        // ③ 由 runtime 组件版本推算
        let (major, minor, patch) = rt_version()?;
        Ok(major * 10_000_000 + minor * 100_000 + patch * 1000)
    }

    /// 版本号回退路径（CANN 7.x）：`major*10_000_000 + minor*100_000 + patch*1000`。
    #[cfg(not(cann_sdk_has_aclsys_get_version_str))]
    pub fn num() -> Result<i32, Error> {
        crate::ensure_acl_init()?;
        let (major, minor, patch) = rt_version()?;
        Ok(major * 10_000_000 + minor * 100_000 + patch * 1000)
    }
}

/// `aclsysGetVersionStr` 候选包名遍历：不同 SDK 对包名支持不同
/// （"CANN" / "CANNToolkit" 等），逐个尝试直到成功。
#[cfg(all(feature = "ffi", cann_sdk_has_aclsys_get_version_str))]
fn aclsys_version_str() -> Option<String> {
    const PKG_CANDIDATES: [&str; 4] = ["CANN", "CANNToolkit", "acl", "ascendcl"];
    for pkg in PKG_CANDIDATES {
        let Ok(pkg_name) = CString::new(pkg) else {
            continue;
        };
        let mut buf = [0u8; cann_sys::ACL_PKG_VERSION_MAX_SIZE];
        // SAFETY: pkgName 是有效的 NUL 结尾 C 字符串；缓冲区 128 字节足够。
        let ret = unsafe {
            cann_sys::aclsysGetVersionStr(
                pkg_name.as_ptr(),
                buf.as_mut_ptr() as *mut std::ffi::c_char,
            )
        };
        if ret == cann_sys::ACL_SUCCESS {
            // SAFETY: 调用成功时缓冲区包含 NUL 结尾 C 字符串。
            let c_str = unsafe { CStr::from_ptr(buf.as_ptr() as *const std::ffi::c_char) };
            return Some(c_str.to_str().unwrap_or_default().to_string());
        }
    }
    None
}

#[cfg(all(feature = "ffi", cann_sdk_has_aclsys_get_version_str))]
fn aclsys_version_num() -> Option<i32> {
    const PKG_CANDIDATES: [&str; 4] = ["CANN", "CANNToolkit", "acl", "ascendcl"];
    for pkg in PKG_CANDIDATES {
        let Ok(pkg_name) = CString::new(pkg) else {
            continue;
        };
        let mut num = 0i32;
        // SAFETY: pkgName 是有效的 NUL 结尾 C 字符串；num 指向栈上 i32。
        let ret = unsafe { cann_sys::aclsysGetVersionNum(pkg_name.as_ptr(), &mut num) };
        if ret == cann_sys::ACL_SUCCESS {
            return Some(num);
        }
    }
    None
}

/// 旧 API `aclsysGetCANNVersion`（枚举包名，pyacl 同款调用路径）。
#[cfg(all(feature = "ffi", cann_sdk_has_aclsys_get_version_str))]
fn enum_version_str() -> Result<String, Error> {
    let mut ver: cann_sys::aclCANNPackageVersion = unsafe { std::mem::zeroed() };
    // SAFETY: `ver` 为有效结构指针；`ACL_PKG_NAME_CANN` 为合法枚举。
    let ret = unsafe {
        cann_sys::aclsysGetCANNVersion(cann_sys::aclCANNPackageName::ACL_PKG_NAME_CANN, &mut ver)
    };
    if ret != cann_sys::ACL_SUCCESS {
        return Err(Error::from(ret));
    }
    // SAFETY: 结构以 NUL 结尾字符数组承载版本字段。
    let c_str = unsafe { CStr::from_ptr(ver.version.as_ptr()) };
    Ok(c_str.to_str().unwrap_or_default().to_string())
}

#[cfg(feature = "ffi")]
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
    fn test_version_str_well_formed() {
        let v = Version::str().unwrap();
        assert!(v.contains('.') && !v.is_empty(), "unexpected version: {v}");
    }

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_version_num_positive() {
        let n = Version::num().unwrap();
        assert!(n > 0, "unexpected version num: {n}");
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
