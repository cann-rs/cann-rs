#[allow(unused_imports)]
use crate::acl_base_rt::aclError;
#[allow(unused_imports)]
use std::ffi::c_char;

pub const ACL_PKG_VERSION_MAX_SIZE: usize = 128;
pub const ACL_PKG_VERSION_PARTS_MAX_SIZE: usize = 64;

#[allow(non_camel_case_types)]
#[repr(C)]
pub enum aclCANNPackageName {
    ACL_PKG_NAME_CANN,
    ACL_PKG_NAME_RUNTIME,
    ACL_PKG_NAME_COMPILER,
    ACL_PKG_NAME_HCCL,
    ACL_PKG_NAME_TOOLKIT,
    ACL_PKG_NAME_OPP,
    ACL_PKG_NAME_OPP_KERNEL,
    ACL_PKG_NAME_DRIVER,
}

#[cfg(cann_sys_ffi)]
unsafe extern "C" {
    // SAFETY: pkgName must be a valid NUL-terminated C string.
    // versionStr must point to a buffer of at least ACL_PKG_VERSION_MAX_SIZE bytes.
    pub fn aclsysGetVersionStr(pkgName: *const c_char, versionStr: *mut c_char) -> aclError;

    // SAFETY: pkgName must be a valid NUL-terminated C string.
    // versionNum must point to a valid i32.
    pub fn aclsysGetVersionNum(pkgName: *const c_char, versionNum: *mut i32) -> aclError;

    // SAFETY: majorVersion, minorVersion, patchVersion must point to valid i32.
    // Requires aclInit() to be called first.
    pub fn aclrtGetVersion(
        majorVersion: *mut i32,
        minorVersion: *mut i32,
        patchVersion: *mut i32,
    ) -> aclError;
}

#[cfg(all(cann_sys_ffi, test))]
mod tests {
    use super::*;
    use crate::acl_base_rt::*;
    use std::ffi::CString;

    #[test]
    fn test_version_max_size() {
        assert_eq!(ACL_PKG_VERSION_MAX_SIZE, 128);
    }

    #[test]
    fn test_version_parts_max_size() {
        assert_eq!(ACL_PKG_VERSION_PARTS_MAX_SIZE, 64);
    }

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_link_sys_get_version_str() {
        let pkg_name = CString::new("CANN").unwrap();
        let mut buf = [0u8; ACL_PKG_VERSION_MAX_SIZE];
        let ret =
            unsafe { aclsysGetVersionStr(pkg_name.as_ptr(), buf.as_mut_ptr() as *mut c_char) };
        assert_eq!(ret, ACL_SUCCESS);
    }

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_sys_get_version_str_content() {
        let pkg_name = CString::new("CANN").unwrap();
        let mut buf = [0u8; ACL_PKG_VERSION_MAX_SIZE];
        let ret =
            unsafe { aclsysGetVersionStr(pkg_name.as_ptr(), buf.as_mut_ptr() as *mut c_char) };
        assert_eq!(ret, ACL_SUCCESS);
        let version = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr() as *const c_char) };
        let version_str = version.to_str().unwrap();
        assert!(
            !version_str.is_empty(),
            "version string should not be empty"
        );
        assert!(
            version_str.contains('.'),
            "version string should contain dots"
        );
    }

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_link_sys_get_version_num() {
        let pkg_name = CString::new("CANN").unwrap();
        let mut num: i32 = 0;
        let ret = unsafe { aclsysGetVersionNum(pkg_name.as_ptr(), &mut num) };
        assert_eq!(ret, ACL_SUCCESS);
    }

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_sys_get_version_num_plausible() {
        let pkg_name = CString::new("CANN").unwrap();
        let mut num: i32 = 0;
        let ret = unsafe { aclsysGetVersionNum(pkg_name.as_ptr(), &mut num) };
        assert_eq!(ret, ACL_SUCCESS);
        assert!(
            (80_000_000..100_000_000).contains(&num),
            "version num {} out of expected range for CANN 8.x or 9.x",
            num
        );
    }
}
