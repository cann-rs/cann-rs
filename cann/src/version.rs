use crate::error::Error;
use std::ffi::CStr;

pub struct Version;

impl Version {
    /// Query CANN version string via FFI call to `aclsysGetVersionStr`.
    /// Requires NPU driver; returns `Err` if unavailable.
    pub fn str() -> Result<String, Error> {
        let pkg_name = c"CANN".as_ptr();
        let mut buf = [0u8; cann_sys::ACL_PKG_VERSION_MAX_SIZE];
        // SAFETY: pkgName is a valid NUL-terminated C string.
        // versionStr buffer is 128 bytes, sufficient for any CANN version.
        let ret = unsafe {
            cann_sys::aclsysGetVersionStr(pkg_name, buf.as_mut_ptr() as *mut std::ffi::c_char)
        };
        if ret != cann_sys::ACL_SUCCESS {
            return Err(Error::from(ret));
        }
        // SAFETY: FFI call succeeded, buffer contains a valid NUL-terminated C string.
        let c_str = unsafe { CStr::from_ptr(buf.as_ptr() as *const std::ffi::c_char) };
        Ok(c_str.to_str().unwrap_or_default().to_string())
    }

    /// Query CANN version number via FFI call to `aclsysGetVersionNum`.
    /// Requires NPU driver; returns `Err` if unavailable.
    pub fn num() -> Result<i32, Error> {
        let pkg_name = c"CANN".as_ptr();
        let mut num = 0i32;
        // SAFETY: pkgName is a valid NUL-terminated C string.
        // versionNum points to a valid i32 on the stack.
        let ret = unsafe { cann_sys::aclsysGetVersionNum(pkg_name, &mut num) };
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
